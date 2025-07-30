use crate::ingest::ingestor::CirroIngestor;
use crate::{errors::CirroIngestError, ingest::constants::GraphType};
use log::{debug, error, info};
use neo4rs::{BoltType, query};
use serde_json::Value;

impl CirroIngestor {
    /// Process Arm resources from the ingestor
    async fn process_arm_resource(
        &self,
        table_name: &str,
        node_insert_query: &str,
        properties: Vec<&str>,
    ) -> Result<(), CirroIngestError> {
        // Get the count of objects in the database
        let count = self
            .sql_conn
            .as_ref()
            .unwrap()
            .query_row(
                format!("SELECT COUNT(*) FROM {}", table_name).as_str(),
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0);

        if count == 0 {
            debug!("No {} found", table_name);
            return Ok(());
        }
        info!("Processing {:>5} {}", count, table_name);

        let limit = 5000;
        let mut offset = 0;

        loop {
            // If the offset exceeds the count, break the loop
            if offset * limit <= count {
                let select_query = format!(
                    "SELECT data FROM {} LIMIT {} OFFSET {}",
                    table_name,
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
                let rows = stmt.query_map([], |row| row.get(0))?;

                let processed_values: Vec<Value> = rows
                    .map(|result| {
                        result.map(|data: String| {
                            // Deserialize the JSON string into a Value
                            let value: Value = serde_json::from_str(&data)
                                .unwrap_or(Value::Object(serde_json::Map::new()));

                            // Create a new Value to hold the processed data
                            let mut new_value = Value::Object(serde_json::Map::new());

                            // For each property, set the value in the new Value
                            for property in &properties {
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
                        })
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
                        .run(query(node_insert_query).param(
                            "batch",
                            BoltType::try_from(serde_json::to_value(&processed_values)?)?,
                        ))
                        .await
                        .map_err(|e| CirroIngestError::DatabaseError(e.to_string()))?;
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

    /// Process ARM resource types
    pub async fn process_specific_arm_resource(
        &self,
        resource_type: &str,
        node_insert_query: &str,
        properties: Vec<&str>,
    ) -> Result<(), CirroIngestError> {
        // Get the count of objects in the database
        let count_query = format!(
            "SELECT COUNT(*) FROM resources WHERE lower(resource_type) = '{}'",
            resource_type.to_lowercase()
        );
        debug!("Executing sqlite count query: {}", count_query);
        let count = self
            .sql_conn
            .as_ref()
            .unwrap()
            .query_row(&count_query, [], |row| row.get::<_, i64>(0))
            .unwrap_or(0);

        if count == 0 {
            debug!("No {} found", resource_type);
            return Ok(());
        }
        info!("Processing {:>5} {}", count, resource_type);

        let limit = 5000;
        let mut offset = 0;

        loop {
            // If the offset exceeds the count, break the loop
            if offset * limit <= count {
                let select_query = format!(
                    "SELECT rg_id, data FROM resources WHERE lower(resource_type) = '{}' LIMIT {} OFFSET {}",
                    resource_type.to_lowercase(),
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
                    let rg_id: String = row.get(0)?;
                    let data: String = row.get(1)?;
                    Ok((rg_id, data))
                })?;

                let processed_values: Vec<Value> = rows
                    .map(|result| {
                        result.map(|(rg_id, data): (String, String)| {
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
                            new_value
                                .as_object_mut()
                                .unwrap()
                                .insert("resourcegroup_id".to_string(), Value::String(rg_id));
                            new_value
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Insert the processed values into the graph
                if !processed_values.is_empty() {
                    debug!(
                        "Inserting {} {} from offset {}",
                        processed_values.len(),
                        resource_type,
                        offset
                    );
                    let _ = self
                        .graph
                        .run(query(node_insert_query).param(
                            "batch",
                            BoltType::try_from(serde_json::to_value(&processed_values)?)?,
                        ))
                        .await
                        .map_err(|e| CirroIngestError::DatabaseError(e.to_string()))?;
                } else {
                    error!(
                        "No values to insert for {} at offset {}",
                        resource_type, offset
                    );
                }
                offset += 1;
                debug!("Processed {} {}s", &processed_values.len(), resource_type);
            } else {
                break;
            }
        }
        Ok(())
    }

    // Process tenants
    pub async fn process_tenants(&self) -> Result<(), CirroIngestError> {
        let properties = vec![
            "/id",
            "/displayName",
            "/tenantId",
            "/countryCode",
            "/domains",
            "/defaultDomain",
            "/tenantCategory",
            "/tenantType",
        ];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (t:Tenant {id: row.id})
            SET t += {
                displayName : row.displayName,
                tenantId : row.tenantId,
                countryCode : row.countryCode,
                domains : row.domains,
                defaultDomain : row.defaultDomain,
                tenantCategory : row.tenantCategory,
                tenantType : row.tenantType
            }

            MERGE (o:GraphOrg {id: row.tenantId})
            MERGE (o)-[:ASSOCIATED_WITH]->(t)
            MERGE (t)-[:ASSOCIATED_WITH]->(o)
        "#;

        self.process_arm_resource("tenants", node_insert_query, properties)
            .await?;

        Ok(())
    }

    // Process subscriptions
    pub async fn process_subscriptions(&self) -> Result<(), CirroIngestError> {
        let properties = vec![
            "/id",
            "/displayName",
            "/authorizationSource",
            "/state",
            "/subscriptionId",
            "/tenantId",
        ];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (s:Subscription {id: row.id})
            SET s:ArmResource
            SET s += {
                displayName : row.displayName,
                authorizationSource : row.authorizationSource,
                state : row.state,
                subscriptionId : row.subscriptionId,
                tenantId : row.tenantId
            }
            MERGE (t:Tenant {tenantId: row.tenantId})
            MERGE (t)-[:CONTAINS]->(s)
        "#;

        self.process_arm_resource("subscriptions", node_insert_query, properties)
            .await?;

        Ok(())
    }

    // Process resource groups
    pub async fn process_resource_groups(&self) -> Result<(), CirroIngestError> {
        let properties = vec!["/id", "/name", "/location", "/type"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (rg:ArmResource {id: row.id})
            SET rg:ResourceGroup
            SET rg += {
                name : row.name,
                location : row.location,
                type : row.type
            }
            MERGE (s:Subscription {id: row.subscriptionId})
            MERGE (s)-[:CONTAINS]->(rg)
        "#;

        // Get the count of objects in the database
        let count = self
            .sql_conn
            .as_ref()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM resources WHERE lower(resource_type) = 'microsoft.resources/resourcegroups'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0);

        if count == 0 {
            debug!("No resource groups found");
            return Ok(());
        }
        info!("Processing {:>5} resource groups", count);

        let limit = 5000;
        let mut offset = 0;

        loop {
            // If the offset exceeds the count, break the loop
            if offset * limit <= count {
                let select_query = format!(
                    "SELECT sub_id, json(data) FROM resources WHERE lower(resource_type) = 'microsoft.resources/resourcegroups' LIMIT {} OFFSET {}",
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
                    let sub_id: String = row.get(0)?;
                    let data: String = row.get(1)?;
                    Ok((sub_id, data))
                })?;

                let processed_values: Vec<Value> = rows
                    .map(|result| {
                        result.map(|(sub_id, data): (String, String)| {
                            // Deserialize the JSON string into a Value
                            let value: Value = serde_json::from_str(&data)
                                .unwrap_or(Value::Object(serde_json::Map::new()));

                            // Create a new Value to hold the processed data
                            let mut new_value = Value::Object(serde_json::Map::new());

                            // For each property, set the value in the new Value
                            for property in &properties {
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
                            // Add the subscription ID to the new Value
                            new_value.as_object_mut().unwrap().insert(
                                "subscriptionId".to_string(),
                                Value::String(sub_id.to_lowercase()),
                            );
                            new_value
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Insert the processed values into the graph
                if !processed_values.is_empty() {
                    debug!(
                        "Inserting {} resource groups from offset {}",
                        processed_values.len(),
                        offset
                    );
                    let _ = self
                        .graph
                        .run(query(node_insert_query).param(
                            "batch",
                            BoltType::try_from(serde_json::to_value(&processed_values)?)?,
                        ))
                        .await
                        .map_err(|e| CirroIngestError::DatabaseError(e.to_string()))?;
                } else {
                    error!(
                        "No values to insert for resource groups at offset {}",
                        offset
                    );
                }
                offset += 1;
                debug!("Processed {} resource groups", &processed_values.len());
            } else {
                break;
            }
        }

        Ok(())
    }

    // Process generic ARM resources
    pub async fn process_generic_arm_resources(&self) -> Result<(), CirroIngestError> {
        let properties = vec![
            "/id",
            "/identity",
            "/kind",
            "/location",
            "/name",
            "/type",
            "/tags",
        ];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj += {
                id : row.id,
                kind : row.kind,
                location : row.location,
                name : row.name,
                type : row.type,
                tags : [key in keys(row.tags) | key + ":" + row.tags[key]]
            }
            MERGE (rg:ArmResource {id: row.resourcegroup_id})
                SET rg:ResourceGroup
                MERGE (rg)-[:HAS_RESOURCE]->(obj)
            WITH obj, row
                WHERE row.identity IS NOT NULL AND toLower(row.identity.type) = 'systemassigned'
                    MERGE (i:GraphObject {id: row.identity.principalId})
                    MERGE (obj)-[:HAS_IDENTITY]->(i)
            WITH obj, row
                WHERE row.identity IS NOT NULL AND toLower(row.identity.type) = 'userassigned'
                    MERGE (i:GraphObject {id: row.identity.principalId})
                    MERGE (obj)-[:HAS_IDENTITY]->(i)
      
        "#;

        // Get the count of objects in the database
        let count = self
            .sql_conn
            .as_ref()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM resources WHERE lower(resource_type) != 'microsoft.resources/resourcegroups'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0);

        if count == 0 {
            debug!("No resources found");
            return Ok(());
        }
        info!("Processing {:>5} resources", count);

        let limit = 5000;
        let mut offset = 0;

        loop {
            // If the offset exceeds the count, break the loop
            if offset * limit <= count {
                let select_query = format!(
                    "SELECT rg_id, data FROM resources WHERE lower(resource_type) != 'microsoft.resources/resourcegroups' LIMIT {} OFFSET {}",
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
                    let rg_id: String = row.get(0)?;
                    let data: String = row.get(1)?;
                    Ok((rg_id, data))
                })?;

                let processed_values: Vec<Value> = rows
                    .map(|result| {
                        result.map(|(rg_id, data): (String, String)| {
                            // Deserialize the JSON string into a Value
                            let value: Value = serde_json::from_str(&data)
                                .unwrap_or(Value::Object(serde_json::Map::new()));

                            // Create a new Value to hold the processed data
                            let mut new_value = Value::Object(serde_json::Map::new());

                            // For each property, set the value in the new Value
                            for property in &properties {
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
                            // Add the resource group ID to the new Value
                            new_value.as_object_mut().unwrap().insert(
                                "resourcegroup_id".to_string(),
                                Value::String(rg_id.to_lowercase()),
                            );
                            new_value
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Insert the processed values into the graph
                if !processed_values.is_empty() {
                    debug!(
                        "Inserting {} resource from offset {}",
                        processed_values.len(),
                        offset
                    );
                    let _ = self
                        .graph
                        .run(query(node_insert_query).param(
                            "batch",
                            BoltType::try_from(serde_json::to_value(&processed_values)?)?,
                        ))
                        .await
                        .map_err(|e| CirroIngestError::DatabaseError(e.to_string()))?;
                } else {
                    error!(
                        "No values to insert for resource groups at offset {}",
                        offset
                    );
                }
                offset += 1;
                debug!("Processed {} resources", &processed_values.len());
            } else {
                break;
            }
        }

        Ok(())
    }
    // Process Azure role assignments
    pub async fn process_role_assignments(&self) -> Result<(), CirroIngestError> {
        let properties = vec![
            "/id",
            "/description",
            "/roleName",
            "/roleType",
            "/permissions",
            "/properties",
        ];

        // We need to pass dynamic role names, so we need APOC/MAGE
        let node_insert_query = match self.graph_type {
            GraphType::Neo4j => {
                r#"
                    UNWIND $batch AS row
                    MERGE (o:GraphObject {id: row.properties.principalId})
                    WITH o, row
                        MATCH (r:ArmResource {id: toLower(row.properties.scope)})
                        CALL apoc.merge.relationship(o, row.roleName, {}, {}, r) YIELD rel
                        SET rel += {
                            id : row.id,
                            description : row.description,
                            roleName : row.roleName,
                            roleType : row.roleType,
                            actions : row.permissions[0].actions,
                            notActions : row.permissions[0].notActions,
                            dataActions : row.permissions[0].dataActions,
                            notDataActions : row.permissions[0].notDataActions
                        }
                "#
            }

            // Memgraph doesn't allow for CALL procedures after a CREATE/MERGE
            // Need to implement a way to merge the nodes first
            // Otherwise, process_role_assignments needs to be run last in the processors
            // Cause this will not show any nodes that are not already in the database
            GraphType::Memgraph => {
                r#"
                    UNWIND $batch AS row
                    MATCH (o:GraphObject {id: row.properties.principalId})
                    MATCH (r:ArmResource {id: toLower(row.properties.scope)})
                    CALL merge.relationship(o, row.roleName, {}, {
                        description : row.description,
                        roleName : row.roleName,
                        roleType : row.roleType,
                        actions : row.permissions[0].actions,
                        notActions : row.permissions[0].notActions,
                        dataActions : row.permissions[0].dataActions,
                        notDataActions : row.permissions[0].notDataActions
                    }, r, {
                        description : row.description,
                        roleName : row.roleName,
                        roleType : row.roleType,
                        actions : row.permissions[0].actions,
                        notActions : row.permissions[0].notActions,
                        dataActions : row.permissions[0].dataActions,
                        notDataActions : row.permissions[0].notDataActions
                    }) YIELD rel
                "#
            }
        };

        self.process_arm_resource("roleAssignments", node_insert_query, properties)
            .await?;

        Ok(())
    }
}
