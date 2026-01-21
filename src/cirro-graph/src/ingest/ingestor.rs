use crate::errors::CirroGraphError;
use crate::specs::{SpecLoader, SpecRegistry, SpecTrait};

use clap::ValueEnum;
use log::{debug, info};
use neo4rs::*;
use rusqlite::{Connection, Result};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum IngestType {
    /// Azure data ingestion
    Az,
    /// Tailscale status data ingestion  
    TsStatus,
}
/// Ingestor used by all Cirro ingestion processes to manage database connections and ingestion specs
pub struct CirroIngestor {
    pub r#type: IngestType,
    pub file: PathBuf,
    pub host: String,
    pub user: String,
    pub password: String,
    pub db_name: String,
    pub graph: neo4rs::Graph,
    pub sql_conn: Option<Connection>,
    pub specs: SpecRegistry,
}

/// Custom Debug trait for CirroIngestor
impl std::fmt::Debug for CirroIngestor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CirroIngestor")
            .field("type", &self.r#type)
            .field("file", &self.file)
            .field("host", &self.host)
            .field("user", &self.user)
            .field("password", &self.password)
            .field("db_name", &self.db_name)
            .field("graph", &format_args!("<neo4rs::Graph>"))
            .field("specs", &format_args!("<SpecRegistry>"))
            .finish()
    }
}

impl CirroIngestor {
    pub async fn new(
        r#type: IngestType,
        file: PathBuf,
        host: String,
        user: String,
        password: String,
        db_name: Option<String>,
    ) -> Self {
        // Load all specs first before database connection
        info!("Loading ingestion specifications...");
        let specs = SpecLoader::load_all_specs()
            .map_err(|e| {
                panic!("Failed to load specs: {}", e);
            })
            .unwrap();

        // If db_name is not provided, use the default
        let db_name = db_name.unwrap_or_else(|| "neo4j".to_string());

        let config = ConfigBuilder::default()
            .uri(host.clone())
            .user(user.clone())
            .password(password.clone())
            .db(db_name.clone())
            .build()
            .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))
            .unwrap();

        info!("Connecting to database at {} with user {}", host, user);
        let graph = Graph::connect(config)
            .await
            .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))
            .unwrap();

        // Test connection to the database
        let mut result = graph.execute(query("RETURN 1")).await.unwrap();
        let row = result.next().await.unwrap().unwrap();
        let value: i64 = row.get("1").unwrap();
        assert_eq!(1, value);
        info!("Successfully connected to the database");

        let sql_conn = Connection::open(file.clone())
            .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))
            .ok();

        let ingestor = CirroIngestor {
            r#type,
            file: file,
            host,
            user,
            password,
            db_name,
            graph,
            sql_conn,
            specs,
        };
        return ingestor;
    }

    /// Runs the ingestor
    pub async fn run(&mut self) -> Result<(), CirroGraphError> {
        // Start stopwatch to measure the time taken for the entire ingestion process
        let start_time = std::time::Instant::now();
        debug!("Ingestion started at: {:?}", start_time);

        // Run the main ingestion logic
        match self.r#type {
            IngestType::Az => self.process_cirro_azure_ingest().await?,
            IngestType::TsStatus => self.process_cirro_tailscale_status_ingest().await?,
        }
        self.generic_post_process().await?;

        // Calculate the total time taken for the ingestion process
        let duration = start_time.elapsed();
        info!("Cirro ingestion process completed in: {:.2?}", duration);

        Ok(())
    }

    pub async fn create_constraints_and_indexes<T>(&self, spec: &T) -> Result<(), CirroGraphError>
    where
        T: SpecTrait,
    {
        let constraint_query =
            crate::ingest::CREATE_CONSTRAINT_QUERY.replace("{}", spec.get_label());
        debug!("Executing query: {}", constraint_query);
        let _ = self
            .graph
            .run(query(&constraint_query))
            .await
            .map_err(|e| {
                CirroGraphError::DatabaseError(format!(
                    "Failed to create constraint for {}: {}",
                    spec.get_name(),
                    e
                ))
            })?;
        let index_query = crate::ingest::CREATE_INDEX_QUERY
            .replace("{}", spec.get_label())
            .replace("{}", spec.get_label());
        debug!("Executing query: {}", index_query);
        let _ = self.graph.run(query(&index_query)).await.map_err(|e| {
            CirroGraphError::DatabaseError(format!(
                "Failed to create index for {}: {}",
                spec.get_name(),
                e
            ))
        })?;
        Ok(())
    }

    async fn generic_post_process(&mut self) -> Result<(), CirroGraphError> {
        info!("Running post-processing queries");

        // First, order the post-processing specs by priority
        let mut post_processing_specs = self.specs.cirro_post_processing_specs.clone();
        post_processing_specs.sort_by_key(|spec| spec.priority);

        for spec in &post_processing_specs {
            debug!("Running post-processing spec: {}", spec.name);
            let _ = self.graph.run(query(&spec.cypher)).await.map_err(|e| {
                CirroGraphError::DatabaseError(format!(
                    "Failed to execute post-processing query for spec {}: {}",
                    spec.name, e
                ))
            })?;
        }
        Ok(())
    }
}
