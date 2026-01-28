#[cfg(feature = "azure")]
use crate::collect::azure::cli::{AzureCommands, handle_azure_command};

#[cfg(feature = "tailscale")]
use crate::collect::tailscale::cli::{TailscaleCommands, handle_tailscale_command};

use crate::errors::CirroError;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum CollectCommands {
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

pub async fn handle_collect_command(command: CollectCommands) -> Result<(), CirroError> {
    match command {
        #[cfg(feature = "azure")]
        CollectCommands::Az { command } => {
            handle_azure_command(command).await?;
        }
        #[cfg(feature = "tailscale")]
        CollectCommands::Ts { command } => {
            handle_tailscale_command(command).await?;
        }
    }

    Ok(())
}
