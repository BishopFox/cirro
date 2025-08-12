pub mod errors;
pub mod export;
pub mod ingest;
pub mod logger;
pub mod styles;

use crate::errors::CirroGraphError;
use crate::logger::setup_logger;

use clap::{ColorChoice, Parser, Subcommand, crate_authors, crate_description, crate_version};
use colored::Colorize;
use std::path::PathBuf;
use tokio;

#[derive(Debug, Parser)]
#[clap(
    about = crate_description!(),
    version = crate_version!(),
    bin_name = "cirro-graph",
    author = crate_authors!(),
    styles = styles::get_styles(),
    color = ColorChoice::Always,
    arg_required_else_help = true,
)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Ingest data into the database
    Ingest {
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
    },
    /// Export data from the database
    Export {
        /// Export type
        #[arg(short, long, value_name = "EXPORT_TYPE")]
        export_type: export::exporter::ExportType,

        /// Output file
        #[arg(short, long, value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
        output_file: PathBuf,

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
    },
}

#[tokio::main]
async fn main() -> Result<(), CirroGraphError> {
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
        Commands::Ingest {
            file,
            graph_type,
            server,
            user,
            password,
            db_name,
            debug,
        } => {
            if let Err(e) = setup_logger(debug) {
                return Err(CirroGraphError::LogSetupError(e));
            }
            // Validate the database host
            if !server.starts_with("bolt://")
                && !server.starts_with("bolt+s://")
                && !server.starts_with("bolt+ssc://")
                && !server.starts_with("neo4j://")
                && !server.starts_with("neo4j+s://")
                && !server.starts_with("neo4j+ssc://")
            {
                return Err(CirroGraphError::InvalidConfig(
            "Database host must start with bolt://, bolt+s://, bolt+ssc://, neo4j://, neo4j+s://, or neo4j+ssc://".into(),
        ));
            }

            // Create the ingestor
            let mut ingestor = ingest::ingestor::CirroIngestor::new(
                file, graph_type, server, user, password, db_name,
            )
            .await;

            // Run the ingestor
            if let Err(e) = ingestor.run().await {
                return Err(e);
            }
        }
        Commands::Export {
            export_type,
            output_file,
            graph_type,
            server,
            user,
            password,
            db_name,
            debug,
        } => {
            if let Err(e) = setup_logger(debug) {
                return Err(CirroGraphError::LogSetupError(e));
            }

            let mut exporter = export::exporter::CirroExporter::new(
                export_type,
                output_file,
                graph_type,
                server,
                user,
                password,
                db_name,
            )
            .await;
            if let Err(e) = exporter.run().await {
                return Err(e);
            }
        }
    }

    Ok(())
}
