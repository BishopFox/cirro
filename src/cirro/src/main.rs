pub mod errors;
pub mod logger;
pub mod styles;

#[cfg(feature = "azure")]
pub mod azure;
#[cfg(feature = "azure")]
use crate::azure::cli::{AzureCommands, handle_azure_command};

#[cfg(feature = "tailscale")]
pub mod tailscale;
#[cfg(feature = "tailscale")]
use crate::tailscale::cli::{TailscaleCommands, handle_tailscale_command};

use crate::errors::CirroError;
use clap::{ColorChoice, Parser, Subcommand, crate_authors, crate_description, crate_version};
use colored::Colorize;
use tokio;

#[derive(Debug, Parser)]
#[clap(
    about = crate_description!(),
    version = crate_version!(),
    bin_name = "cirro",
    author = crate_authors!(),
    styles = styles::get_styles(),
    color = ColorChoice::Always,
    arg_required_else_help = true,
)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Azure commands
    #[cfg(feature = "azure")]
    Az {
        #[command(subcommand)]
        command: AzureCommands,
    },
    /// Tailscale commands
    #[cfg(feature = "tailscale")]
    Ts {
        #[command(subcommand)]
        command: TailscaleCommands,
    },
}

#[tokio::main]
async fn main() -> Result<(), CirroError> {
    let cli = Cli::parse();

    let logo = r#"
   ___      _                            
  / __|    (_)      _ _     _ _    ___   
 | (__     | |     | '_|   | '_|  / _ \  
  \___|   _|_|_   _|_|_   _|_|_   \___/  
_|"""""|_|"""""|_|"""""|_|"""""|_|"""""| 
"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-' 
        "#;

    println!("{}", logo.blue().bold());

    match cli.command {
        #[cfg(feature = "azure")]
        Some(Commands::Az { command }) => {
            handle_azure_command(command).await?;
        }
        #[cfg(feature = "tailscale")]
        Some(Commands::Ts { command }) => {
            handle_tailscale_command(command).await?;
        }
        None => {}
    }

    Ok(())
}
