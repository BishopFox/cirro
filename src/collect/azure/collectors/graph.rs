use super::common::*;
use crate::collect::azure::collect::Collector;
use crate::collect::azure::credentials::common::Token;
use crate::errors::CirroError;

use futures::stream::{self, StreamExt};
use log::{debug, error, info};
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use std::{cmp, collections::HashMap, vec};

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

// Rate limit logging flag to prevent log spam when multiple tasks hit rate limits at the same time
static RATE_LIMIT_LOGGED: AtomicBool = AtomicBool::new(false);

/// Represents a Graph object with its resource type, query parameters, and optional expand properties
#[derive(Clone)]
struct GraphObject {
    /// The name of the resource type to query in the Graph API (e.g., "users", "groups", "applications")
    /// This is used for logging and to determine the database table name when writing results.
    name: String,

    /// The type of resource in the Graph API (e.g., "users", "groups")
    /// This is used to construct the URI for the Graph API request.
    uri: String,

    /// The query parameters to be used in the Graph API request
    /// This typically includes parameters like `$top`, `$filter`, etc., to control the data returned.
    /// For example, "$top=999" to limit the results to 999 items
    query_params: String,

    /// Optional properties to expand in the Graph API response
    /// This is a map where the key is the property to expand (e.g., "members", "owners") and the value is a vector of strings
    /// The vector represents the specific fields to include in the expanded response in JSON pointer format.
    expand_properties: HashMap<String, Vec<String>>,
}

impl GraphObject {
    pub fn new<R, Q>(
        name: R,
        uri: R,
        query_params: Q,
        expand_properties: Option<HashMap<String, Vec<String>>>,
    ) -> Self
    where
        R: Into<String>,
        Q: Into<String>,
    {
        GraphObject {
            name: name.into(),
            uri: uri.into(),
            query_params: query_params.into(),
            expand_properties: expand_properties.unwrap_or_default(),
        }
    }
}

/// Enumerates Microsoft Graph data
/// This function retrieves various objects from the Microsoft Graph API
pub async fn enumerate_graph(collector: Arc<Collector>) -> Result<(), CirroError> {
    info!("Starting Graph enumeration");

    // Initialize our token cache with a valid token
    {
        let mut cache = TOKEN_CACHE.lock().await;
        if cache.is_none() || cache.as_ref().unwrap().is_expired_or_not_set() {
            debug!("Pre-fetching initial Graph API token");
            *cache = Some(collector.msgraph_credential.get_token().await?);
            info!("MS Graph token retrieved successfully");
        }
    }

    // Create all enumerators with their configurations
    let mut enumerators = vec![
        GraphObject::new("organization", "organization", "", None),
        GraphObject::new(
            "authorizationPolicy",
            "policies/authorizationPolicy",
            "",
            None,
        ),
        GraphObject::new(
            "users",
            "users",
            "$top=999",
            Some(HashMap::from([("memberOf".into(), vec!["/id".into()])])),
        ),
        GraphObject::new(
            "groups",
            "groups",
            "$top=999",
            Some(HashMap::from([
                ("members".into(), vec!["/id".into()]),
                ("owners".into(), vec!["/id".into()]),
                ("memberOf".into(), vec!["/id".into()]),
            ])),
        ),
        GraphObject::new(
            "applications",
            "applications",
            "$top=999",
            Some(HashMap::from([
                ("owners".into(), vec!["/id".into()]),
                (
                    "federatedIdentityCredentials".into(),
                    vec![
                        "/id".into(),
                        "/name".into(),
                        "/description".into(),
                        "/issuer".into(),
                        "/audiences".into(),
                    ],
                ),
            ])),
        ),
        GraphObject::new(
            "servicePrincipals",
            "servicePrincipals",
            "$top=999",
            Some(HashMap::from([
                ("owners".into(), vec!["/id".into()]),
                ("memberOf".into(), vec!["/id".into()]),
                (
                    "appRoleAssignedTo".into(),
                    vec!["/principalId".into(), "/appRoleId".into()],
                ),
                (
                    "endpoints".into(),
                    vec![
                        "/id".into(),
                        "/capability".into(),
                        "/providerName".into(),
                        "/providerResourceId".into(),
                        "/uri".into(),
                    ],
                ),
            ])),
        ),
        GraphObject::new(
            "devices",
            "devices",
            "$top=999",
            Some(HashMap::from([
                ("registeredOwners".into(), vec!["/id".into()]),
                ("registeredUsers".into(), vec!["/id".into()]),
                ("memberOf".into(), vec!["/id".into()]),
            ])),
        ),
        GraphObject::new(
            "directoryRoles",
            "directoryRoles",
            "",
            Some(HashMap::from([("members".into(), vec!["/id".into()])])),
        ),
        GraphObject::new(
            "administrativeUnits",
            "administrativeUnits",
            "",
            Some(HashMap::from([
                ("members".into(), vec!["/id".into()]),
                (
                    "scopedRoleMembers".into(),
                    vec!["/roleId".into(), "/roleMemberInfo/id".into()],
                ),
            ])),
        ),
    ];

    if let Some(filters) = collector.option_enum_flags.graph_object_filters() {
        enumerators.retain(|obj| filters.contains(&obj.name));
    }

    // Optional flag enumerators based on user input
    if collector.option_enum_flags.graph_pim {
        enumerators.push(GraphObject::new(
            "eligibleRoleAssignments",
            "/roleManagement/directory/roleEligibilitySchedules",
            "$top=999",
            None,
        ));
    }

    if collector.option_enum_flags.caps {
        enumerators.push(GraphObject::new(
            "conditionalAccessPolicies",
            "/identity/conditionalAccess/policies",
            "",
            None,
        ));

        enumerators.push(GraphObject::new(
            "namedLocations",
            "/identity/conditionalAccess/namedLocations",
            "",
            None,
        ));
    }

    // Match concurrency to the requested enumerators (minimum 1 to satisfy the stream API)
    let concurrency = cmp::max(1, enumerators.len());

    // Start a timer
    let start_time = Instant::now();

    // Use a concurrent stream with controlled concurrency for better resource usage
    let results = stream::iter(enumerators)
        .map(|enumerator| {
            let collector_clone = Arc::clone(&collector);
            async move {
                match query_objects(enumerator, collector_clone).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Error collecting Graph data: {}", e);
                        Err(e)
                    }
                }
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    // Log the total time taken for the enumeration
    info!(
        "Graph enumeration completed in {} seconds",
        fmt_duration(start_time.elapsed())
    );
    // Check for errors
    let errors: Vec<_> = results.into_iter().filter_map(|r| r.err()).collect();
    if !errors.is_empty() {
        error!("Graph enumeration completed with {} errors", errors.len());
    }

    Ok(())
}

/// Queries objects from the Microsoft Graph API
/// This function simulates querying objects from the Graph API and sending results to the database
async fn query_objects(
    graph_object: GraphObject,
    collector: Arc<Collector>,
) -> Result<(), CirroError> {
    // Start timing the query
    let start_time = std::time::Instant::now();
    let uri_path = &graph_object.uri;
    let graph_object_name = &graph_object.name;

    info!(
        "Querying Graph API for resource type: {}",
        graph_object_name
    );

    let mut next_uri = format!("{}?{}", uri_path, graph_object.query_params);

    // Use the global HTTP client instead of creating a new one for each request
    let mut all_values = Vec::new();
    let mut total_fetched = 0;

    // Batch process the pages
    loop {
        debug!("Fetching data from: {}", next_uri);

        // Get the next page of data using our shared client
        let response_data = paged_graph_request(&collector, &next_uri)
            .await
            .map_err(|e| {
                error!("Failed to fetch data from Graph API: {} - {}", &next_uri, e);
                e // Since paged_graph_request already returns CirroError, we can just pass it through
            })?;

        // Check for errors in the response
        if let Some(error) = response_data.get("@odata.error") {
            let error_message = error
                .get("message")
                .and_then(|m| m.get("value"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            error!("Graph API error: {}", error_message);
            return Err(CirroError::ODataError(error_message.to_string()));
        }

        // Get the values - avoid unnecessary clone when possible
        if let Some(values) = response_data.get("value").and_then(|v| v.as_array()) {
            let page_count = values.len();
            total_fetched += page_count;
            debug!(
                "Fetched {} objects for {}, total: {}",
                page_count, graph_object_name, total_fetched
            );

            // Collect values for batch processing
            all_values.extend(values.iter().cloned());
        }

        // Check for next link
        if let Some(next_link) = response_data
            .get("@odata.nextLink")
            .and_then(|v| v.as_str())
        {
            next_uri = next_link.to_string();
        } else {
            break; // No more pages to fetch
        }
    }

    info!(
        "Fetched {} objects for {}. Processing...",
        total_fetched, graph_object_name
    );

    // Process the values in parallel with controlled concurrency
    let expand_properties = &graph_object.expand_properties;
    let need_expand = !expand_properties.is_empty();

    // If we need to expand properties, use a semaphore to limit concurrent requests
    // let concurrency = if need_expand { 20 } else { 50 };
    let concurrency = 100;
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

    // Create a stream for processing the values with controlled concurrency
    let results = stream::iter(all_values)
        .map(|value| {
            let collector_clone = Arc::clone(&collector);
            let resource_type = uri_path.clone();
            let expand_properties = expand_properties.clone();
            let sem = Arc::clone(&semaphore);

            async move {
                // Acquire permit from semaphore to control concurrency
                let _permit = sem.acquire().await.map_err(|e| {
                    error!("Failed to acquire semaphore: {}", e);
                    CirroError::SemaphoreError(e.to_string())
                })?;

                // Make a mutable copy of the value for expansion
                let mut value_clone = value.clone();

                // Expand properties if specified
                if need_expand {
                    // Expand each property
                    for (property, json_pointer) in &expand_properties {
                        expand_object(
                            &collector_clone,
                            &resource_type,
                            &mut value_clone,
                            property,
                            json_pointer,
                        )
                        .await;
                    }
                }

                let id: String;

                if uri_path.starts_with("policies/") {
                    // Special handling for policies
                    // Get the ID from the object
                    let policy_type = value_clone
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown_id")
                        .to_string();

                    let tenant_id = collector_clone
                        .msgraph_credential
                        .as_ref()
                        .get_token()
                        .await?
                        .get_claims()?
                        .tid
                        .unwrap_or_else(|| "unknown_tenant".to_string());

                    id = format!("{}_{}", tenant_id, policy_type);
                } else {
                    // Get the ID from the object
                    id = value_clone
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown_id")
                        .to_string();
                };

                Ok::<(String, serde_json::Value), CirroError>((id, value_clone))
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<Result<(String, serde_json::Value), CirroError>>>()
        .await;

    let mut rows: Vec<(String, serde_json::Value)> = Vec::new();
    let mut error_count = 0usize;

    for result in results {
        match result {
            Ok((id, value)) => rows.push((id, value)),
            Err(_) => error_count += 1,
        }
    }

    if !rows.is_empty() {
        let table = if uri_path.starts_with("policies/") {
            "policies".to_string()
        } else {
            graph_object_name.to_string()
        };

        collector
            .write_values_batch_to_db(table, rows)
            .await
            .map_err(|e| {
                error!(
                    "Failed to write batch for resource type {}: {}",
                    graph_object_name, e
                );
                e
            })?;
    }

    // Check for errors in processing results
    if error_count > 0 {
        error!(
            "{} errors occurred while processing {} objects",
            error_count, graph_object_name
        );
    }

    let elapsed_time = start_time.elapsed();
    info!(
        "Completed Graph API query for resource type: {} ({}) - {} objects processed",
        graph_object_name,
        fmt_duration(elapsed_time),
        total_fetched
    );

    Ok(())
}

/// Expand the properties of a Graph object
/// This function expands the properties of a Graph object based on the provided expand properties
/// The object is modified in place, but no database write occurs in this function.
pub async fn expand_object(
    collector: &Collector,
    resource_type: &str,
    object: &mut serde_json::Value,
    property: &str,
    json_pointer: &Vec<String>,
) {
    // Early return if we don't have a valid ID to work with
    // This should never happen, but it's a good safety check
    let object_id = {
        let id = match object.get("id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                error!(
                    "Cannot expand object without ID for {}/{}",
                    resource_type, property
                );
                return;
            }
        };
        id.to_string()
    };

    // Create the initial URL
    let mut next_url = format!(
        "{}/beta/{}/{}/{}",
        collector.cloud_endpoints.msgraph_url, resource_type, object_id, property
    );

    debug!(
        "Expanding property {} for {}/{}",
        property, resource_type, object_id
    );

    // Prepare to collect all expanded values
    let mut all_expanded_values = Vec::new();

    // Use the global HTTP client for better connection reuse
    loop {
        // Get the response data, with error handling
        let response_data = match paged_graph_request(collector, &next_url).await {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Failed to fetch expanded data for {}/{}/{}: {}",
                    resource_type, object_id, property, e
                );
                break;
            }
        };

        // Process the values if available
        if let Some(values) = response_data.get("value").and_then(|v| v.as_array()) {
            // Only extract values if there are any
            if !values.is_empty() {
                // Extract only the specific fields based on json_pointer
                for value in values {
                    // If there is only one value, use it directly in an array
                    // Otherwise, push a map into all_expanded_values
                    if json_pointer.len() == 1 {
                        if let Some(expanded_value) = value.pointer(&json_pointer[0]) {
                            debug!(
                                "Expanded value for {}/{}/{}: {}",
                                resource_type, object_id, property, expanded_value
                            );
                            // Clone the value to avoid borrowing issues
                            all_expanded_values.push(expanded_value.clone());
                        }
                    } else {
                        let mut expanded_map = serde_json::Map::new();
                        for pointer in json_pointer {
                            if let Some(expanded_value) = value.pointer(pointer) {
                                debug!(
                                    "Expanded value for {}/{}/{}: {}",
                                    resource_type, object_id, property, expanded_value
                                );
                                expanded_map.insert(
                                    pointer.trim_start_matches('/').to_string(),
                                    expanded_value.clone(),
                                );
                            }
                        }
                        all_expanded_values.push(serde_json::Value::Object(expanded_map));
                    }
                }
            }
        }

        // Check for next link - using get() instead of direct indexing to avoid panics
        if let Some(next_link) = response_data
            .get("@odata.nextLink")
            .and_then(|v| v.as_str())
        {
            next_url = next_link.to_string();
        } else {
            break; // No more pages to fetch
        }
    }

    // Only update the object if we have any values to add
    if !all_expanded_values.is_empty() {
        // Update the object with all expanded values at once
        if let Some(obj_mut) = object.as_object_mut() {
            // Create or get the array for this property
            let entry = obj_mut
                .entry(property)
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));

            // Add all the expanded values to the array
            if let Some(array) = entry.as_array_mut() {
                array.extend(all_expanded_values);
                debug!(
                    "Added {} expanded values for {}/{}/{}",
                    array.len(),
                    resource_type,
                    object_id,
                    property
                );
            }
        }
    }
}

/// Performs a paged request to the Graph API with optimized token caching
pub async fn paged_graph_request(
    collector: &Collector,
    uri: &str,
) -> Result<serde_json::Value, CirroError> {
    // Build the full URL correctly
    let graph_url = if uri.starts_with(&collector.cloud_endpoints.msgraph_url) {
        uri.to_owned()
    } else {
        format!(
            "{}/beta/{}",
            collector.cloud_endpoints.msgraph_url,
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
            debug!("Fetching new Graph API token");
            let token = collector.msgraph_credential.get_token().await?;

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
        let result = HTTP_CLIENT
            .get(&graph_url)
            .bearer_auth(&*token_str)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;

        let response = match result {
            Ok(resp) => resp,
            Err(e) => {
                // Handle request errors, which may include timeouts or connection issues
                // Sometimes there will be an IO timeout around the throttling limits so need to retry in a little bit
                if retries >= max_retries {
                    return Err(CirroError::IoError(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!(
                            "Request to {} failed after {} retries: {}",
                            graph_url, retries, e
                        ),
                    )));
                }

                debug!(
                    "Request timeout for {}, retrying after 5 seconds (attempt {})",
                    graph_url,
                    retries + 1
                );
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                retries += 1;
                continue; // Retry the request after waiting
            }
        };

        let status = response.status();

        // Check for rate limiting first
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
                info!("Graph API rate limit hit, all tasks will back off for 25 seconds");

                // The task that logs the message also starts a timer to reset the flag
                // Should be slightly longer than the backoff and other tasks will not log
                tokio::spawn(async {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    RATE_LIMIT_LOGGED.store(false, Ordering::Relaxed);
                });
            }

            // Determined from documented throttling guidance
            // https://learn.microsoft.com/en-us/graph/throttling-limits#identity-and-access-service-limits
            tokio::time::sleep(std::time::Duration::from_secs(25)).await;

            retries += 1;
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
