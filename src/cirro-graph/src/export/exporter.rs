use crate::errors::CirroGraphError;
use crate::ingest::constants::*;

use log::{debug, info};
use neo4rs::*;
use serde_json::{Value, json};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExportType {
    Json,
    Opengraph,
}

pub struct CirroExporter {
    export_type: ExportType,
    output_file: PathBuf,
    graph: neo4rs::Graph,
    graph_type: GraphType,
    host: String,
    user: String,
    db_name: String,
}

/// Custom Debug trait for CirroIngestor
impl std::fmt::Debug for CirroExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CirroExporter")
            .field("export_type", &self.export_type)
            .field("output_file", &self.output_file)
            .field("graph_type", &self.graph_type)
            .field("host", &self.host)
            .field("user", &self.user)
            .field("db_name", &self.db_name)
            .finish()
    }
}

impl CirroExporter {
    pub async fn new(
        export_type: ExportType,
        output_file: PathBuf,
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

        let ingestor = CirroExporter {
            export_type,
            output_file,
            graph,
            graph_type,
            host,
            user,
            db_name,
        };
        return ingestor;
    }

    /// Runs the ingestor
    pub async fn run(&mut self) -> Result<(), CirroGraphError> {
        let graph_data = match self.graph_type {
            GraphType::Neo4j => self.get_neo4j_graph().await?,
            GraphType::Memgraph => self.get_memgraph_graph().await?,
        };

        match self.export_type {
            ExportType::Json => {
                info!("Starting JSON export to {}", self.output_file.display());

                // Write JSON to file with ident of 2
                let mut file = File::create(&self.output_file)?;
                file.write_all(serde_json::to_string_pretty(&graph_data)?.as_bytes())?;
            }
            ExportType::Opengraph => {
                info!(
                    "Starting OpenGraph export to {}",
                    self.output_file.display()
                );
                self.export_opengraph(graph_data).await?;
                info!("OpenGraph export completed successfully");
            }
        }
        Ok(())
    }

    /// Export data to OpenGraph format
    async fn export_opengraph(
        &mut self,
        graph_data: serde_json::Value,
    ) -> Result<(), CirroGraphError> {
        // Implement OpenGraph export logic here

        // Opengraph schema is:
        // {
        //   "graph": {
        //     "nodes": [],
        //     "edges": []
        //   }
        // }

        // Need to iterate over nodes and edges and add them to the OpenGraph schema
        let mut opengraph = json!({
            "graph": {
                "nodes": [],
                "edges": []

            }
        });
        let mut nodes: Vec<Value> = Vec::new();
        let mut edges: Vec<Value> = Vec::new();

        match self.graph_type {
            GraphType::Neo4j => {}
            GraphType::Memgraph => {
                // Need to iterate over nodes and edges and add them to the OpenGraph schema

                for object in graph_data.as_array().unwrap() {
                    let object_map = object.as_object().unwrap();

                    let object_type = object_map.get("type").unwrap().as_str().unwrap();
                    debug!("Processing object of type: {}", object_type);
                    debug!(
                        "Object keys: {}",
                        object_map
                            .keys()
                            .map(|k| k.as_str())
                            .collect::<Vec<&str>>()
                            .join(", ")
                    );
                    match object_type {
                        "node" => {
                            // If there's no id, this node is not valid
                            let id = match object_map.get("id").unwrap().as_number() {
                                Some(id) => id.to_string(),
                                None => {
                                    debug!("Node without id found: {}", object);
                                    continue;
                                }
                            };

                            // If there are no labels, this node is not valid. Need to figure out why this happens but should be fine for export
                            let labels = match object_map.get("labels").unwrap().as_array() {
                                Some(labels) => labels,
                                None => {
                                    debug!("Node without labels found: {}", object);
                                    continue;
                                }
                            };
                            let properties =
                                object_map.get("properties").unwrap().as_object().unwrap();

                            // If properties.name doesn't exist, check for properties.displayName and copy it
                            // This is mostly for GraphObjects and other random nodes in the graph
                            // OpenGraph uses the "name" property to display the node while using the node "id" as a backup
                            let mut resolved_properties = properties.clone();
                            if !resolved_properties.contains_key("name") {
                                if let Some(display_name) = resolved_properties.get("displayName") {
                                    resolved_properties.insert("name".into(), display_name.clone());
                                }
                            }

                            let node = json!({
                                "id": id,
                                "kinds": labels,
                                "properties": resolved_properties
                            });
                            nodes.push(node);
                        }
                        "relationship" => {
                            // All of the following fields are required for a relationship, except for properties
                            let start = match object_map.get("start").unwrap().as_number() {
                                Some(start) => start.to_string(),
                                None => {
                                    debug!("Relationship without start found: {}", object);
                                    continue;
                                }
                            };
                            let end = match object_map.get("end").unwrap().as_number() {
                                Some(end) => end.to_string(),
                                None => {
                                    debug!("Relationship without end found: {}", object);
                                    continue;
                                }
                            };
                            let label = match object_map.get("label").unwrap().as_str() {
                                Some(label) => label,
                                None => {
                                    debug!("Relationship without label found: {}", object);
                                    continue;
                                }
                            };
                            let properties =
                                object_map.get("properties").unwrap().as_object().unwrap();

                            let relationship = json!({
                                "kind": label,
                                "start": { "value": start, "match_by": "id" },
                                "end": { "value": end, "match_by": "id" },
                                "properties": properties
                            });
                            edges.push(relationship);
                        }
                        _ => {
                            info!("Unknown object type found: {}", object_type);
                            continue;
                        }
                    }
                }
            }
        }

        // Add nodes and edges to the OpenGraph schema
        opengraph["graph"]["nodes"] = Value::Array(nodes);
        opengraph["graph"]["edges"] = Value::Array(edges);

        // Write the OpenGraph schema to the output file
        let mut file = File::create(&self.output_file)?;
        file.write_all(serde_json::to_string_pretty(&opengraph)?.as_bytes())?;

        Ok(())
    }

    /// Get all nodes and edges from neo4j
    async fn get_neo4j_graph(&mut self) -> Result<serde_json::Value, CirroGraphError> {
        Ok(serde_json::from_str("<neo4j_graph_data>").unwrap())
    }

    /// Get all nodes and edges from memgraph
    async fn get_memgraph_graph(&mut self) -> Result<serde_json::Value, CirroGraphError> {
        let mut result = self
            .graph
            .execute(query(
                "CALL export_util.json('', {stream: 'true', write_properties:'true'}) YIELD data",
            ))
            .await?;

        let mut json_data = String::new();
        while let Ok(Some(row)) = result.next().await {
            let data: String = row.get("data").unwrap();
            json_data.push_str(&data);
        }

        Ok(serde_json::from_str(&json_data).unwrap())
    }
}
