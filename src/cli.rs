use clap::{Parser, Subcommand};
use colored::Colorize;

#[cfg(feature = "collector")]
use crate::collect::cli::{CollectCommands, handle_collect_command};

#[cfg(feature = "graph")]
use crate::graph::cli::{GraphCommands, handle_graph_command};

use crate::errors::CirroError;
use crate::styles;

#[derive(Debug, Parser)]
#[clap(
    about = clap::crate_description!(),
    version = clap::crate_version!(),
    bin_name = "cirro",
    author = clap::crate_authors!(),
    styles = styles::get_styles(),
    color = clap::ColorChoice::Always,
    arg_required_else_help = true,
)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Data collection commands
    #[cfg(feature = "collector")]
    Collect {
        #[command(subcommand)]
        command: CollectCommands,
    },
    /// Graph database commands
    #[cfg(feature = "graph")]
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
}

pub async fn handle_command(command: Option<Commands>) -> Result<(), CirroError> {
    let logo = r#"
   ___      _                            
  / __|    (_)      _ _     _ _    ___   
 | (__     | |     | '_|   | '_|  / _ \  
  \___|   _|_|_   _|_|_   _|_|_   \___/  
_|"""""|_|"""""|_|"""""|_|"""""|_|"""""| 
"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-' 
        "#;

    println!("{}", logo.blue().bold());

    match command {
        #[cfg(feature = "collector")]
        Some(Commands::Collect { command }) => {
            handle_collect_command(command).await?;
        }
        #[cfg(feature = "graph")]
        Some(Commands::Graph { command }) => {
            handle_graph_command(command).await?;
        }
        None => {}
    }

    Ok(())
}
