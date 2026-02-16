use super::common::*;
use crate::collect::azure::collect::Collector;
use crate::collect::azure::credentials::common::Token;
use crate::errors::CirroError;

use dashmap::DashMap;
use log::warn;
use log::{debug, info};
use once_cell::sync::Lazy;
use reqwest::Client;
use reqwest::StatusCode;
use serde_json::Map;
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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

// Rate limit logging flag to prevent log spam when multiple tasks hit rate limits at the same time
static RATE_LIMIT_LOGGED: AtomicBool = AtomicBool::new(false);

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
            info!("Fetching next page: {}", next_url);
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

        let response = match result {
            Ok(resp) => resp,
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
                continue; // Retry the request after waiting
            }
        };

        // let response = result.map_err(|e| {
        //     CirroError::HttpError(format!(
        //         "Failed to send request to {}: {:?} - {}",
        //         arm_url,
        //         e.status(),
        //         e
        //     ))
        // })?;

        let status = response.status();

        if status.is_success() {
            break response;
        } else if status == StatusCode::TOO_MANY_REQUESTS {
            // Handle rate limiting by checking for Too Many Requests status
            if retries >= max_retries {
                return Err(CirroError::HttpError(
                    "Too many requests, exceeded max retries".to_string(),
                ));
            }

            // Only log the first time any task hits the rate limit
            if !RATE_LIMIT_LOGGED.swap(true, Ordering::Relaxed) {
                info!("ARM API rate limit hit, all tasks will back off for 25 seconds");

                // The task that logs the message also starts a timer to reset the flag
                // Should be slightly longer than the backoff and other tasks will not log
                tokio::spawn(async {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    RATE_LIMIT_LOGGED.store(false, Ordering::Relaxed);
                });
            }

            tokio::time::sleep(std::time::Duration::from_secs(25)).await;
            continue; // Retry the request after waiting
        } else {
            let error_text = response.text().await?;
            return Err(CirroError::HttpError(format!(
                "HTTP {} - {}",
                status.as_u16(),
                error_text
            )));
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
                let mut assignment_with_definition = assignment.clone();
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
        "providers/Microsoft.Management/getEntities?api-version=2021-04-01&$top=999".to_string();
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
    for role in &roles {
        let role_id = role.get("id").and_then(Value::as_str).ok_or_else(|| {
            CirroError::ArmApiError("Role ID not found in role object".to_string())
        })?;
        let role_data = serde_json::to_value(role.clone())
            .map_err(|e| CirroError::SerializationError(e.to_string()))?;
        role_rows.push((role_id.to_lowercase(), role_data));
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

            // Convert JSON values to Strings
            let api_versions_strings: Vec<String> = api_versions_array
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();

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

    let mut tasks = Vec::new();
    // Write the resource groups to the database
    for resource_group in &resource_groups {
        let rg_data = serde_json::to_value(resource_group.clone())
            .map_err(|e| CirroError::SerializationError(e.to_string()))?;
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
        let resource_providers_clone = Arc::new(resource_providers.clone());
        tasks.push(async move {
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
                collector_clone,
                resource_group,
                subscription_id,
                subscription_name,
                &resource_providers_clone,
            )
            .await;
        });
    }

    // Execute all tasks concurrently
    futures::future::join_all(tasks).await;

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
    for role in &roles {
        let role_id = role.get("id").and_then(Value::as_str).ok_or_else(|| {
            CirroError::ArmApiError("Role ID not found in role object".to_string())
        })?;
        let role_data = serde_json::to_value(role.clone())
            .map_err(|e| CirroError::SerializationError(e.to_string()))?;
        role_rows.push((role_id.to_lowercase(), role_data));
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

    let mut tasks = Vec::new();
    for resource in &resources {
        let collector_clone = collector.clone();
        tasks.push(async move {
            let resource_id = resource.get("id").and_then(Value::as_str).unwrap();
            let resource_type = resource.get("type").and_then(Value::as_str).unwrap();

            // Iterate over the resource providers and request until we find the API version
            let provider_api_versions = api_versions
                .get(&resource_type.to_lowercase())
                .map(|v| v.clone())
                .unwrap();

            if provider_api_versions.is_empty() {
                warn!(
                    "No API versions found for resource type {} in resource group {}",
                    resource_type, rg_name
                );
                return;
            }

            for api_version in provider_api_versions {
                let resource_uri = format!("{}?api-version={}", resource_id, api_version);
                let result =
                    paged_arm_request(&collector_clone, &resource_uri, reqwest::Method::GET, None)
                        .await;
                if let Err(e) = result {
                    if e.to_string().contains("NoRegisteredProviderFound") {
                        continue;
                    } else {
                        warn!("Failed to query resource {}: {}", resource_id, e);
                        break; // Break on any error other than NoRegisteredProviderFound
                    }
                } else {
                    let result = result.unwrap();
                    let value_clone = serde_json::to_value(result)
                        .map_err(|e| CirroError::SerializationError(e.to_string()))
                        .unwrap();

                    // Write the resource to the database
                    let _ = collector_clone
                        .write_arm_resource_to_db(
                            resource_id.to_string().to_lowercase(),
                            subscription_id.to_string().to_lowercase(),
                            rg_id.to_string().to_lowercase(),
                            resource_type.to_string(),
                            value_clone,
                        )
                        .await
                        .map_err(|e| CirroError::DatabaseError(e.to_string()))
                        .unwrap();
                    break; // Break after the first successful API version
                }
            }
        });
    }
    // Execute all tasks concurrently
    futures::future::join_all(tasks).await;

    info!(
        "Enumerating resource group: {} - {}",
        subscription_name, rg_name
    );

    Ok(())
}

pub async fn enumerate_arm(collector: Arc<Collector>) -> Result<(), CirroError> {
    info!("Starting ARM enumeration");

    // Start timer
    let start_time = std::time::Instant::now();

    // First we need to get the tenants
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

    // Management groups
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
    // Call enumerate_subscriptions as tasks
    let mut tasks = Vec::new();
    for subscription in &subscriptions {
        let collector_clone = collector.clone();
        let subscription_clone = subscription.clone();
        tasks.push(tokio::spawn(async move {
            let result = enumerate_subscription(collector_clone, &subscription_clone).await;
            if let Err(e) = result {
                warn!(
                    "Failed to enumerate subscription {}: {}",
                    subscription_clone
                        .get("subscriptionId")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown"),
                    e
                );
            }
        }));
    }
    if !tasks.is_empty() {
        futures::future::join_all(tasks).await;
    }

    info!(
        "ARM enumeration completed in {} seconds",
        fmt_duration(start_time.elapsed())
    );

    Ok(())
}
