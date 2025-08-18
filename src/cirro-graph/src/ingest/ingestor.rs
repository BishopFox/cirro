use crate::errors::CirroGraphError;
use crate::ingest::constants::*;

use log::{debug, info};
use neo4rs::*;
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::pin::Pin;
use strum::IntoEnumIterator;

pub struct CirroIngestor {
    pub file: PathBuf,
    pub host: String,
    pub user: String,
    pub password: String,
    pub db_name: String,
    pub graph: neo4rs::Graph,
    pub graph_type: GraphType,
    pub constants: CypherConstants,
    pub sql_conn: Option<Connection>,
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
            .finish()
    }
}

impl CirroIngestor {
    pub async fn new(
        file: PathBuf,
        graph_type: GraphType,
        host: String,
        user: String,
        password: String,
        db_name: Option<String>,
    ) -> Self {
        // If db_name is not provided, use the default based on graph type
        let db_name = db_name.unwrap_or_else(|| match graph_type {
            GraphType::Neo4j => "neo4j".to_string(),
            GraphType::Memgraph => "memgraph".to_string(),
        });

        let config = ConfigBuilder::default()
            .uri(host.clone())
            .user(user.clone())
            .password(password.clone())
            .db(db_name.clone())
            .build()
            .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))
            .unwrap();

        info!(
            "Connecting to {:?} database at {} with user {}",
            graph_type, host, user
        );
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
            graph_type,
            constants: CypherConstants::new(graph_type),
            sql_conn,
        };
        return ingestor;
    }

    /// Runs the ingestor
    pub async fn run(&mut self) -> Result<(), CirroGraphError> {
        // Create constraints and indexes for each node type
        for node_type in NodeType::iter() {
            let constraint_query = self
                .constants
                .create_constraint_query
                .replace("{}", &node_type.to_string());

            debug!("Executing query: {}", constraint_query);
            let _ = self
                .graph
                .run(query(&constraint_query))
                .await
                .map_err(|e| {
                    CirroGraphError::DatabaseError(format!(
                        "Failed to create constraint for {}: {}",
                        node_type, e
                    ))
                })?;

            if !self.constants.create_index_query.is_empty() {
                let index_query = self
                    .constants
                    .create_index_query
                    .replace("{}", &node_type.to_string());

                debug!("Executing query: {}", index_query);
                let _ = self.graph.run(query(&index_query)).await.map_err(|e| {
                    CirroGraphError::DatabaseError(format!(
                        "Failed to create index for {}: {}",
                        node_type, e
                    ))
                })?;
            }
        }
        // Start stopwatch to measure the time taken for the entire ingestion process
        let start_time = std::time::Instant::now();
        debug!("Ingestion started at: {:?}", start_time);

        if self.graph_type == GraphType::Memgraph {
            debug!("Setting storage mode to IN_MEMORY_ANALYTICAL for Memgraph");
            let _ = self
                .graph
                .run(query("STORAGE MODE IN_MEMORY_ANALYTICAL;"))
                .await
                .map_err(|e| {
                    CirroGraphError::DatabaseError(format!(
                        "Failed to set storage mode for Memgraph: {}",
                        e
                    ))
                })?;
        }

        // Run the main ingestion logic
        self.process_file().await?;

        info!("Running final post-processing merge query");
        self.post_process().await?;

        // Calculate the total time taken for the ingestion process
        let duration = start_time.elapsed();
        info!("Cirro ingestion process completed in: {:.2?}", duration);

        Ok(())
    }

    async fn process_file(&mut self) -> Result<(), CirroGraphError> {
        info!(
            "Starting Cirro ingest on file: {:?}",
            self.file.as_path().file_name().unwrap()
        );

        // Define type that matches the actual function signature. Thanks Copilot cause this is harder than Python.
        #[rustfmt::skip]
        type ProcessorFn = for<'a> fn(&'a CirroIngestor,) -> Pin<Box<dyn Future<Output = Result<(), CirroGraphError>> + 'a>,>;

        let processors: Vec<ProcessorFn> = vec![
            |ingestor| Box::pin(CirroIngestor::process_graph_users(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_graph_devices(ingestor)),
            // Applications must be processed before service principals due to OPTIONAL MATCH call in service principals
            |ingestor| Box::pin(CirroIngestor::process_graph_applications(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_graph_service_principals(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_graph_groups(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_graph_roles(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_graph_organizations(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_graph_policies(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_tenants(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_mg_entities(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_subscriptions(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_resource_groups(ingestor)),
            // This must always be first to ensure all resources are entered regardless of type
            |ingestor| Box::pin(CirroIngestor::process_generic_arm_resources(ingestor)),
            // Process specific ARM resources
            |ingestor| Box::pin(CirroIngestor::process_automation_accounts(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_automation_runbooks(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_availability_sets(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_azurearc_sql_servers(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_bastion_hosts(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_classic_storage_accounts(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_cognitive_services_account(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_communication_services(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_container_registries(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_datafactories(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_disks(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_dns_zones(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_event_grid_system_topics(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_event_grid_topics(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_galleries(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_gallery_applications(ingestor)),
            |ingestor| {
                Box::pin(CirroIngestor::process_gallery_application_versions(
                    ingestor,
                ))
            },
            |ingestor| Box::pin(CirroIngestor::process_hybrid_machines(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_keyvaults(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_network_interfaces(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_network_security_groups(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_private_dns_zones(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_private_endpoints(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_public_ip_addresses(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_recovery_services_vaults(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_restore_point_collections(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_route_tables(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_server_farms(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_sites(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_snapshots(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_sql_servers(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_sql_databases(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_sql_virtual_machines(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_ssh_public_keys(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_storage_accounts(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_user_assigned_identities(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_vm_applications(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_virtual_machines(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_virtual_machine_extensions(ingestor)),
            |ingestor| Box::pin(CirroIngestor::process_virtual_networks(ingestor)),
            // This must always be last to ensure all other nodes are processed first
            |ingestor| Box::pin(CirroIngestor::process_role_assignments(ingestor)),
            // Enrichment processors
            |ingestor| Box::pin(CirroIngestor::process_enrich_storage_keys(ingestor)),
        ];

        for processor in processors {
            debug!("Running processor: {:?}", processor);
            let result = processor(self).await;
            if let Err(e) = result {
                return Err(CirroGraphError::ProcessingError(format!(
                    "Failed to process file with error: {}",
                    e
                )));
            }
            debug!("Processor completed successfully");
            self.post_process().await?;
        }

        Ok(())
    }

    async fn post_process(&mut self) -> Result<(), CirroGraphError> {
        debug!("Running post-processing merge query");

        match self.graph_type {
            GraphType::Neo4j => {
                // Merge all nodes with the same ID
                debug!("Merging nodes with the same ID in Neo4j");
                let _ = self
                    .graph
                    .run(query(
                        r#" MATCH (n)
                        WITH toLower(n.id) AS id, COLLECT(n) AS nodesToMerge
                        WHERE id IS NOT NULL AND size(nodesToMerge) > 1
                        CALL apoc.refactor.mergeNodes(nodesToMerge, {properties: "override", mergeRels:true, preserveExistingSelfRels: true})
                        YIELD node
                        SET node.id = id
                        SET node.type = toLower(node.type)
                        RETURN count(node);"#,
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
                        r#"MATCH (a)-[r]->(b)
                        WITH a, b, type(r) AS relType, collect(r) AS rels
                        WHERE size(rels) > 1
                        CALL apoc.refactor.mergeRelationships(rels, {properties: "combine"})
                        YIELD rel
                        RETURN count(rel);"#,
                    ))
                    .await
                    .map_err(|e| {
                        CirroGraphError::DatabaseError(format!(
                            "Failed to run post-processing merge relationships query: {}",
                            e
                        ))
                    })?;
            }
            GraphType::Memgraph => {
                // Merge all nodes with the same ID
                debug!("Merging nodes with the same ID in Memgraph");
                let _ = self
                    .graph
                    .run(query(
                        r#"MATCH (n)
                WHERE n.id IS NOT NULL
                SET n.id = toLower(n.id)
                WITH n
                WHERE n.type IS NOT NULL
                SET n.type = toLower(n.type)"#,
                    ))
                    .await
                    .map_err(|e| {
                        CirroGraphError::DatabaseError(format!(
                            "Failed to assert lowercase IDs: {}",
                            e
                        ))
                    })?;
                let _ = self
                    .graph
                    .run(query(
                        r#"MATCH (n)
                        WITH toLower(n.id) AS id, COLLECT(n) AS nodesToMerge
                        WHERE id IS NOT NULL AND size(nodesToMerge) > 1
                        CALL refactor.merge_nodes(nodesToMerge, {properties: "override", mergeRels:true})
                        YIELD node
                        RETURN count(*);"#,
                    ))
                    .await
                    .map_err(|e| {
                        CirroGraphError::DatabaseError(format!(
                            "Failed to run post-processing merge query: {}",
                            e
                        ))
                    })?;
                // Merge relationships with the same label from the same node
                debug!("Merging relationships with the same label in Memgraph");
                let _ = self
                    .graph
                    .run(query(
                        r#"MATCH (a)-[r]->(b)
                        WHERE id(a) <> id(b)  // ❗ skip self-loops
                        WITH a, b, type(r) AS rel_type, collect(r) AS rels
                        WHERE size(rels) > 1
                        CALL {
                        WITH rels
                        WITH rels[0] AS keeper, rels[1..] AS duplicates
                        UNWIND duplicates AS d
                            SET keeper += d
                            DELETE d
                        RETURN keeper
                        }"#,
                    ))
                    .await
                    .map_err(|e| {
                        CirroGraphError::DatabaseError(format!(
                            "Failed to run post-processing merge relationships query: {}",
                            e
                        ))
                    })?;
            }
        }
        Ok(())
    }
}
