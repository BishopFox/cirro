pub mod dns;
pub mod enrich;
pub mod errors;
pub mod ingest;
pub mod logger;
pub mod styles;

use crate::errors::CirroGraphError;
use crate::logger::setup_logger;

use clap::{
    Args, ColorChoice, Parser, Subcommand, builder::ArgPredicate, crate_authors, crate_description,
    crate_version,
};
use colored::Colorize;
use serde::Serialize;
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
    /// Perform checks related to DNS resolution
    Dns {
        /// Output directory
        #[arg(short, long, default_value = ".", value_name = "OUTPUT_DIR", value_hint = clap::ValueHint::DirPath)]
        output_dir: PathBuf,

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
        db_name: String,

        /// Enable debug logging
        #[arg(long, action = clap::ArgAction::SetTrue)]
        debug: bool,
    },

    /// Export an enrichment config file based on provided flags
    Enrich {
        /// Output JSON file
        #[arg(short, long, default_value = "enrich_config.json", value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
        output_file: PathBuf,

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
        db_name: String,

        /// Enable debug logging
        #[arg(long, action = clap::ArgAction::SetTrue)]
        debug: bool,

        /// Object ID to analyze for permissions
        #[arg(short, long, value_name = "ID")]
        id: uuid::Uuid,

        /// Relationship depth for permission queries (default: 10)
        #[arg(long, value_name = "DEPTH", default_value = "10")]
        relation_depth: isize,

        #[clap(flatten)]
        flags: EnrichFlags,
    },
}

#[derive(Args, Debug, Serialize)]
#[group(required = true, multiple = true)]
pub struct EnrichFlags {
    /// Perform all checks
    #[arg(long, action = clap::ArgAction::SetTrue)]
    all: bool,

    /// Check for storage account keys
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_ifs([
            ("all", ArgPredicate::Equals("true".into()), "true"),
        ]))]
    storage_keys: bool,
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
        Commands::Dns {
            output_dir,
            server,
            user,
            password,
            db_name,
            debug,
        } => {
            if let Err(e) = setup_logger(debug) {
                return Err(CirroGraphError::LogSetupError(e));
            }

            let mut checker =
                dns::DnsChecker::new(output_dir, server, user, password, db_name).await;

            if let Err(e) = checker.run().await {
                return Err(e);
            }
        }
        Commands::Enrich {
            output_file,
            server,
            user,
            password,
            db_name,
            debug,
            id,
            relation_depth,
            flags,
        } => {
            if let Err(e) = setup_logger(debug) {
                return Err(CirroGraphError::LogSetupError(e));
            }

            let enricher = enrich::EnrichConfigurator::new(
                output_file,
                server,
                user,
                password,
                db_name,
                relation_depth,
                flags,
            )
            .await?;

            if let Err(e) = enricher.make_config(id).await {
                return Err(e);
            }
        }
    }

    Ok(())
}
