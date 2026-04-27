use crate::errors::CirroError;
use crate::graph::ingest::CREATE_CONSTRAINT_QUERY;
use crate::graph::specs::{SpecLoader, SpecRegistry, SpecTrait};
use clap::ValueEnum;
use log::{debug, info};
use neo4rs::*;
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use tokio::time::Duration;

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
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
    pub labels: Option<Vec<String>>,
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
        labels: Option<Vec<String>>,
    ) -> Self {
        // Load all specs first before database connection
        info!("Loading ingestion specifications...");

        let specs = SpecLoader::load_all_specs_filtered(labels.clone())
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
            .map_err(|e| CirroError::DatabaseError(e.to_string()))
            .unwrap();

        info!("Connecting to database at {} with user {}", host, user);
        let graph = Graph::connect(config)
            .await
            .map_err(|e| CirroError::DatabaseError(e.to_string()))
            .unwrap();

        // Test connection to the database
        let mut result = graph.execute(query("RETURN 1")).await.unwrap();
        let row = result.next().await.unwrap().unwrap();
        let value: i64 = row.get("1").unwrap();
        assert_eq!(1, value);
        info!("Successfully connected to the database");

        let sql_conn = Connection::open(file.clone())
            .map_err(|e| CirroError::DatabaseError(e.to_string()))
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
            labels,
        };
        return ingestor;
    }

    /// Runs the ingestor
    pub async fn run(&mut self, post_process_only: bool, dry_run: bool) -> Result<(), CirroError> {
        // Start stopwatch to measure the time taken for the entire ingestion process
        let start_time = std::time::Instant::now();
        debug!("Ingestion started at: {:?}", start_time);

        if dry_run {
            info!("Dry-run enabled: no ingestion or post-processing queries will be executed");
        }

        if !post_process_only {
            // Run the main ingestion logic
            match self.r#type {
                IngestType::Az => self.process_cirro_azure_ingest(dry_run).await?,
                IngestType::TsStatus => self.process_cirro_tailscale_status_ingest(dry_run).await?,
            }

            // Ensure all transactions are committed before post-processing
            if !dry_run {
                self.finalize_transactions().await?;
            }
        }

        if dry_run && self.r#type == IngestType::Az {
            self.show_unimplemented_azure_resource_types().await?;
        }

        self.generic_post_process(dry_run).await?;

        // Calculate the total time taken for the ingestion process
        if !dry_run {
            let duration = start_time.elapsed();
            info!("Cirro ingestion process completed in: {:.2?}", duration);
        }

        Ok(())
    }

    pub async fn create_constraints_by_name(&self, label_name: &str) -> Result<(), CirroError> {
        let constraint_query = CREATE_CONSTRAINT_QUERY
            .replacen("{}", label_name, 1)
            .replacen("{}", "id", 1);
        debug!("Executing query: {}", constraint_query);
        let _ = self
            .graph
            .run(query(&constraint_query))
            .await
            .map_err(|e| {
                CirroError::DatabaseError(format!(
                    "Failed to create constraint for {}: {}",
                    label_name, e
                ))
            })?;
        Ok(())
    }

    pub async fn create_constraints_and_indexes_by_spec<T>(
        &self,
        spec: &T,
    ) -> Result<(), CirroError>
    where
        T: SpecTrait,
    {
        let label_name = spec.get_label();
        self.create_constraints_by_name(&label_name).await?;
        Ok(())
    }

    /// Ensures all transactions are committed before proceeding
    async fn finalize_transactions(&self) -> Result<(), CirroError> {
        info!("Finalizing all transactions...");
        let mut final_txn = self.graph.start_txn().await.map_err(|e| {
            CirroError::DatabaseError(format!("Failed to start finalization transaction: {}", e))
        })?;
        final_txn.run(query("RETURN 1")).await.map_err(|e| {
            CirroError::DatabaseError(format!("Failed to execute finalization query: {}", e))
        })?; // Dummy query to ensure transaction context
        final_txn.commit().await.map_err(|e| {
            CirroError::DatabaseError(format!("Failed to commit finalization transaction: {}", e))
        })?;

        // Add a brief pause to ensure commit propagation
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    async fn generic_post_process(&mut self, dry_run: bool) -> Result<(), CirroError> {
        if dry_run {
            info!("Dry-run: post-processing queries that would run:");
            let mut post_processing_specs = self.specs.cirro_post_processing_specs.clone();
            post_processing_specs.sort_by_key(|spec| spec.priority);
            for spec in &post_processing_specs {
                info!(
                    "[dry-run] priority {} post-process spec: {}",
                    spec.priority, spec.name
                );
                debug!("[dry-run] cypher: {}", spec.cypher);
            }
            return Ok(());
        }

        info!("Waiting for all transactions to commit...");

        // Force a transaction commit by running a simple query that requires a read
        let _ = self
            .graph
            .execute(query("MATCH (n) RETURN count(n) LIMIT 1"))
            .await
            .map_err(|e| {
                CirroError::DatabaseError(format!("Failed to execute commit check query: {}", e))
            })?;

        // Add a small delay to ensure commit propagation
        tokio::time::sleep(Duration::from_millis(500)).await;

        info!("Running post-processing queries");

        // First, order the post-processing specs by priority
        let mut post_processing_specs = self.specs.cirro_post_processing_specs.clone();
        post_processing_specs.sort_by_key(|spec| spec.priority);

        for spec in &post_processing_specs {
            debug!("Running post-processing spec: {}", spec.name);

            self.graph.run(query(&spec.cypher)).await.map_err(|e| {
                CirroError::DatabaseError(format!(
                    "Failed to execute post-processing query for spec {}: {}",
                    spec.name, e
                ))
            })?;
        }

        Ok(())
    }
}
