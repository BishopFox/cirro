pub mod errors;
pub mod styles;

use clap::Parser;

#[cfg(feature = "collector")]
pub mod collect;

#[cfg(feature = "graph")]
pub mod graph;

use crate::errors::CirroError;

mod cli;
use cli::{Cli, handle_command};

#[tokio::main]
async fn main() -> Result<(), CirroError> {
    let cli = Cli::parse();
    handle_command(cli.command).await
}
