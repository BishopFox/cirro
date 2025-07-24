use crate::collect::Collector;
use crate::collectors::common::*;
use crate::errors::CirroError;

use futures::stream::{self, StreamExt};
use log::{debug, error, info};
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use std::{collections::HashMap, vec};

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
static TOKEN_CACHE: Lazy<tokio::sync::Mutex<Option<crate::credentials::common::Token>>> =
    Lazy::new(|| tokio::sync::Mutex::new(None));

/// Represents a Graph object with its resource type, query parameters, and optional expand properties
#[derive(Clone)]
struct GraphObject {
    /// The type of resource in the Graph API (e.g., "users", "groups")
    /// This is used to specify which endpoint to query in the Microsoft Graph API.
    resource_type: String,

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
        resource_type: R,
        query_params: Q,
        expand_properties: Option<HashMap<String, Vec<String>>>,
    ) -> Self
    where
        R: Into<String>,
        Q: Into<String>,
    {
        GraphObject {
            resource_type: resource_type.into(),
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
    let enumerators = vec![
        GraphObject::new("users", "$top=999", None),
        GraphObject::new(
            "groups",
            "$top=999",
            Some(HashMap::from([
                ("members".into(), vec!["/id".into()]),
                ("owners".into(), vec!["/id".into()]),
            ])),
        ),
        GraphObject::new(
            "applications",
            "$top=999",
            Some(HashMap::from([("owners".into(), vec!["/id".into()])])),
        ),
        GraphObject::new(
            "servicePrincipals",
            "$top=999",
            Some(HashMap::from([("owners".into(), vec!["/id".into()])])),
        ),
        GraphObject::new(
            "devices",
            "$top=999",
            Some(HashMap::from([
                ("registeredOwners".into(), vec!["/id".into()]),
                ("registeredUsers".into(), vec!["/id".into()]),
            ])),
        ),
        GraphObject::new(
            "directoryRoles",
            "",
            Some(HashMap::from([("members".into(), vec!["/id".into()])])),
        ),
        GraphObject::new(
            "administrativeUnits",
            "",
            Some(HashMap::from([
                ("members".into(), vec!["/id".into()]),
                (
                    "scopedRoleMembers".into(),
                    vec!["roleId".into(), "/roleMemberInfo/id".into()],
                ),
            ])),
        ),
    ];

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
        .buffer_unordered(7) // Use the number of enumerators for concurrency
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
        return Err(CirroError::MultipleErrors(errors));
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
    let resource_type = &graph_object.resource_type;

    info!("Querying Graph API for resource type: {}", resource_type);

    let mut next_uri = format!("{}?{}", resource_type, graph_object.query_params);

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
                page_count, resource_type, total_fetched
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
        total_fetched, resource_type
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
            let resource_type = resource_type.clone();
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

                // Get the ID from the object
                let id = value_clone
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown_id")
                    .to_string();

                // Prepare and send the DB write message - this happens regardless of whether
                // properties were expanded, but we have better logging if they were
                let _ = &collector_clone
                    .write_value_to_db(resource_type.clone(), id.clone(), value_clone)
                    .await;

                Ok(())
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<Result<(), CirroError>>>()
        .await;

    // Check for errors in the results
    let error_count = results.iter().filter(|r| r.is_err()).count();
    if error_count > 0 {
        error!(
            "{} errors occurred while processing {} objects",
            error_count, resource_type
        );
    }

    let elapsed_time = start_time.elapsed();
    info!(
        "Completed Graph API query for resource type: {} ({}) - {} objects processed",
        resource_type,
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
                    for pointer in json_pointer {
                        // Use the pointer to get the value (typically "/id" for IDs)
                        if let Some(expanded_value) = value.pointer(pointer) {
                            debug!(
                                "Expanded value for {}/{}/{}: {}",
                                resource_type, object_id, property, expanded_value
                            );
                            // Clone the value to avoid borrowing issues
                            all_expanded_values.push(expanded_value.clone());
                        }
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

    // Make the request using the global client
    let response = HTTP_CLIENT
        .get(&graph_url)
        .bearer_auth(&*token_str)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                CirroError::IoError(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("Request timed out: {}", e),
                ))
            } else {
                CirroError::RequestError(e)
            }
        })?;

    let status = response.status();

    // Check for non-200 status codes early to avoid parsing JSON for error responses
    if status != StatusCode::OK {
        let error_text = response.text().await?;
        return Err(CirroError::HttpError(format!(
            "HTTP {} - {}",
            status.as_u16(),
            error_text
        )));
    }

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
