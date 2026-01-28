use crate::collect::logger::setup_logger;
use crate::errors::CirroError;

use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

use crate::collect::tailscale::socket::TSSocketCollector;

/// Determine the default Tailscale socket path based on the operating system
/// https://github.com/tailscale/tailscale/blob/0a5639dcc008d60fe375a6707be1fec1ffc2ec53/paths/paths.go#L24
fn default_tailscaled_socket() -> String {
    if cfg!(target_os = "windows") {
        return r"\\.\pipe\ProtectedPrefix\Administrators\Tailscale\tailscaled".to_string();
    }

    if cfg!(target_os = "macos") {
        return "/var/run/tailscaled.socket".to_string();
    }

    // Linux - iterate through possible socket locations
    if cfg!(target_os = "linux") {
        let socket_paths = vec![
            "/var/run/tailscale/tailscaled.sock",
            "/run/tailscale/tailscaled.sock",
            "/var/packages/Tailscale/var/tailscaled.sock",
            "/var/packages/Tailscale/etc/tailscaled.sock",
            "/perm/tailscaled/tailscaled.sock",
            "/tmp/tailscale/tailscaled.sock",
        ];

        // Check each path to see if the socket exists
        for path in socket_paths {
            if fs::metadata(path).is_ok() {
                return path.to_string();
            }
        }
    }

    // Final fallback
    "tailscaled.sock".to_string()
}

fn validate_socket_path(path: &str) -> Result<PathBuf, String> {
    let pb = PathBuf::from(path);
    if pb.exists() {
        Ok(pb)
    } else {
        Err(format!(
            "The specified socket path does not exist: {}",
            path
        ))
    }
}

#[derive(Debug, Subcommand)]
pub enum TailscaleCommands {
    /// Collect data from local Tailscale socket
    Socket {
        /// Path to Tailscale socket (OS default used if not specified)
        /// For Linux, default is /var/run/tailscale/tailscaled.sock
        /// For Windows, default is \\.\pipe\tailscaled.sock
        #[arg(short, long, value_parser = validate_socket_path)]
        socket_path: Option<PathBuf>,

        /// Output file path (default: cirro_ts_status.json)
        #[arg(short, long, default_value = "cirro_ts_status.json")]
        output_path: PathBuf,

        /// Debug output
        #[arg(long = "debug", action = clap::ArgAction::SetTrue)]
        debug: bool,
    }, // Collect data using Tailscale API
       // Api {
       //     /// Tailscale API key
       //     #[arg(short, long, value_name = "API_KEY")]
       //     tailscale_key: String,

       //     /// Debug output
       //     #[arg(long = "debug", action = clap::ArgAction::SetTrue)]
       //     debug: bool,
       // },
}

pub async fn handle_tailscale_command(command: TailscaleCommands) -> Result<(), CirroError> {
    match command {
        TailscaleCommands::Socket {
            socket_path,
            output_path,
            debug,
        } => {
            if let Err(e) = setup_logger(debug) {
                return Err(CirroError::Unknown(e.to_string()));
            }
            let path = socket_path.unwrap_or_else(|| PathBuf::from(default_tailscaled_socket()));
            let collector = TSSocketCollector::new(path, output_path).await?;
            collector.run().await?;
        } // TailscaleCommands::Api {
          //     tailscale_key,
          //     debug,
          // } => {
          //     if let Err(e) = setup_logger(debug) {
          //         return Err(CirroError::Unknown(e.to_string()));
          //     }
          //     println!("Collecting data using Tailscale API key: {}", tailscale_key);
          //     // Implement API data collection logic here
          // }
    }
    Ok(())
}
