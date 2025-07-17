pub mod errors;
pub mod ingest;
pub mod logger;
pub mod styles;

use crate::errors::CirroIngestError;
use crate::logger::setup_logger;

use clap::{ColorChoice, Parser, crate_authors, crate_description, crate_version};
use colored::Colorize;
use std::path::PathBuf;
use tokio;

#[derive(Debug, Parser)]
#[clap(
    about = crate_description!(),
    version = crate_version!(),
    bin_name = "cirro-ingest",
    author = crate_authors!(),
    styles = styles::get_styles(),
    color = ColorChoice::Always,
    arg_required_else_help = true,
)]
#[command(disable_help_subcommand = true)]
struct Cli {
    /// Cirro results database file
    #[arg(short, long, value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    file: PathBuf,

    /// Database type
    #[arg(short, long, value_name = "GRAPH_TYPE", default_value = "neo4j")]
    graph_type: ingest::constants::GraphType,

    /// Database server. Possible schemes: bolt, bolt+s, bolt+ssc, neo4j, neo4j+s, neo4j+ssc
    #[arg(
        short,
        long,
        value_name = "SERVER",
        default_value = "bolt://localhost:7687"
    )]
    server: String,

    /// Database user
    #[arg(short, long, value_name = "USER", default_value = "neo4j")]
    user: String,

    /// Database password
    #[arg(short, long, value_name = "PASSWORD", default_value = "password")]
    password: String,

    /// Database name. Defaults to "neo4j" or "memgraph" depending on the graph type.
    #[arg(short, long, value_name = "NAME")]
    db_name: Option<String>,

    /// Enable debug logging
    #[arg(long, action = clap::ArgAction::SetTrue)]
    debug: bool,
}
#[tokio::main]
async fn main() -> Result<(), CirroIngestError> {
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
    if let Err(e) = setup_logger(cli.debug) {
        return Err(CirroIngestError::LogSetupError(e));
    }

    // Validate the database host
    if !cli.server.starts_with("bolt://")
        && !cli.server.starts_with("bolt+s://")
        && !cli.server.starts_with("bolt+ssc://")
        && !cli.server.starts_with("neo4j://")
        && !cli.server.starts_with("neo4j+s://")
        && !cli.server.starts_with("neo4j+ssc://")
    {
        return Err(CirroIngestError::InvalidConfig(
            "Database host must start with bolt://, bolt+s://, bolt+ssc://, neo4j://, neo4j+s://, or neo4j+ssc://".into(),
        ));
    }

    // Create the ingestor
    let mut ingestor = ingest::ingestor::CirroIngestor::new(
        cli.file,
        cli.graph_type,
        cli.server,
        cli.user,
        cli.password,
        cli.db_name,
    )
    .await;

    // Run the ingestor
    if let Err(e) = ingestor.run().await {
        return Err(e);
    }
    Ok(())
}
