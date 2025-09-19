use log::{debug, info, warn};
use neo4rs::*;
use regex::Regex;
use serde_json::{Map, Value};
use std::path::PathBuf;

use crate::EnrichFlags;
use crate::errors::CirroGraphError;

pub struct EnrichConfigurator {
    output_file: PathBuf,
    server: String,
    user: String,
    db_name: String,
    graph: Graph,
    relation_depth: isize,
    flags: EnrichFlags,
}

#[derive(serde::Serialize, Default)]
struct EnrichConfig {
    id: String,
    need_graph_token: bool,
    need_arm_token: bool,
    checks: Map<String, Value>,
}

#[derive(Debug)]
struct EnrichmentPath {
    resource_id: String,
    resource: String,
    role_name: String,
    actions: Vec<String>,
    not_actions: Vec<String>,
    data_actions: Vec<String>,
    not_data_actions: Vec<String>,
}

/// Custom Debug trait implementation for EnrichConfigurator
impl std::fmt::Debug for EnrichConfigurator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnrichConfigurator")
            .field("output_file", &self.output_file)
            .field("server", &self.server)
            .field("user", &self.user)
            .field("db_name", &self.db_name)
            .field("flags", &self.flags)
            .finish()
    }
}

impl EnrichConfigurator {
    /// Initialize the EnrichConfigurator with database connection
    pub async fn new(
        output_file: PathBuf,
        server: String,
        user: String,
        password: String,
        db_name: String,
        depth: isize,
        flags: EnrichFlags,
    ) -> Result<Self, CirroGraphError> {
        let neo4j_config = ConfigBuilder::default()
            .uri(&server)
            .user(&user)
            .password(&password)
            .db(db_name.clone())
            .build()
            .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))
            .unwrap();

        info!("Connecting to database at {} with user {}", server, user);
        let graph = Graph::connect(neo4j_config)
            .await
            .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))
            .unwrap();

        // Test connection to the database
        let mut result = graph.execute(query("RETURN 1")).await.unwrap();
        let row = result.next().await.unwrap().unwrap();
        let value: i64 = row.get("1").unwrap();
        assert_eq!(1, value);
        info!("Successfully connected to the database");

        Ok(EnrichConfigurator {
            output_file,
            server,
            user,
            db_name,
            graph,
            relation_depth: depth,
            flags,
        })
    }

    /// Make the enrichment configuration based on flags
    pub async fn make_config(&self, id: uuid::Uuid) -> Result<(), CirroGraphError> {
        info!("Starting enrichment for object ID: {}", id);

        let mut enrich_config = EnrichConfig::default();
        enrich_config.id = id.to_string();

        // STORAGE KEYS
        if self.flags.all || self.flags.storage_keys {
            enrich_config.need_arm_token = true;
            info!("Collecting storage accounts for storage key enrichment");
            match self.get_storage_accounts_for_keys(&id).await {
                Ok(accounts) => {
                    enrich_config
                        .checks
                        .insert("storage_account_keys".to_string(), accounts.into());
                }
                Err(e) => {
                    warn!("Error fetching storage accounts for keys: {}", e);
                }
            }
        }

        // Write pretty JSON to output file
        let _ = std::fs::write(
            &self.output_file,
            serde_json::to_string_pretty(&enrich_config)
                .map_err(|e| CirroGraphError::ProcessingError(e.to_string()))?,
        );
        info!(
            "Enrichment configuration written to {}",
            self.output_file.as_path().canonicalize().unwrap().display()
        );

        Ok(())
    }

    /// Get paths from object to specific resource types
    /// resource_type should be a label, e.g., "StorageAccount"
    async fn get_rbac_path_to_resource(
        &self,
        id: &uuid::Uuid,
        resource_type: &str,
    ) -> Result<Vec<EnrichmentPath>, CirroGraphError> {
        let query_str = format!(
            r#"
            MATCH p=(n:GraphObject{{id:'{id}'}})-[*..{depth}]->(s:{resource_type})
            WITH p, nodes(p) AS ns, relationships(p) AS rs, s
            WHERE ALL(idx IN RANGE(1, size(ns)-2)
                    WHERE NOT ns[idx]:GraphObject
                        OR (
                            ns[idx]:GraphGroup 
                            AND type(rs[idx-1]) = "MEMBER_OF"
                        ))
            WITH s, [r IN rs WHERE r.roleName IS NOT NULL | r][0] AS firstRel
            WHERE firstRel IS NOT NULL
            RETURN DISTINCT
                s.id AS id,
                s.name AS name,
                firstRel.roleName AS roleName,
                [x IN firstRel.actions WHERE x IS NOT NULL | toLower(x)] AS actions,
                [x IN firstRel.notActions WHERE x IS NOT NULL | toLower(x)] AS notActions,
                [x IN firstRel.dataActions WHERE x IS NOT NULL | toLower(x)] AS dataActions,
                [x IN firstRel.notDataActions WHERE x IS NOT NULL | toLower(x)] AS notDataActions
            "#,
            id = id,
            depth = self.relation_depth,
            resource_type = resource_type
        );

        debug!(
            "Executing query to find paths to resource type {}:\n{}",
            resource_type, query_str
        );

        let mut query_result = self
            .graph
            .execute(query(&query_str))
            .await
            .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))?;

        let mut paths = Vec::new();

        while let Ok(Some(row)) = query_result.next().await {
            let resource_id: String = row
                .get("id")
                .map_err(|e| CirroGraphError::ProcessingError(e.to_string()))?;
            let resource: String = row
                .get("name")
                .map_err(|e| CirroGraphError::ProcessingError(e.to_string()))?;
            let role_name: String = row
                .get("roleName")
                .map_err(|e| CirroGraphError::ProcessingError(e.to_string()))?;
            let actions: Vec<String> = row
                .get("actions")
                .map_err(|e| CirroGraphError::ProcessingError(e.to_string()))?;
            let not_actions: Vec<String> = row
                .get("notActions")
                .map_err(|e| CirroGraphError::ProcessingError(e.to_string()))?;
            let data_actions: Vec<String> = row
                .get("dataActions")
                .map_err(|e| CirroGraphError::ProcessingError(e.to_string()))?;
            let not_data_actions: Vec<String> = row
                .get("notDataActions")
                .map_err(|e| CirroGraphError::ProcessingError(e.to_string()))?;

            paths.push(EnrichmentPath {
                resource_id,
                resource,
                role_name,
                actions,
                not_actions,
                data_actions,
                not_data_actions,
            });
        }

        Ok(paths)
    }

    // Get storage accounts accessible by the id for key retrieval
    /// Returns a JSON array of storage account IDs
    async fn get_storage_accounts_for_keys(
        &self,
        id: &uuid::Uuid,
    ) -> Result<Vec<String>, CirroGraphError> {
        let required_actions = vec!["microsoft.storage/storageaccounts/listkeys/action"];
        let paths = self.get_rbac_path_to_resource(id, "StorageAccount").await?;
        debug!(
            "Found {} unfiltered storage accounts paths for keys",
            paths.len()
        );

        let filtered_paths = filter_paths_by_permissions(paths, &required_actions, &[]);

        let filtered_accounts: Vec<String> = filtered_paths
            .into_iter()
            .map(|path| path.resource_id)
            .collect();

        Ok(filtered_accounts)
    }
}

// Filter EnrichmentPaths based on required actions and data actions
/// Returns only the paths that have the necessary permissions
fn filter_paths_by_permissions(
    paths: Vec<EnrichmentPath>,
    required_actions: &[&str],
    required_data_actions: &[&str],
) -> Vec<EnrichmentPath> {
    paths
        .into_iter()
        .filter(|path| {
            let regular_actions_ok = if required_actions.is_empty() {
                true
            } else {
                is_allowed(
                    path.actions.clone(),
                    path.not_actions.clone(),
                    required_actions.iter().map(|s| s.to_string()).collect(),
                )
            };

            let data_actions_ok = if required_data_actions.is_empty() {
                true
            } else {
                is_allowed(
                    path.data_actions.clone(),
                    path.not_data_actions.clone(),
                    required_data_actions.iter().map(|s| s.to_string()).collect(),
                )
            };

            // Check if actions contain "*" which grants everything
            let has_wildcard = path.actions.iter().any(|action| action == "*");

            let is_accessible = (regular_actions_ok && data_actions_ok) || has_wildcard;

            if is_accessible {
                info!(
                    "\t{:<40} - accessible via: {:?}",
                    path.resource, path.role_name
                );
            } else {
                debug!(
                    "Resource {} is NOT accessible. Actions: {:?}, NotActions: {:?}, DataActions: {:?}, NotDataActions: {:?}",
                    path.resource,
                    path.actions,
                    path.not_actions,
                    path.data_actions,
                    path.not_data_actions
                );
            }

            is_accessible
        })
        .collect()
}

/// Helper function to converts an Azure action pattern into a regex and checks if it matches the action
fn match_azure_action(pattern: &str, action: &str) -> bool {
    // If the pattern is "*", it matches everything
    if pattern == "*" {
        return true;
    }

    // Escape regex metacharacters except '*'
    let mut regex_pattern = regex::escape(pattern);

    // Replace escaped '*' with a regex fragment
    // This means '*' matches one full segment (no slashes)
    regex_pattern = regex_pattern.replace("\\*", "[^/]+");

    // Anchor to whole string
    regex_pattern = format!("^{}$", regex_pattern);

    let rex = Regex::new(&regex_pattern).unwrap();
    rex.is_match(action)
}

fn is_allowed(actions: Vec<String>, not_actions: Vec<String>, requested: Vec<String>) -> bool {
    let mut allowed = false;
    for req in &requested {
        let mut req_allowed = false;

        // Check against not_actions first
        // Not actions take precedence over all actions, regardless of role
        for na in &not_actions {
            if match_azure_action(na, req) {
                req_allowed = false;
                break;
            }
        }

        if !req_allowed {
            // Now check against actions
            for a in &actions {
                if match_azure_action(a, req) {
                    req_allowed = true;
                    break;
                }
            }
        }
        if req_allowed {
            allowed = true;
        } else {
            return false; // If any requested action is not allowed, return false
        }
    }
    allowed
}
