use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;
use log::{debug, error, info};
use neo4rs::{BoltType, query};
use serde_json::Value;

impl CirroIngestor {
    /// Process enrichments
    pub async fn process_enrichment(
        &self,
        module_name: &str,
        node_insert_query: &str,
        properties: Vec<&str>,
    ) -> Result<(), CirroGraphError> {
        // Get the count of objects in the database
        let count_query = format!(
            "SELECT COUNT(*) FROM enrichments WHERE module  = '{}'",
            module_name.to_lowercase()
        );
        debug!("Executing sqlite count query: {}", count_query);
        let count = self
            .sql_conn
            .as_ref()
            .unwrap()
            .query_row(&count_query, [], |row| row.get::<_, i64>(0))
            .unwrap_or(0);

        if count == 0 {
            debug!("No {} found", module_name);
            return Ok(());
        }
        info!("Processing {:>5} {}", count, module_name);

        let limit = 5000;
        let mut offset = 0;

        loop {
            // If the offset exceeds the count, break the loop
            if offset * limit <= count {
                let select_query = format!(
                    "SELECT resource_id, data FROM enrichments WHERE module = '{}' LIMIT {} OFFSET {}",
                    module_name,
                    limit,
                    offset * limit
                );
                debug!("Executing query: {}", select_query);

                let mut stmt = self
                    .sql_conn
                    .as_ref()
                    .unwrap()
                    .prepare(&select_query)
                    .unwrap();

                // Execute the query and process each row
                let rows = stmt.query_map([], |row| {
                    let resource_id: String = row.get(0)?;
                    let data: String = row.get(1)?;
                    Ok((resource_id, data))
                })?;

                let processed_values: Vec<Value> = rows
                    .map(|result| {
                        result.map(|(resource_id, data): (String, String)| {
                            // Deserialize the JSON string into a Value
                            let value: Value = serde_json::from_str(&data)
                                .unwrap_or(Value::Object(serde_json::Map::new()));

                            // Create a new Value to hold the processed data
                            let mut new_value = Value::Object(serde_json::Map::new());

                            // For each property, set the value in the new Value
                            for property in &properties {
                                if let Some(val) = value.pointer(property) {
                                    new_value.as_object_mut().unwrap().insert(
                                        property.trim_start_matches('/').to_string(),
                                        val.clone(),
                                    );
                                }
                            }

                            // Add the resource group ID to the new Value
                            new_value.as_object_mut().unwrap().insert(
                                "resource_id".to_string(),
                                Value::String(resource_id.to_lowercase()),
                            );
                            new_value
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Insert the processed values into the graph
                if !processed_values.is_empty() {
                    debug!(
                        "Inserting {} {} from offset {}",
                        processed_values.len(),
                        module_name,
                        offset
                    );
                    let _ = self
                        .graph
                        .run(query(node_insert_query).param(
                            "batch",
                            BoltType::try_from(serde_json::to_value(&processed_values)?)?,
                        ))
                        .await
                        .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))?;
                } else {
                    error!(
                        "No values to insert for {} at offset {}",
                        module_name, offset
                    );
                }
                offset += 1;
                debug!("Processed {} {}s", &processed_values.len(), module_name);
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Process storage account keys enrichment
    pub async fn process_enrich_storage_keys(&self) -> Result<(), CirroGraphError> {
        let module_name = "storage_account_keys";
        let properties = vec!["/keys"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.resource_id})
            SET obj:StorageAccount
            
            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.keys, []) AS key
                    MERGE (k:StorageAccountKey {name: key.keyName})
                    SET k += {
                        value: key.value,
                        permissions: key.permissions,
                        creationTime: key.creationTime
                    }
                    MERGE (obj)-[:HAS_KEY]->(k)
                    RETURN count(*) AS _
                }
        "#;

        self.process_enrichment(module_name, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
