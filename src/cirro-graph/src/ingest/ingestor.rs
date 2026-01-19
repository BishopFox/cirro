use crate::errors::CirroGraphError;
use crate::specs::{SpecLoader, SpecRegistry, SpecTrait};

use log::{debug, info};
use neo4rs::*;
use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct CirroIngestor {
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
        self.process_cirro_azure_ingest().await?;

        info!("Running final post-processing merge query");
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
        debug!("Running post-processing merge query");

        // Merge all nodes with the same ID - Neo4j only
        debug!("Merging nodes with the same ID in Neo4j");
        let _ = self
            .graph
            .run(query(
                r#"CALL apoc.periodic.iterate(
                    "
                    MATCH (n)
                    WHERE n.id IS NOT NULL
                    WITH toLower(n.id) AS lid, collect(n) AS ns
                    WHERE size(ns) > 1
                    RETURN lid, ns
                    ",
                    "
                    WITH lid, ns
                    WITH
                        head(ns) AS keep,
                        tail(ns) AS dups,
                        lid
                    UNWIND dups AS d
                    CALL apoc.refactor.mergeNodes(
                        [keep, d],
                        {properties:'combine', mergeRels:true}
                    )
                    YIELD node
                    // Ensure ids are lowercase
                    SET keep.id = lid
                    RETURN 1
                    ",
                    {batchSize: 200, parallel: false}
                    );"#,
            ))
            .await
            .map_err(|e| {
                CirroGraphError::DatabaseError(format!(
                    "Failed to run post-processing merge query: {}",
                    e
                ))
            })?;
        // Merge relationships with the same label from the same node
        debug!("Merging relationships with the same label in Neo4j");
        let _ = self
            .graph
            .run(query(
                r#"CALL apoc.periodic.iterate(
                    "
                    MATCH (a)-[r]->(b)
                    WITH a, b, type(r) AS relType, collect(r) AS rels
                    WHERE size(rels) > 1
                    RETURN rels
                    ",
                    "
                    WITH rels
                    CALL apoc.refactor.mergeRelationships(rels, {properties: 'overwrite'})
                    YIELD rel
                    RETURN 1
                    ",
                    {batchSize: 200, parallel: false}
                );"#,
            ))
            .await
            .map_err(|e| {
                CirroGraphError::DatabaseError(format!(
                    "Failed to run post-processing merge relationships query: {}",
                    e
                ))
            })?;
        Ok(())
    }
}
