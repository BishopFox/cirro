use super::common::*;
use crate::collect::azure::collect::Collector;
use crate::collect::azure::credentials::common::Token;
use crate::collect::azure::db::DBWriteMessage;
use crate::errors::CirroError;

use dashmap::DashMap;
use futures::stream::{self, StreamExt};
use log::warn;
use log::{debug, info};
use once_cell::sync::Lazy;
use reqwest::Client;
use reqwest::StatusCode;
use serde_json::Map;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

// Maximum number of concurrent resource group enumerations per subscription
const MAX_CONCURRENT_RESOURCE_GROUPS: usize = 16;
// Maximum number of concurrent resource detail fetches per resource group
const MAX_CONCURRENT_RESOURCES: usize = 32;

// Create a global reqwest client to reuse connections
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
        .pool_max_idle_per_host(100)
        .build()
        .unwrap()
});

// Global static cache for the token
static TOKEN_CACHE: Lazy<tokio::sync::Mutex<Option<Token>>> =
    Lazy::new(|| tokio::sync::Mutex::new(None));

// Global thread-safe map for role definitions
// Key: Role definition ID, Value: Role definition object
static ROLE_DEFINITIONS: Lazy<DashMap<String, Value>> = Lazy::new(|| DashMap::new());

// Global rate limit gate: when set, all tasks must wait until this instant before sending requests
static RATE_LIMIT_UNTIL: Lazy<tokio::sync::Mutex<Option<tokio::time::Instant>>> =
    Lazy::new(|| tokio::sync::Mutex::new(None));

/// Checks whether a given state key is marked as completed in the resume state.
fn is_completed(resume_state: &HashMap<String, (String, usize, f64)>, key: &str) -> bool {
    resume_state
        .get(key)
        .map_or(false, |(status, _, _)| status == "done")
}

/// Sends a SaveState message to mark a unit of work as done.
async fn save_arm_state(
    collector: &Collector,
    resource_type: String,
    total_collected: usize,
    elapsed_secs: f64,
) {
    if let Some(sender) = &collector.db_writer {
        let _ = sender
            .send(DBWriteMessage::SaveState {
                resource_type,
                next_uri: "done".to_string(),
                total_collected,
                elapsed_secs,
            })
            .await;
    }
}

/// Sends a ClearState message to remove a state key.
async fn clear_arm_state(collector: &Collector, resource_type: String) {
    if let Some(sender) = &collector.db_writer {
        let _ = sender
            .send(DBWriteMessage::ClearState { resource_type })
            .await;
    }
}

/// Queries resources from the ARM API
async fn query_resources(
    collector: Arc<Collector>,
    uri: &str,
) -> Result<Vec<Map<String, Value>>, CirroError> {
    debug!("Querying: {}", uri);

    let mut resources: Vec<Map<String, Value>> = Vec::new();
    let base_url = collector.cloud_endpoints.arm_url;
    let mut next_url = if uri.starts_with(base_url) {
        uri.to_owned()
    } else {
        format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            uri.trim_start_matches('/')
        )
    };

    loop {
        let response_data = loop {
            match paged_arm_request(&collector, &next_url, reqwest::Method::GET, None).await {
                Ok(response) => break response,
                Err(error) => {
                    return Err(error);
                }
            }
        };

        if response_data.get("error").is_some() {
            return Err(CirroError::HttpError(
                "Error response from ARM API".to_string(),
            ));
        }

        if let Some(value) = response_data.get("value") {
            // Sometimes the value is an array, sometimes it's a single object
            if let Some(obj) = value.as_object() {
                resources.push(obj.clone());
            } else if let Some(arr) = value.as_array() {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        resources.push(obj.clone());
                    }
                }
            }
        } else if let Some(obj) = response_data.as_object() {
            // Also handle the case where the response is a single object
            resources.push(obj.clone());
        } else {
            return Err(CirroError::HttpError(format!(
                "Unexpected response format from ARM API: {}",
                next_url
            )));
        }

        // Check for next link
        if let Some(next_link) = response_data
            .get("@odata.nextLink")
            .or_else(|| response_data.get("nextLink"))
            .and_then(|v| v.as_str())
        {
            next_url = next_link.to_string();
            debug!("Fetching next page: {}", next_url);
        } else {
            break; // No more pages to fetch
        }
    }
    Ok(resources.into())
}

/// Makes a paginated request to the ARM API with retry logic
pub async fn paged_arm_request(
    collector: &Collector,
    uri: &str,
    http_verb: reqwest::Method,
    body: Option<String>,
) -> Result<Value, CirroError> {
    // Build the full URL correctly
    let arm_url = if uri.starts_with(&collector.cloud_endpoints.arm_url) {
        uri.to_owned()
    } else {
        format!(
            "{}/{}",
            collector.cloud_endpoints.arm_url.trim_end_matches('/'),
            uri.trim_start_matches('/')
        )
    };

    // Get the access token, using cache when possible
    let token_str = {
        let mut cache = TOKEN_CACHE.lock().await;

        // Check if we need a new token based on the token's actual expiration
        let need_new_token = match &*cache {
            Some(cached_token) => cached_token.is_expired_or_not_set(),
            None => true,
        };

        if need_new_token {
            debug!("Fetching new ARM token");
            let token = collector.arm_credential.get_token().await?;

            // Log expiration information for debugging
            if let Some(expires) = token.expires_on {
                let now = chrono::Utc::now();
                let time_until_expiry = expires.signed_duration_since(now);
                debug!(
                    "New token expires at {} (valid for {} minutes)",
                    expires,
                    time_until_expiry.num_minutes()
                );
            } else {
                debug!("New token has no expiration time set");
            }

            *cache = Some(token.clone());
            Arc::clone(&token.access_token)
        } else {
            // Return a clone of the cached token
            Arc::clone(&cache.as_ref().unwrap().access_token)
        }
    };

    let mut retries = 0;
    let max_retries = 5;

    let response = loop {
        // Block if a global rate limit is active before sending any request
        {
            let rate_limit = RATE_LIMIT_UNTIL.lock().await;
            if let Some(until) = *rate_limit {
                let now = tokio::time::Instant::now();
                if now < until {
                    let wait = until - now;
                    drop(rate_limit);
                    debug!(
                        "Waiting {:.1}s for global rate limit gate",
                        wait.as_secs_f64()
                    );
                    tokio::time::sleep_until(until).await;
                }
            }
        }

        // Make the request using the global client
        let result = match http_verb {
            reqwest::Method::GET => {
                HTTP_CLIENT
                    .get(&arm_url)
                    .bearer_auth(&*token_str)
                    .timeout(std::time::Duration::from_secs(30))
                    .send()
                    .await
            }
            reqwest::Method::POST => {
                HTTP_CLIENT
                    .post(&arm_url)
                    .bearer_auth(&*token_str)
                    .body(body.clone().unwrap_or_default())
                    .header("content-length", body.as_ref().map_or(0, |b| b.len()))
                    .timeout(std::time::Duration::from_secs(30))
                    .send()
                    .await
            }
            _ => return Err(CirroError::UnsupportedHttpMethod(http_verb.to_string())),
        };

        match result {
            Ok(response) => {
                let status = response.status();

                // Check rate limiting FIRST before any other status handling
                if status == StatusCode::TOO_MANY_REQUESTS {
                    if retries >= max_retries {
                        return Err(CirroError::HttpError(
                            "Too many requests, exceeded max retries".to_string(),
                        ));
                    }

                    // Set the global rate limit gate so all concurrent tasks block
                    let now = tokio::time::Instant::now();
                    let backoff_until = now + std::time::Duration::from_secs(25);
                    {
                        let mut rate_limit = RATE_LIMIT_UNTIL.lock().await;
                        // Only set the gate if it's not already active
                        if rate_limit.map_or(true, |existing| now >= existing) {
                            info!("ARM API rate limit hit, blocking all tasks for 25 seconds");
                            *rate_limit = Some(backoff_until);
                        }
                    }

                    retries += 1;
                    tokio::time::sleep_until(backoff_until).await;
                    continue;
                } else if status.is_success() {
                    break response;
                } else {
                    let error_text = response.text().await?;
                    return Err(CirroError::HttpError(format!(
                        "HTTP {} - {}",
                        status.as_u16(),
                        error_text
                    )));
                }
            }
            Err(e) => {
                if retries >= max_retries {
                    return Err(CirroError::HttpError(format!(
                        "Failed to send request to {} after {} retries: {}",
                        arm_url, retries, e
                    )));
                }

                debug!(
                    "Request timeout for {}, retrying after 5 seconds (attempt {}/{})",
                    arm_url,
                    retries + 1,
                    max_retries
                );
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                retries += 1;
                continue;
            }
        }
    };

    // Parse the response data
    let response_data: Value = response.json::<Value>().await?;

    // Check for OData error
    if let Some(err) = response_data.get("@odata.error") {
        let msg = err
            .get("message")
            .and_then(|m| m.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown odata.error");
        return Err(CirroError::ODataError(msg.to_string()));
    }

    Ok(response_data)
}

/// Enumerates role assignments for a given scope
async fn enumerate_role_assignments(
    collector: Arc<Collector>,
    scope: &str,
) -> Vec<Map<String, Value>> {
    let mut roles: Vec<Map<String, Value>> = Vec::new();

    let uri = format!(
        "{}/providers/Microsoft.Authorization/roleAssignments?api-version=2022-04-01",
        scope
    );
    debug!("Querying role assignments at {}", uri);

    let assignments = query_resources(collector.clone(), &uri)
        .await
        .unwrap_or_else(|e| {
            info!(
                "Failed to query role assignments for scope {}: {}",
                scope, e
            );
            roles.clone()
        });
    if assignments.is_empty() {
        info!("No role assignments found for scope {}", scope);
        return roles;
    }

    // Need to get the roleDefinitionId from the properties map
    for assignment in assignments {
        if let Some(properties) = assignment.get("properties").and_then(Value::as_object) {
            if let Some(role_definition_id) =
                properties.get("roleDefinitionId").and_then(Value::as_str)
            {
                let definition_uri = format!(
                    "{}?disambiguation_dummy&api-version=2022-04-01",
                    role_definition_id
                );

                let definition_response: Value;

                if let Some(cached_definition) = ROLE_DEFINITIONS.get(role_definition_id) {
                    definition_response = cached_definition.clone();
                } else {
                    match paged_arm_request(&collector, &definition_uri, reqwest::Method::GET, None)
                        .await
                    {
                        Ok(definition) => {
                            ROLE_DEFINITIONS
                                .insert(role_definition_id.to_string(), definition.clone());
                            definition_response = definition;
                        }
                        Err(e) => {
                            warn!(
                                "Failed to fetch role definition for {}: {}",
                                role_definition_id, e
                            );
                            continue; // Skip this assignment if we can't fetch the definition
                        }
                    }
                }

                // Add the role definition to the assignment
                let mut assignment_with_definition = assignment;
                assignment_with_definition.insert(
                    "permissions".to_string(),
                    definition_response
                        .pointer("/properties/permissions")
                        .cloned()
                        .unwrap_or(Value::Null),
                );
                assignment_with_definition.insert(
                    "roleName".to_string(),
                    definition_response
                        .pointer("/properties/roleName")
                        .cloned()
                        .unwrap_or(Value::Null),
                );
                assignment_with_definition.insert(
                    "roleType".to_string(),
                    definition_response
                        .pointer("/properties/type")
                        .cloned()
                        .unwrap_or(Value::Null),
                );
                assignment_with_definition.insert(
                    "description".to_string(),
                    definition_response
                        .pointer("/properties/description")
                        .cloned()
                        .unwrap_or(Value::Null),
                );

                // Add the assignment to the roles list
                roles.push(assignment_with_definition);
            }
        }
    }
    return roles;
}

/// Enumerates management group entities
async fn enumerate_management_groups(
    collector: Arc<Collector>,
) -> Result<Vec<Map<String, Value>>, CirroError> {
    let mut resources: Vec<Map<String, Value>> = Vec::new();

    let mut next_url =
        "providers/Microsoft.Management/getEntities?api-version=2021-04-01&$top=500".to_string();
    loop {
        let response =
            paged_arm_request(&collector, &next_url, reqwest::Method::POST, None).await?;

        if response.get("error").is_some() {
            return Err(CirroError::HttpError(
                "Error response from ARM API".to_string(),
            ));
        }

        if let Some(value) = response.get("value") {
            // Sometimes the value is an array, sometimes it's a single object
            if let Some(obj) = value.as_object() {
                resources.push(obj.clone());
            } else if let Some(arr) = value.as_array() {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        resources.push(obj.clone());
                    }
                }
            }
        } else if let Some(obj) = response.as_object() {
            // Also handle the case where the response is a single object
            resources.push(obj.clone());
        } else {
            return Err(CirroError::HttpError(format!(
                "Unexpected response format from ARM API: {}",
                next_url
            )));
        }

        // Check for next link
        if let Some(next_link) = response
            .get("@odata.nextLink")
            .or_else(|| response.get("nextLink"))
            .and_then(|v| v.as_str())
        {
            next_url = next_link.to_string();
        } else {
            break; // No more pages to fetch
        }
    }
    Ok(resources.into())
}

/// Enumerates a subscription
async fn enumerate_subscription(
    collector: Arc<Collector>,
    subscription: &Map<String, Value>,
    resume_state: Arc<HashMap<String, (String, usize, f64)>>,
) -> Result<(), CirroError> {
    let subscription_id = subscription
        .get("subscriptionId")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CirroError::ArmApiError("Subscription ID not found in subscription object".to_string())
        })?;

    let subscription_name = subscription
        .get("displayName")
        .and_then(Value::as_str)
        .unwrap_or("Unknown Subscription");

    info!(
        "Enumerating ARM subscription: {} ({})",
        subscription_id, subscription_name,
    );

    // Get the role assignments for this subscription
    let roles = enumerate_role_assignments(
        collector.clone(),
        format!("subscriptions/{}", subscription_id).as_str(),
    )
    .await;
    if roles.is_empty() {
        info!(
            "No role assignments found for subscription {}",
            subscription_id
        );
        // We might not have permissions to view role assignments, so continue
        // return Ok(());
    }
    info!(
        "Found {} role assignments for subscription {} ({})",
        roles.len(),
        subscription_id,
        subscription_name
    );

    // Write the roles to the database
    let mut role_rows = Vec::with_capacity(roles.len());
    for role in roles {
        let role_id = role
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| CirroError::ArmApiError("Role ID not found in role object".to_string()))?
            .to_lowercase();
        role_rows.push((role_id, Value::Object(role)));
    }
    if !role_rows.is_empty() {
        collector
            .write_values_batch_to_db("roleAssignments".into(), role_rows)
            .await?;
    }

    // Get all the resource providers for this subscription
    // These are stored for reuse to avoid repetitive API calls
    let resource_providers: DashMap<String, Vec<String>> = DashMap::new();
    let provider_uri = format!(
        "subscriptions/{}/providers?api-version=2021-04-01",
        subscription_id
    );
    let providers_response = query_resources(collector.clone(), &provider_uri)
        .await
        .map_err(|e| CirroError::ArmApiError(format!("Failed to query providers: {}", e)))?;

    if providers_response.is_empty() {
        info!(
            "No resource providers found for subscription {}",
            subscription_id
        );
        return Ok(());
    }

    for provider in providers_response {
        let namespace = provider
            .get("namespace")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CirroError::ArmApiError("Namespace not found in provider object".to_string())
            })?;

        let resource_types = provider
            .get("resourceTypes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                CirroError::ArmApiError("Resource types not found in provider object".to_string())
            })?;

        for resource_type in resource_types {
            let type_name = namespace.to_string()
                + "/"
                + resource_type
                    .get("resourceType")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        CirroError::ArmApiError(
                            "Resource type not found in resource type object".to_string(),
                        )
                    })?;
            let type_name = type_name.to_lowercase();

            let api_versions = resource_type.get("apiVersions").ok_or_else(|| {
                CirroError::ArmApiError("API version not found in resource type object".to_string())
            })?;
            let api_versions_array = api_versions.as_array().ok_or_else(|| {
                CirroError::ArmApiError(format!(
                    "API versions is not an array for resource type {}: {}",
                    type_name, api_versions
                ))
            })?;

            if api_versions_array.is_empty() {
                debug!(
                    "No API versions found for resource type {} in subscription {}",
                    type_name, subscription_id
                );
                continue; // Skip this resource type if no API versions are available
            }

            // Concept here is to remove any preview API versions, but if that results in an empty list, fall back to including them with a warning
            // This is because some resource types only have preview API versions, and we don't want to skip them entirely
            // But preview versions aren't everywhere, so we want to prefer non-preview when available
            let preview_filtered_api_versions: Vec<String> = api_versions_array
                .iter()
                .filter_map(Value::as_str)
                .filter(|version| !version.ends_with("-preview"))
                .map(String::from)
                .collect();

            let api_versions_strings = if preview_filtered_api_versions.is_empty() {
                debug!(
                    "No non-preview API versions found for resource type {} in subscription {}; falling back to preview versions",
                    type_name, subscription_id
                );
                api_versions_array
                    .iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            } else {
                preview_filtered_api_versions
            };

            resource_providers.insert(type_name.clone(), api_versions_strings);
        }
    }

    // Enumerate resources for this subscription
    let resourcegroup_uri = format!(
        "subscriptions/{}/resourceGroups?api-version=2021-04-01",
        subscription_id
    );
    let resource_groups = query_resources(collector.clone(), &resourcegroup_uri)
        .await
        .map_err(|e| CirroError::ArmApiError(format!("Failed to query resource groups: {}", e)))?;

    if resource_groups.is_empty() {
        info!(
            "No resource groups found for subscription {}",
            subscription_id
        );
        return Ok(());
    }
    info!(
        "Found {} resource groups for subscription {} ({})",
        resource_groups.len(),
        subscription_id,
        subscription_name
    );

    // Wrap resource_providers in Arc once to share across all resource group tasks
    let resource_providers_arc = Arc::new(resource_providers);

    // Prepare resource group data for concurrent processing
    let mut rg_futures = Vec::with_capacity(resource_groups.len());
    let mut skipped_rgs = 0usize;
    for resource_group in &resource_groups {
        let rg_name = resource_group
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unknown");

        // Check if this resource group was already completed in a previous run
        let rg_state_key = format!("arm:rg:{}/{}", subscription_id, rg_name);
        if is_completed(&resume_state, &rg_state_key) {
            debug!("Skipping resource group {} (already completed)", rg_name);
            skipped_rgs += 1;
            continue;
        }

        let rg_data = Value::Object(resource_group.clone());
        let rg_id = rg_data
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CirroError::ArmApiError(
                    "Resource group ID not found in resource group object".to_string(),
                )
            })?
            .to_string();
        let resource_type = rg_data
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CirroError::ArmApiError(
                    "Resource group type not found in resource group object".to_string(),
                )
            })?
            .to_string();

        let collector_clone = collector.clone();
        let resource_providers_clone = Arc::clone(&resource_providers_arc);
        let rg_state_key_clone = rg_state_key.clone();
        rg_futures.push(async move {
            let _ = collector_clone
                .write_arm_resource_to_db(
                    rg_id.clone().to_lowercase(),
                    format!("/subscriptions/{}", subscription_id.to_lowercase()),
                    rg_id.clone(),
                    resource_type,
                    rg_data,
                )
                .await;
            let _ = enumerate_resourcegroup(
                collector_clone.clone(),
                resource_group,
                subscription_id,
                subscription_name,
                &resource_providers_clone,
            )
            .await;
            // Mark this resource group as completed
            save_arm_state(&collector_clone, rg_state_key_clone, 0, 0.0).await;
        });
    }

    // Execute resource group tasks with bounded concurrency
    if skipped_rgs > 0 {
        info!(
            "Skipped {} already-completed resource groups for subscription {} ({})",
            skipped_rgs, subscription_id, subscription_name
        );
    }
    stream::iter(rg_futures)
        .buffer_unordered(MAX_CONCURRENT_RESOURCE_GROUPS)
        .collect::<Vec<_>>()
        .await;

    info!(
        "ARM subscription {} ({}) enumeration completed",
        subscription_id, subscription_name
    );

    Ok(())
}

async fn enumerate_resourcegroup(
    collector: Arc<Collector>,
    resource_group: &Map<String, Value>,
    subscription_id: &str,
    subscription_name: &str,
    api_versions: &Arc<DashMap<String, Vec<String>>>,
) -> Result<(), CirroError> {
    let rg_id = resource_group
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CirroError::ArmApiError(
                "Resource group ID not found in resource group object".to_string(),
            )
        })?;

    let rg_name = resource_group
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Unknown Resource Group");

    // Get the role assignments for this resource group
    let roles = enumerate_role_assignments(collector.clone(), rg_id).await;
    if roles.is_empty() {
        debug!(
            "No role assignments found for resource group {} in subscription {}",
            rg_name, subscription_name
        );
    } else {
        debug!(
            "Found {} role assignments for resource group {} in subscription {}",
            roles.len(),
            rg_name,
            subscription_name
        );
    }
    // Write the roles to the database
    let mut role_rows = Vec::with_capacity(roles.len());
    for role in roles {
        let role_id = role
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| CirroError::ArmApiError("Role ID not found in role object".to_string()))?
            .to_lowercase();
        role_rows.push((role_id, Value::Object(role)));
    }
    if !role_rows.is_empty() {
        collector
            .write_values_batch_to_db("roleAssignments".into(), role_rows)
            .await?;
    }

    // Enumerate the resources in this resource group
    let resource_uri = format!(
        "{}/resources?api-version=2023-07-01",
        rg_id.trim_end_matches('/')
    );
    let resources = query_resources(collector.clone(), &resource_uri)
        .await
        .map_err(|e| CirroError::ArmApiError(format!("Failed to query resources: {}", e)))?;
    if resources.is_empty() {
        info!(
            "No resources found in resource group {} in subscription {}",
            rg_name, subscription_name
        );
        return Ok(());
    }

    let resource_futures: Vec<_> = resources
        .iter()
        .map(|resource| {
            let collector_clone = collector.clone();
            async move {
                let resource_id = match resource.get("id").and_then(Value::as_str) {
                    Some(id) => id,
                    None => {
                        warn!("Resource missing 'id' field in resource group {}", rg_name);
                        return;
                    }
                };
                let resource_type = match resource.get("type").and_then(Value::as_str) {
                    Some(t) => t,
                    None => {
                        warn!(
                            "Resource {} missing 'type' field in resource group {}",
                            resource_id, rg_name
                        );
                        return;
                    }
                };

                // Iterate over the resource providers and request until we find the API version
                let provider_api_versions = match api_versions
                    .get(&resource_type.to_lowercase())
                    .map(|v| v.clone())
                {
                    Some(v) if !v.is_empty() => v,
                    _ => {
                        warn!(
                            "No API versions found for resource type {} in resource group {}",
                            resource_type, rg_name
                        );
                        return;
                    }
                };

                for api_version in provider_api_versions {
                    let resource_uri = format!("{}?api-version={}", resource_id, api_version);
                    match paged_arm_request(
                        &collector_clone,
                        &resource_uri,
                        reqwest::Method::GET,
                        None,
                    )
                    .await
                    {
                        Err(e) => {
                            if e.to_string().contains("NoRegisteredProviderFound") {
                                continue;
                            } else {
                                warn!("Failed to query resource {}: {}", resource_id, e);
                                break;
                            }
                        }
                        Ok(result) => {
                            if let Err(e) = collector_clone
                                .write_arm_resource_to_db(
                                    resource_id.to_string().to_lowercase(),
                                    subscription_id.to_string().to_lowercase(),
                                    rg_id.to_string().to_lowercase(),
                                    resource_type.to_string(),
                                    result,
                                )
                                .await
                            {
                                warn!(
                                    "Failed to write resource {} to database: {}",
                                    resource_id, e
                                );
                            }
                            break; // Break after the first successful API version
                        }
                    }
                }
            }
        })
        .collect();
    // Execute resource tasks with bounded concurrency
    stream::iter(resource_futures)
        .buffer_unordered(MAX_CONCURRENT_RESOURCES)
        .collect::<Vec<_>>()
        .await;

    info!(
        "Enumerating resource group: {} - {}",
        subscription_name, rg_name
    );

    Ok(())
}

pub async fn enumerate_arm(
    collector: Arc<Collector>,
    resume_state: HashMap<String, (String, usize, f64)>,
) -> Result<(), CirroError> {
    info!("Starting ARM enumeration");

    // Start timer
    let start_time = std::time::Instant::now();

    // First we need to get the tenants
    if is_completed(&resume_state, "arm:tenants") {
        info!("Skipping tenants (already collected)");
    } else {
        let tenants = query_resources(collector.clone(), "tenants?api-version=2022-12-01").await?;
        if tenants.is_empty() {
            return Err(CirroError::ArmApiError("No tenants found".to_string()));
        }
        info!("Found {} tenants", tenants.len());
        debug!("Tenants: {:?}", tenants);

        // Write the tenants to the database
        let mut tenant_rows = Vec::with_capacity(tenants.len());
        for tenant in &tenants {
            let tenant_id = tenant.get("id").and_then(Value::as_str).ok_or_else(|| {
                CirroError::ArmApiError("Tenant ID not found in tenant object".to_string())
            })?;

            let tenant_data = serde_json::to_value(tenant.clone())
                .map_err(|e| CirroError::SerializationError(e.to_string()))?;

            tenant_rows.push((tenant_id.to_lowercase(), tenant_data));
        }
        if !tenant_rows.is_empty() {
            collector
                .write_values_batch_to_db("tenants".into(), tenant_rows)
                .await?;
        }
        save_arm_state(
            &collector,
            "arm:tenants".to_string(),
            tenants.len(),
            start_time.elapsed().as_secs_f64(),
        )
        .await;
    }

    // Management groups
    if is_completed(&resume_state, "arm:management_groups") {
        info!("Skipping management groups (already collected)");
    } else {
        // If unable to get management groups, return an empty vector
        let mg_entities = enumerate_management_groups(collector.clone())
            .await
            .unwrap_or_else(|e| {
                warn!("Failed to enumerate management groups: {}", e);
                Vec::new()
            });
        info!("Found {} management group entities", mg_entities.len());
        debug!("Management Groups entities: {:?}", mg_entities);

        // Write the management groups entities to the database
        let mut management_rows = Vec::with_capacity(mg_entities.len());
        for entity in &mg_entities {
            let entity_id = entity.get("id").and_then(Value::as_str).ok_or_else(|| {
                CirroError::ArmApiError("Management group ID not found in object".to_string())
            })?;
            let entity_data = serde_json::to_value(entity.clone())
                .map_err(|e| CirroError::SerializationError(e.to_string()))?;
            management_rows.push((entity_id.to_lowercase(), entity_data));
        }
        if !management_rows.is_empty() {
            collector
                .write_values_batch_to_db("managementGroupEntities".into(), management_rows)
                .await?;
        }
        save_arm_state(
            &collector,
            "arm:management_groups".to_string(),
            mg_entities.len(),
            start_time.elapsed().as_secs_f64(),
        )
        .await;
    }

    // Get the subscriptions
    let subscriptions =
        query_resources(collector.clone(), "/subscriptions/?api-version=2024-08-01").await?;
    if subscriptions.is_empty() {
        return Err(CirroError::ArmApiError(
            "No subscriptions found".to_string(),
        ));
    }
    info!("Found {} subscriptions", subscriptions.len());
    debug!("Subscriptions: {:?}", subscriptions);

    // Apply subscription filter if specified
    let subscriptions = if let Some(ref filter) = collector.subscription_filter {
        let filtered: Vec<_> = subscriptions
            .into_iter()
            .filter(|s| {
                s.get("subscriptionId")
                    .and_then(Value::as_str)
                    .map_or(false, |id| filter.contains(id))
            })
            .collect();
        info!(
            "Filtered to {} subscriptions matching --subscription-ids",
            filtered.len()
        );
        if filtered.is_empty() {
            return Err(CirroError::ArmApiError(
                "No subscriptions matched the provided --subscription-ids filter".to_string(),
            ));
        }
        filtered
    } else {
        subscriptions
    };

    // Write the subscriptions to the database
    let mut subscription_rows = Vec::with_capacity(subscriptions.len());
    for subscription in &subscriptions {
        let subscription_id = subscription
            .get("subscriptionId")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CirroError::ArmApiError(
                    "Subscription ID not found in subscription object".to_string(),
                )
            })?;
        let subscription_data = serde_json::to_value(subscription.clone())
            .map_err(|e| CirroError::SerializationError(e.to_string()))?;
        subscription_rows.push((subscription_id.to_lowercase(), subscription_data));
    }
    if !subscription_rows.is_empty() {
        collector
            .write_values_batch_to_db("subscriptions".into(), subscription_rows)
            .await?;
    }

    // Now we can query resources for each subscription
    // Run subscriptions through a bounded pool to avoid unbounded task fan-out.
    let resume_state_arc = Arc::new(resume_state);
    let subscription_concurrency = collector.subscription_concurrency.max(1);
    let mut subscription_futures = Vec::new();
    let mut skipped_subs = 0usize;
    for subscription in &subscriptions {
        let sub_id = subscription
            .get("subscriptionId")
            .and_then(Value::as_str)
            .unwrap_or("unknown");

        // Check if this subscription was already fully completed
        let sub_state_key = format!("arm:sub:{}", sub_id);
        if is_completed(&resume_state_arc, &sub_state_key) {
            info!("Skipping subscription {} (already completed)", sub_id);
            skipped_subs += 1;
            continue;
        }

        let collector_clone = collector.clone();
        let subscription_clone = subscription.clone();
        let resume_state_clone = Arc::clone(&resume_state_arc);
        let sub_state_key = sub_state_key.clone();
        subscription_futures.push(async move {
            let result = enumerate_subscription(
                collector_clone.clone(),
                &subscription_clone,
                resume_state_clone,
            )
            .await;
            match result {
                Ok(()) => {
                    // Mark subscription as fully completed
                    save_arm_state(&collector_clone, sub_state_key, 0, 0.0).await;
                }
                Err(e) => {
                    warn!(
                        "Failed to enumerate subscription {}: {}",
                        subscription_clone
                            .get("subscriptionId")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown"),
                        e
                    );
                }
            }
        });
    }
    if skipped_subs > 0 {
        info!("Skipped {} already-completed subscriptions", skipped_subs);
    }
    if !subscription_futures.is_empty() {
        info!(
            "Enumerating {} subscriptions with concurrency {}",
            subscription_futures.len(),
            subscription_concurrency
        );
        stream::iter(subscription_futures)
            .buffer_unordered(subscription_concurrency)
            .collect::<Vec<_>>()
            .await;
    }

    // If arm_pim flag is set, also gather eligible role assignments for the current user
    if collector.option_enum_flags.arm_pim {
        info!("Gathering eligible ARM role assignments for current user");
        let eligible_roles = query_resources(
            collector.clone(),
            "providers/Microsoft.Authorization/roleEligibilityScheduleInstances?api-version=2020-10-01&$filter=asTarget()",
        )
        .await?;
        if eligible_roles.is_empty() {
            info!("No eligible ARM role assignments found for current user");
        } else {
            info!(
                "Found {} eligible ARM role assignments for current user",
                eligible_roles.len()
            );
            debug!("Eligible ARM role assignments: {:?}", eligible_roles);

            // Write the eligible roles directly to the database
            // It will already be in JSON format so we can just convert it to a Value
            let mut eligible_role_rows = Vec::with_capacity(eligible_roles.len());
            for role in &eligible_roles {
                let role_id = role.get("id").and_then(Value::as_str).ok_or_else(|| {
                    CirroError::ArmApiError("Role ID not found in eligible role object".to_string())
                })?;
                let role_data = serde_json::to_value(role.clone())
                    .map_err(|e| CirroError::SerializationError(e.to_string()))?;
                eligible_role_rows.push((role_id.to_lowercase(), role_data));
            }
            if !eligible_role_rows.is_empty() {
                collector
                    .write_values_batch_to_db("eligibleArmRBAC".into(), eligible_role_rows)
                    .await?;
            }
        }
    }

    // Clear all arm:* state keys on full completion
    for key in resume_state_arc.keys() {
        clear_arm_state(&collector, key.clone()).await;
    }
    // Also clear the phase markers we just set
    clear_arm_state(&collector, "arm:tenants".to_string()).await;
    clear_arm_state(&collector, "arm:management_groups".to_string()).await;
    for subscription in &subscriptions {
        let sub_id = subscription
            .get("subscriptionId")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        clear_arm_state(&collector, format!("arm:sub:{}", sub_id)).await;
    }

    info!(
        "ARM enumeration completed in {} seconds",
        fmt_duration(start_time.elapsed())
    );

    Ok(())
}
