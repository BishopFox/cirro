use crate::errors::CirroError;
use crate::graph::ingest::CREATE_CONSTRAINT_QUERY;
use crate::graph::ingest::ingestor::CirroIngestor;
use crate::graph::specs::CirroAzureIngestSpec;
use log::{debug, info, warn};
use neo4rs::{BoltType, query};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

impl CirroIngestor {
    /// Process ingest
    pub async fn process_cirro_azure_ingest(&self, dry_run: bool) -> Result<(), CirroError> {
        info!(
            "Starting Cirro Azure ingest on file: {:?}",
            self.file.as_path().file_name().unwrap()
        );

        // Create constraints for base labels upfront
        if dry_run {
            info!("[dry-run] Would create default constraints and indexes");
        } else {
            create_default_constraints_and_indexes(&self.graph).await?;
        }

        // Group spec indices by priority for concurrent execution
        let mut priority_groups: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
        for (i, spec) in self.specs.cirro_azure_specs.iter().enumerate() {
            priority_groups
                .entry(spec.priority.unwrap_or(0))
                .or_default()
                .push(i);
        }

        // Process each priority group sequentially
        for (priority, indices) in &priority_groups {
            info!(
                "Processing {} specs at priority {}",
                indices.len(),
                priority
            );

            for &i in indices {
                let spec = &self.specs.cirro_azure_specs[i];
                process_spec(&self.file, &self.graph, spec, dry_run).await?;
            }
        }

        if dry_run {
            info!("[dry-run] Skipping Azure ingestion transaction finalization");
            return Ok(());
        }

        // Ensure all Azure ingestion transactions are committed
        debug!("Finalizing Azure ingestion transactions...");
        let mut final_txn = self.graph.start_txn().await.map_err(|e| {
            CirroError::DatabaseError(format!(
                "Failed to start Azure finalization transaction: {}",
                e
            ))
        })?;
        final_txn
            .run(neo4rs::query("RETURN 1"))
            .await
            .map_err(|e| {
                CirroError::DatabaseError(format!(
                    "Failed to execute Azure finalization query: {}",
                    e
                ))
            })?;
        final_txn.commit().await.map_err(|e| {
            CirroError::DatabaseError(format!(
                "Failed to commit Azure finalization transaction: {}",
                e
            ))
        })?;

        Ok(())
    }

    pub async fn show_unimplemented_azure_resource_types(&self) -> Result<(), CirroError> {
        let all_resource_types: HashSet<String> = self
            .specs
            .cirro_azure_specs
            .iter()
            .filter_map(|spec| spec.resource_type.clone())
            .map(|resource_type| resource_type.trim_start_matches('!').to_lowercase())
            .collect();

        let mut stmt = self
            .sql_conn
            .as_ref()
            .ok_or_else(|| {
                CirroError::DatabaseError(
                    "SQLite connection unavailable for dry-run resource type stats".to_string(),
                )
            })?
            .prepare("SELECT DISTINCT lower(resource_type) FROM resources")
            .map_err(|e| {
                CirroError::DatabaseError(format!(
                    "Failed to prepare SQL statement for fetching resource types: {}",
                    e
                ))
            })?;

        let sql_resource_types_iter =
            stmt.query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| {
                    CirroError::DatabaseError(format!(
                        "Failed to execute SQL query for fetching resource types: {}",
                        e
                    ))
                })?;

        let sql_resource_types: HashSet<String> =
            sql_resource_types_iter.filter_map(|res| res.ok()).collect();

        let mut unimplemented_resource_types: Vec<String> = sql_resource_types
            .difference(&all_resource_types)
            .cloned()
            .collect();
        unimplemented_resource_types.sort();

        info!(
            "[dry-run] Azure resource type coverage: {} in input, {} implemented by specs, {} missing",
            sql_resource_types.len(),
            all_resource_types.len(),
            unimplemented_resource_types.len()
        );

        if unimplemented_resource_types.is_empty() {
            info!("[dry-run] All input resource types are covered by implemented specs");
        } else {
            info!(
                "[dry-run] Resource types without implemented specs: {:?}",
                unimplemented_resource_types
            );
        }

        Ok(())
    }
}

/// Process a single spec with its own SQLite connection and pipelined Neo4j writes
async fn process_spec(
    sql_file: &PathBuf,
    graph: &neo4rs::Graph,
    spec: &CirroAzureIngestSpec,
    dry_run: bool,
) -> Result<(), CirroError> {
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
    let condition = match resource_type {
        Some(rt) => {
            if rt.starts_with('!') {
                format!(
                    "WHERE lower(resource_type) != '{}'",
                    rt.trim_start_matches('!').to_lowercase()
                )
            } else {
                format!("WHERE lower(resource_type) = '{}'", rt.to_lowercase())
            }
        }
        None => String::new(),
    };

    let count_query = if condition.is_empty() {
        format!("SELECT COUNT(*) FROM {}", table_name)
    } else {
        format!("SELECT COUNT(*) FROM {} {}", table_name, condition)
    };

    // Use rowid cursor for efficient pagination
    let data_query_template = if condition.is_empty() {
        format!(
            "SELECT rowid, {} FROM {} WHERE rowid > {{}} ORDER BY rowid LIMIT {{}}",
            select_clause, table_name
        )
    } else {
        format!(
            "SELECT rowid, {} FROM {} {} AND rowid > {{}} ORDER BY rowid LIMIT {{}}",
            select_clause, table_name, condition
        )
    };

    // Open a read-only SQLite connection for this task
    let sql_conn = rusqlite::Connection::open_with_flags(
        &sql_file,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| CirroError::DatabaseError(format!("Failed to open SQLite connection: {}", e)))?;

    // Get the count of objects in the database
    let count = sql_conn
        .query_row(count_query.as_str(), [], |row: &rusqlite::Row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or(0);

    if count == 0 {
        debug!("No {} found", spec.name);
        return Ok(());
    }
    if dry_run {
        info!("[dry-run] Would process {:>5} {}", count, spec.name);
    } else {
        info!("Processing {:>5} {}", count, spec.name);
    }

    // Create constraints for this spec's label only when data exists
    if !spec.label.is_empty() && !dry_run {
        create_constraint(graph, &spec.label, "id").await?;
    } else if !spec.label.is_empty() {
        debug!(
            "[dry-run] Would create constraint for label {} on id",
            spec.label
        );
    }

    // Create additional constraints from constraint_properties.
    // Supports single-property entries (Label:property) and composite entries
    // (Label:prop1+prop2).
    if let Some(constraint_props) = &spec.constraint_properties {
        for entry in constraint_props {
            if let Some((label, properties)) = parse_label_properties(entry) {
                if dry_run {
                    let property_list = properties.join(", ");
                    debug!(
                        "[dry-run] Would create constraint for label {} on ({})",
                        label, property_list
                    );
                } else {
                    create_constraint_for_properties(graph, label, &properties).await?;
                }
            } else {
                warn!(
                    "Skipping invalid constraint_properties entry '{}' in spec '{}'",
                    entry, spec.name
                );
            }
        }
    }

    // Create additional indexes from index_properties.
    // Supports single-property entries (Label:property) and composite entries
    // (Label:prop1+prop2).
    if let Some(index_props) = &spec.index_properties {
        for entry in index_props {
            if let Some((label, properties)) = parse_label_properties(entry) {
                if dry_run {
                    let property_list = properties.join(", ");
                    debug!(
                        "[dry-run] Would create index for label {} on ({})",
                        label, property_list
                    );
                } else {
                    create_index_for_properties(graph, label, &properties).await?;
                }
            } else {
                warn!(
                    "Skipping invalid index_properties entry '{}' in spec '{}'",
                    entry, spec.name
                );
            }
        }
    }

    if dry_run {
        debug!(
            "[dry-run] Would execute ingest query for spec: {}",
            spec.name
        );
        return Ok(());
    }

    let limit = 2500;
    let mut last_rowid: i64 = 0;

    loop {
        let select_query = data_query_template
            .replacen("{}", &last_rowid.to_string(), 1)
            .replacen("{}", &limit.to_string(), 1);
        debug!("Executing query: {}", select_query);

        let mut stmt = sql_conn.prepare(&select_query)?;

        // Execute the query and collect all rows
        // Column 0 is rowid, mapped columns start at 1
        let rows: Vec<_> = stmt
            .query_map([], |row| {
                let rowid: i64 = row.get(0)?;

                if let Some(mappings) = &spec.column_mappings {
                    let mut mapped_values =
                        std::collections::HashMap::with_capacity(mappings.len());
                    for (i, cypher_param) in mappings.values().enumerate() {
                        let value: String = row.get(i + 1)?;
                        mapped_values.insert(cypher_param.clone(), value);
                    }
                    let data: String = row.get(mapped_values.len() + 1)?;
                    Ok((rowid, Some(mapped_values), data))
                } else {
                    let data: String = row.get(1)?;
                    Ok((rowid, None, data))
                }
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if rows.is_empty() {
            break;
        }

        // Update cursor to the last rowid in this batch
        last_rowid = rows.last().map(|(rid, _, _)| *rid).unwrap_or(last_rowid);

        // Process rows in parallel, consuming each row to free raw JSON strings early
        let processed_values: Vec<Value> = rows
            .into_iter()
            .map(|(_, mapped_values, data)| {
                let value: Value =
                    serde_json::from_str(&data).unwrap_or(Value::Object(serde_json::Map::new()));

                let mut new_value = Value::Object(serde_json::Map::new());

                if let Some(mappings) = mapped_values {
                    for (key, val) in mappings {
                        new_value
                            .as_object_mut()
                            .unwrap()
                            .insert(key, Value::String(val));
                    }
                }

                for property in &spec.properties {
                    if let Some(val) = value.pointer(property) {
                        if *property == "/id" {
                            new_value.as_object_mut().unwrap().insert(
                                property.trim_start_matches('/').to_string(),
                                val.as_str().unwrap_or("").to_lowercase().into(),
                            );
                        } else {
                            new_value
                                .as_object_mut()
                                .unwrap()
                                .insert(property.trim_start_matches('/').to_string(), val.clone());
                        }
                    }
                }
                new_value
            })
            .collect();

        debug!(
            "Inserting {} {} - {} (cursor at rowid {})",
            processed_values.len(),
            table_name,
            &spec.name,
            last_rowid
        );

        // Write batch to Neo4j
        let batch_len = processed_values.len();
        let bolt_batch = BoltType::try_from(Value::Array(processed_values))?;
        graph
            .run(query(&spec.cypher).param("batch", bolt_batch))
            .await
            .map_err(|e| CirroError::DatabaseError(e.to_string()))?;

        debug!("Processed {} {}s", batch_len, table_name);
    }
    Ok(())
}

/// Create default constraints for base labels
async fn create_default_constraints_and_indexes(graph: &neo4rs::Graph) -> Result<(), CirroError> {
    for label in ["ArmResource", "GraphObject"] {
        create_constraint(graph, label, "id").await?;
    }

    for label in ["GraphApplication", "GraphServicePrincipal"] {
        create_index(graph, label, "appId").await?;
    }

    Ok(())
}

fn parse_label_properties(entry: &str) -> Option<(&str, Vec<String>)> {
    let (label, props_raw) = entry.split_once(':')?;
    let label = label.trim();
    if label.is_empty() {
        return None;
    }

    let properties: Vec<String> = props_raw
        .split('+')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(ToString::to_string)
        .collect();

    if properties.is_empty() {
        None
    } else {
        Some((label, properties))
    }
}

async fn create_constraint_for_properties(
    graph: &neo4rs::Graph,
    label_name: &str,
    properties: &[String],
) -> Result<(), CirroError> {
    if properties.len() == 1 {
        return create_constraint(graph, label_name, &properties[0]).await;
    }

    let pattern = properties
        .iter()
        .map(|property| format!("n.{}", property))
        .collect::<Vec<_>>()
        .join(", ");

    let constraint_query = format!(
        "CREATE CONSTRAINT IF NOT EXISTS FOR (n:{}) REQUIRE ({}) IS UNIQUE",
        label_name, pattern
    );
    debug!("Executing query: {}", constraint_query);
    graph.run(query(&constraint_query)).await.map_err(|e| {
        CirroError::DatabaseError(format!(
            "Failed to create composite constraint for {}.({}): {}",
            label_name,
            properties.join(", "),
            e
        ))
    })?;

    Ok(())
}

async fn create_index_for_properties(
    graph: &neo4rs::Graph,
    label_name: &str,
    properties: &[String],
) -> Result<(), CirroError> {
    if properties.len() == 1 {
        return create_index(graph, label_name, &properties[0]).await;
    }

    let pattern = properties
        .iter()
        .map(|property| format!("n.{}", property))
        .collect::<Vec<_>>()
        .join(", ");
    let query_name = format!(
        "{}_{}_idx",
        label_name,
        properties
            .iter()
            .map(|property| property.replace('.', "_"))
            .collect::<Vec<_>>()
            .join("_")
    );

    let index_query = format!(
        "CREATE INDEX {} IF NOT EXISTS FOR (n:{}) ON ({})",
        query_name, label_name, pattern
    );
    debug!("Executing query: {}", index_query);
    graph.run(query(&index_query)).await.map_err(|e| {
        CirroError::DatabaseError(format!(
            "Failed to create composite index for {}.({}): {}",
            label_name,
            properties.join(", "),
            e
        ))
    })?;

    Ok(())
}

/// Create an index on a label's property (idempotent via IF NOT EXISTS)
async fn create_index(
    graph: &neo4rs::Graph,
    label_name: &str,
    property: &str,
) -> Result<(), CirroError> {
    let index_query = format!(
        "CREATE INDEX {}_{} IF NOT EXISTS FOR (n:{}) ON (n.{})",
        label_name, property, label_name, property
    );
    debug!("Executing query: {}", index_query);
    graph.run(query(&index_query)).await.map_err(|e| {
        CirroError::DatabaseError(format!(
            "Failed to create index for {}.{}: {}",
            label_name, property, e
        ))
    })?;
    Ok(())
}

/// Create a uniqueness constraint for a label's property (idempotent via IF NOT EXISTS)
async fn create_constraint(
    graph: &neo4rs::Graph,
    label_name: &str,
    property: &str,
) -> Result<(), CirroError> {
    let constraint_query = CREATE_CONSTRAINT_QUERY
        .replacen("{}", label_name, 1)
        .replacen("{}", property, 1);
    debug!("Executing query: {}", constraint_query);
    graph.run(query(&constraint_query)).await.map_err(|e| {
        CirroError::DatabaseError(format!(
            "Failed to create constraint for {}.{}: {}",
            label_name, property, e
        ))
    })?;
    Ok(())
}
