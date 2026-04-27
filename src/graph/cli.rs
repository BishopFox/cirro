use crate::errors::*;
use crate::graph::export::exporter::CirroExporter;
use crate::graph::export::types::ExportFormat;
use crate::graph::ingest::ingestor::{CirroIngestor, IngestType};
use crate::graph::logger::setup_logger;

use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum GraphCommands {
    /// Ingest data into the database
    Ingest {
        /// Type of data to ingest
        #[arg(short, long, value_enum)]
        r#type: IngestType,

        /// File to ingest
        #[arg(short, long, value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
        file: PathBuf,

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

        /// Database name. Defaults to "neo4j".
        #[arg(short, long, value_name = "NAME")]
        db_name: Option<String>,

        /// Ingest only these nodes (comma-separated list of labels). Useful for testing or partial ingestions.
        #[arg(long, value_name = "LABELS", value_delimiter = ',', num_args = 1..)]
        labels: Option<Vec<String>>,

        /// Run only post-processing specs
        #[arg(long, action = clap::ArgAction::SetTrue)]
        post_process: bool,

        /// Show what would be ingested and post-processed without executing queries
        #[arg(long, action = clap::ArgAction::SetTrue)]
        dry_run: bool,

        /// Enable debug logging
        #[arg(long, action = clap::ArgAction::SetTrue)]
        debug: bool,
    },
    /// Export database in different formats
    Export {
        /// Format to export
        #[arg(short, long, value_name = "FORMAT")]
        format: ExportFormat,

        /// Output file path (default: ./cirro_export)
        #[arg(short, long, value_name = "OUTPUT", default_value = "cirro_export", value_hint = clap::ValueHint::FilePath)]
        output: PathBuf,

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

        /// Database name. Defaults to "neo4j".
        #[arg(short, long, value_name = "NAME")]
        db_name: Option<String>,

        /// Enable debug logging
        #[arg(long, action = clap::ArgAction::SetTrue)]
        debug: bool,
    },
}

pub async fn handle_graph_command(command: GraphCommands) -> Result<(), CirroError> {
    match command {
        GraphCommands::Ingest {
            r#type,
            file,
            server,
            user,
            password,
            db_name,
            labels,
            post_process,
            dry_run,
            debug,
        } => {
            if let Err(e) = setup_logger(debug) {
                return Err(CirroError::Unknown(e.to_string()));
            }

            // Validate the file exists and is readable
            if !file.exists() {
                return Err(CirroError::Unknown(format!(
                    "File does not exist: {}",
                    file.display()
                )));
            }
            if !file.is_file() {
                return Err(CirroError::Unknown(format!(
                    "Path is not a file: {}",
                    file.display()
                )));
            }
            // Check if file is readable by attempting to open it
            std::fs::File::open(&file).map_err(|e| {
                CirroError::Unknown(format!("File is not readable: {} - {}", file.display(), e))
            })?;

            // Validate the database host
            if !server.starts_with("bolt://")
                && !server.starts_with("bolt+s://")
                && !server.starts_with("bolt+ssc://")
                && !server.starts_with("neo4j://")
                && !server.starts_with("neo4j+s://")
                && !server.starts_with("neo4j+ssc://")
            {
                return Err(CirroError::Unknown(
            "Database host must start with bolt://, bolt+s://, bolt+ssc://, neo4j://, neo4j+s://, or neo4j+ssc://".into(),
        ));
            }

            // Create the ingestor
            let mut ingestor =
                CirroIngestor::new(r#type, file, server, user, password, db_name, labels).await;

            // Run the ingestor
            if let Err(e) = ingestor.run(post_process, dry_run).await {
                return Err(CirroError::Unknown(e.to_string()));
            }
        }

        GraphCommands::Export {
            format,
            output,
            server,
            user,
            password,
            db_name,
            debug,
        } => {
            if let Err(e) = setup_logger(debug) {
                return Err(CirroError::Unknown(e.to_string()));
            }
            // Validate the database host
            if !server.starts_with("bolt://")
                && !server.starts_with("bolt+s://")
                && !server.starts_with("bolt+ssc://")
                && !server.starts_with("neo4j://")
                && !server.starts_with("neo4j+s://")
                && !server.starts_with("neo4j+ssc://")
            {
                return Err(CirroError::Unknown(
                    "Database host must start with bolt://, bolt+s://, bolt+ssc://, neo4j://, neo4j+s://, or neo4j+ssc://".into(),
                ));
            }

            // Create the exporter
            let mut exporter =
                CirroExporter::new(format, output, server, user, password, db_name).await;

            // Run the exporter
            if let Err(e) = exporter.run().await {
                return Err(CirroError::Unknown(e.to_string()));
            }
        }
    }

    Ok(())
}
