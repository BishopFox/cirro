use crate::ingest::ingestor::CirroIngestor;
use crate::{errors::CirroGraphError, specs::CirroAzureIngestSpec};
use log::{debug, error, info};
use neo4rs::{BoltType, query};
use serde_json::Value;

impl CirroIngestor {
    /// Process ingest
    pub async fn process_cirro_azure_ingest(&self) -> Result<(), CirroGraphError> {
        info!(
            "Starting Cirro Azure ingest on file: {:?}",
            self.file.as_path().file_name().unwrap()
        );

        // List all the specs for Cirro Azure
        for spec in &self.specs.cirro_azure_specs {
            self.process_spec(spec).await?;
        }

        // Ensure all Azure ingestion transactions are committed
        debug!("Finalizing Azure ingestion transactions...");
        let mut final_txn = self.graph.start_txn().await.map_err(|e| {
            CirroGraphError::DatabaseError(format!(
                "Failed to start Azure finalization transaction: {}",
                e
            ))
        })?;
        final_txn
            .run(neo4rs::query("RETURN 1"))
            .await
            .map_err(|e| {
                CirroGraphError::DatabaseError(format!(
                    "Failed to execute Azure finalization query: {}",
                    e
                ))
            })?;
        final_txn.commit().await.map_err(|e| {
            CirroGraphError::DatabaseError(format!(
                "Failed to commit Azure finalization transaction: {}",
                e
            ))
        })?;

        Ok(())
    }

    /// Process Arm resources from the ingestor
    async fn process_spec(&self, spec: &CirroAzureIngestSpec) -> Result<(), CirroGraphError> {
        // First we need to determine how to query the data from SQLite
        let table_name = &spec.table_name;
        let resource_type = &spec.resource_type;

        // Build the SELECT clause based on column_mappings
        let select_clause = if let Some(mappings) = &spec.column_mappings {
            let mapped_columns: Vec<String> = mappings.keys().map(|v| v.to_string()).collect();
            format!("{}, data", mapped_columns.join(", "))
        } else {
            "data".to_string()
        };

        // Create count and data queries based on whether resource_type is specified
        let (count_query, data_query_template) = match resource_type {
            Some(rt) => {
                // Resource type might start with a '!' for negation
                // Right now, only resource groups use this feature but it could be expanded or removed later
                // If it starts with '!', we query all resources NOT of that type
                if rt.starts_with('!') {
                    let condition = format!(
                        "WHERE lower(resource_type) != '{}'",
                        rt.trim_start_matches('!').to_lowercase()
                    );
                    (
                        format!("SELECT COUNT(*) FROM {} {}", table_name, condition),
                        format!(
                            "SELECT {} FROM {} {} LIMIT {{}} OFFSET {{}}",
                            select_clause, table_name, condition
                        ),
                    )
                } else {
                    let condition = format!("WHERE lower(resource_type) = '{}'", rt.to_lowercase());
                    (
                        format!("SELECT COUNT(*) FROM {} {}", table_name, condition),
                        format!(
                            "SELECT {} FROM {} {} LIMIT {{}} OFFSET {{}}",
                            select_clause, table_name, condition
                        ),
                    )
                }
            }
            None => (
                format!("SELECT COUNT(*) FROM {}", table_name),
                format!(
                    "SELECT {} FROM {} LIMIT {{}} OFFSET {{}}",
                    select_clause, table_name
                ),
            ),
        };

        // Get the count of objects in the database
        let count = self
            .sql_conn
            .as_ref()
            .unwrap()
            .query_row(count_query.as_str(), [], |row: &rusqlite::Row| {
                row.get::<_, i64>(0)
            })
            .unwrap_or(0);

        if count == 0 {
            debug!("No {} found", spec.name);
            return Ok(());
        }
        info!("Processing {:>5} {}", count, spec.name);

        // If spec has a label, create constraints and indexes
        if !spec.label.is_empty() {
            self.create_constraints_and_indexes(spec).await?;
        } else {
            debug!(
                "Spec {} does not have a label defined, skipping constraint and index creation",
                spec.name
            );
        }

        // Now we can process the data in batches
        let limit = 5000;
        let mut offset = 0;

        loop {
            // If the offset exceeds the count, break the loop
            if offset * limit <= count {
                let select_query = data_query_template
                    .replacen("{}", &limit.to_string(), 1)
                    .replacen("{}", &(offset * limit).to_string(), 1);
                debug!("Executing query: {}", select_query);

                let mut stmt = self
                    .sql_conn
                    .as_ref()
                    .unwrap()
                    .prepare(&select_query)
                    .unwrap();

                // Execute the query and process each row
                let rows = stmt.query_map([], |row| {
                    let mut mapped_values = std::collections::HashMap::new();

                    // Extract mapped column values if they exist
                    if let Some(mappings) = &spec.column_mappings {
                        for (i, cypher_param) in mappings.values().enumerate() {
                            let value: String = row.get(i)?;
                            mapped_values.insert(cypher_param.clone(), value);
                        }
                        // Get the data column (always last when mappings exist)
                        let data: String = row.get(mapped_values.len())?;
                        Ok((mapped_values, data))
                    } else {
                        // No mappings, just get the data column
                        let data: String = row.get(0)?;
                        Ok((mapped_values, data))
                    }
                })?;

                let processed_values: Vec<Value> = rows
                    .map(|result| {
                        result.map(
                            |(mapped_values, data): (
                                std::collections::HashMap<String, String>,
                                String,
                            )| {
                                // Deserialize the JSON string into a Value
                                let value: Value = serde_json::from_str(&data)
                                    .unwrap_or(Value::Object(serde_json::Map::new()));

                                // Create a new Value to hold the processed data
                                let mut new_value = Value::Object(serde_json::Map::new());

                                // Add mapped column values first
                                for (key, val) in mapped_values {
                                    new_value
                                        .as_object_mut()
                                        .unwrap()
                                        .insert(key, Value::String(val));
                                }

                                // For each property, set the value in the new Value
                                for property in &spec.properties {
                                    if let Some(val) = value.pointer(property) {
                                        if *property == "/id" {
                                            // Make sure the ID is always lowercase
                                            new_value.as_object_mut().unwrap().insert(
                                                property.trim_start_matches('/').to_string(),
                                                val.as_str().unwrap_or("").to_lowercase().into(),
                                            );
                                        } else {
                                            new_value.as_object_mut().unwrap().insert(
                                                property.trim_start_matches('/').to_string(),
                                                val.clone(),
                                            );
                                        }
                                    }
                                }
                                new_value
                            },
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Insert the processed values into the graph
                if !processed_values.is_empty() {
                    debug!(
                        "Inserting {} {} from offset {}",
                        processed_values.len(),
                        table_name,
                        offset
                    );
                    let _ = self
                        .graph
                        .run(query(&spec.cypher).param(
                            "batch",
                            BoltType::try_from(serde_json::to_value(&processed_values)?)?,
                        ))
                        .await
                        .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))?;
                } else {
                    error!(
                        "No values to insert for {} at offset {}",
                        table_name, offset
                    );
                }
                offset += 1;
                debug!("Processed {} {}s", &processed_values.len(), table_name);
            } else {
                break;
            }
        }
        Ok(())
    }
}
