use crate::errors::CirroError;
use crate::graph::export::colors::{
    darken, expand_to_k_hops, generate_palette, greedy_color, text_color_for_bg,
};
use crate::graph::export::types::ExportFormat;
use log::info;
use neo4rs::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Exporter used by all Cirro export processes
pub struct CirroExporter {
    pub r#type: ExportFormat,
    pub output: PathBuf,
    pub host: String,
    pub user: String,
    pub password: String,
    pub db_name: String,
    pub graph: neo4rs::Graph,
}

/// Custom Debug trait for CirroExporter
impl std::fmt::Debug for CirroExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CirroExporter")
            .field("type", &self.r#type)
            .field("output", &self.output)
            .field("host", &self.host)
            .field("user", &self.user)
            .field("password", &self.password)
            .field("db_name", &self.db_name)
            .field("graph", &format_args!("<neo4rs::Graph>"))
            .finish()
    }
}

impl CirroExporter {
    pub async fn new(
        r#type: ExportFormat,
        output: PathBuf,
        host: String,
        user: String,
        password: String,
        db_name: Option<String>,
    ) -> Self {
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

        CirroExporter {
            r#type,
            output,
            host,
            user,
            password,
            db_name,
            graph,
        }
    }

    pub async fn run(&mut self) -> Result<(), CirroError> {
        match self.r#type {
            ExportFormat::Opengraph => {
                self.export_opengraph().await?;
            }
            ExportFormat::Grass => {
                self.export_grass().await?;
            }
        }

        Ok(())
    }

    pub async fn export_opengraph(&self) -> Result<(), CirroError> {
        // BloodHound OpenGraph format looks like this:
        // {
        //   "graph": {
        //     "nodes": [
        //       {
        //         "id": "123",
        //         "kinds": [
        //           "Person"
        //         ],
        //         "properties": {
        //           "displayname": "bob",
        //           "property": "a",
        //           "objectid": "123",
        //           "name": "BOB"
        //         }
        //       },
        //       {
        //         "id": "234",
        //         "kinds": [
        //           "Person"
        //         ],
        //         "properties": {
        //           "displayname": "alice",
        //           "property": "b",
        //           "objectid": "234",
        //           "name": "ALICE"
        //         }
        //       }
        //     ],
        //     "edges": [
        //       {
        //         "kind": "Knows",
        //         "start": {
        //           "value": "123",
        //           "match_by": "id"
        //         },
        //         "end": {
        //           "value": "234",
        //           "match_by": "id"
        //         }
        //       }
        //     ]
        //   }
        // }
        info!("Starting export in BloodHound OpenGraph format...");

        // First we need to get all nodes
        let mut nodes_result = self
            .graph
            .execute(query("MATCH (n) RETURN n"))
            .await
            .map_err(|e| CirroError::DatabaseError(e.to_string()))?;

        let mut nodes: Vec<serde_json::Value> = Vec::new();
        while let Ok(Some(row)) = nodes_result.next().await {
            let node: Node = row.get("n").unwrap();

            // Convert Node to JSON manually
            let mut node_map = serde_json::Map::new();
            node_map.insert(
                "id".to_string(),
                serde_json::Value::String(node.id().to_string()),
            );

            let labels: Vec<String> = node.labels().into_iter().map(|s| s.to_string()).collect();
            node_map.insert("kinds".to_string(), serde_json::json!(labels));

            let mut properties = serde_json::Map::new();
            // Keep track if "name" property exists
            let mut has_name = false;
            for key in node.keys() {
                if let Ok(value) = node.get::<serde_json::Value>(key) {
                    properties.insert(key.to_string(), value);
                    if key == "name" {
                        has_name = true;
                    }
                }
            }
            node_map.insert(
                "properties".to_string(),
                serde_json::Value::Object(properties),
            );

            // We need to make sure the "name" is set for nodes so it displays properly
            // If "name" doesn't exist in the map, then we use displayName or `id` from properties (not node id) as fallback
            // Id is guaranteed to exist but not all nodes have displayName or name
            if !has_name {
                let fallback_name = if let Some(display_name) = node_map
                    .get("properties")
                    .and_then(|p| p.as_object())
                    .unwrap()
                    .get("displayName")
                {
                    display_name.clone()
                } else {
                    serde_json::Value::String(
                        node_map
                            .get("properties")
                            .and_then(|p| p.as_object())
                            .unwrap()
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    )
                };

                if let Some(props) = node_map
                    .get_mut("properties")
                    .and_then(|p| p.as_object_mut())
                {
                    props.insert("name".to_string(), fallback_name);
                }
            }

            nodes.push(serde_json::Value::Object(node_map));
        }

        // Next, we need to get all edges
        let mut edges_result = self
            .graph
            .execute(query("MATCH ()-[r]->() RETURN r"))
            .await
            .map_err(|e| CirroError::DatabaseError(e.to_string()))?;

        let mut edges: Vec<serde_json::Value> = Vec::new();
        while let Ok(Some(row)) = edges_result.next().await {
            let rel: Relation = row.get("r").unwrap();

            // Convert Relation to JSON manually
            let mut edge_map = serde_json::Map::new();
            edge_map.insert(
                // OpenGraph doesn't allow hyphens in relationship names. Cirro does.
                "kind".to_string(),
                serde_json::Value::String(rel.typ().to_string().replace("-", "_")),
            );

            // Start node
            let mut start_map = serde_json::Map::new();
            start_map.insert(
                "value".to_string(),
                // Convert start_node_id to string
                serde_json::Value::String(rel.start_node_id().to_string()),
            );

            start_map.insert(
                "match_by".to_string(),
                serde_json::Value::String("id".to_string()),
            );
            edge_map.insert("start".to_string(), serde_json::Value::Object(start_map));

            // End node
            let mut end_map = serde_json::Map::new();
            end_map.insert(
                "value".to_string(),
                // Convert end_node_id to string
                serde_json::Value::String(rel.end_node_id().to_string()),
            );

            end_map.insert(
                "match_by".to_string(),
                serde_json::Value::String("id".to_string()),
            );
            edge_map.insert("end".to_string(), serde_json::Value::Object(end_map));

            // Add edge properties
            let mut properties = serde_json::Map::new();
            for key in rel.keys() {
                if let Ok(value) = rel.get::<serde_json::Value>(key) {
                    properties.insert(key.to_string(), value);
                }
            }
            edge_map.insert(
                "properties".to_string(),
                serde_json::Value::Object(properties),
            );

            edges.push(serde_json::Value::Object(edge_map));
        }

        // Store counts before moving the vectors
        let node_count = nodes.len();
        let edge_count = edges.len();

        // Construct final JSON
        let mut graph_map = serde_json::Map::new();
        graph_map.insert("nodes".to_string(), serde_json::Value::Array(nodes));
        graph_map.insert("edges".to_string(), serde_json::Value::Array(edges));
        let mut final_map = serde_json::Map::new();
        final_map.insert("graph".to_string(), serde_json::Value::Object(graph_map));
        let final_json = serde_json::Value::Object(final_map);

        // Write to output file with tokio
        let output_str = serde_json::to_string_pretty(&final_json).map_err(|e| {
            CirroError::SerializationError(format!("Failed to serialize OpenGraph JSON: {}", e))
        })?;
        tokio::fs::write(&self.output, output_str)
            .await
            .map_err(|e| {
                CirroError::ProcessingError(format!(
                    "Failed to write OpenGraph JSON to file {}: {}",
                    self.output.display(),
                    e
                ))
            })?;

        info!(
            "Successfully exported {} nodes and {} edges to {}",
            node_count,
            edge_count,
            self.output.display()
        );
        Ok(())
    }

    pub async fn export_grass(&self) -> Result<(), CirroError> {
        info!("Starting export in Neo4j GraSS stylesheet format...");

        // Query the database for label-relationship structure
        let mut result = self
            .graph
            .execute(query(
                "MATCH (n)-[r]->(m) \
                 RETURN DISTINCT labels(n) AS sourceLabels, type(r) AS relType, labels(m) AS targetLabels",
            ))
            .await
            .map_err(|e| CirroError::DatabaseError(e.to_string()))?;

        // Build adjacency map: label -> set of directly connected labels
        let mut all_labels: HashSet<String> = HashSet::new();
        let mut adj: HashMap<String, HashSet<String>> = HashMap::new();

        while let Ok(Some(row)) = result.next().await {
            let source_labels: Vec<String> = row.get("sourceLabels").unwrap_or_default();
            let target_labels: Vec<String> = row.get("targetLabels").unwrap_or_default();

            for src in &source_labels {
                all_labels.insert(src.clone());
                for tgt in &target_labels {
                    all_labels.insert(tgt.clone());
                    if src != tgt {
                        adj.entry(src.clone()).or_default().insert(tgt.clone());
                        adj.entry(tgt.clone()).or_default().insert(src.clone());
                    }
                }
            }
        }

        if all_labels.is_empty() {
            info!("No labels found in database, writing empty stylesheet");
            let output_path = self.output.with_extension("grass");
            tokio::fs::write(&output_path, "").await.map_err(|e| {
                CirroError::ProcessingError(format!(
                    "Failed to write GraSS file to {}: {}",
                    output_path.display(),
                    e
                ))
            })?;
            return Ok(());
        }

        let labels: Vec<String> = all_labels.into_iter().collect();

        // Expand adjacency to 3 hops
        let expanded_adj = expand_to_k_hops(&labels, &adj, 3);

        // Greedy graph coloring
        let coloring = greedy_color(&labels, &expanded_adj);

        // Generate palette
        let num_colors = coloring.values().max().map_or(0, |m| m + 1);
        let palette = generate_palette(num_colors);

        // Base labels that should appear last in GraSS output
        let base_labels: HashSet<&str> = ["GraphObject", "ArmResource"].into_iter().collect();

        // Sort labels: specific labels first (alphabetical), base labels last
        let mut sorted_labels: Vec<&String> = labels.iter().collect();
        sorted_labels.sort_by(|a, b| {
            let a_is_base = base_labels.contains(a.as_str());
            let b_is_base = base_labels.contains(b.as_str());
            match (a_is_base, b_is_base) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.cmp(b),
            }
        });

        // Build GraSS stylesheet
        let mut grass = String::new();
        for label in &sorted_labels {
            let color_idx = coloring[label.as_str()];
            let color = &palette[color_idx];
            let border = darken(color, 0.8);
            let text = text_color_for_bg(color);
            let caption = if label.starts_with("Graph") {
                "{displayName}"
            } else {
                "{name}"
            };
            grass.push_str(&format!(
                "node.{} {{\n  color: {};\n  border-color: {};\n  text-color-internal: {};\n  caption: '{}';\n}}\n",
                label, color, border, text, caption
            ));
        }

        let output_path = self.output.with_extension("grass");
        tokio::fs::write(&output_path, &grass).await.map_err(|e| {
            CirroError::ProcessingError(format!(
                "Failed to write GraSS file to {}: {}",
                output_path.display(),
                e
            ))
        })?;

        info!(
            "Successfully exported GraSS stylesheet with {} labels to {}",
            sorted_labels.len(),
            output_path.display()
        );

        Ok(())
    }
}
