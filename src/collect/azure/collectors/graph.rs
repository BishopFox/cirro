use super::common::*;
use crate::collect::azure::collect::Collector;
use crate::collect::azure::credentials::common::Token;
use crate::collect::azure::db::DBWriteMessage;
use crate::errors::CirroError;

use futures::stream::{self, StreamExt};
use log::{debug, error, info, warn};
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
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(120))
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .pool_max_idle_per_host(20)
        .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
        .gzip(true)
        .build()
        .unwrap()
});

// Global static cache for the token
static TOKEN_CACHE: Lazy<tokio::sync::Mutex<Option<Token>>> =
    Lazy::new(|| tokio::sync::Mutex::new(None));

// Rate limit logging flag to prevent log spam when multiple tasks hit rate limits at the same time
static RATE_LIMIT_LOGGED: AtomicBool = AtomicBool::new(false);

/// Maximum number of individual requests in a single Graph $batch call.
const BATCH_MAX_REQUESTS: usize = 20;

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
    /// For example, "$top=999" to limit the results to 999 items.
    query_params: String,

    /// A single navigation property that should always be expanded for each object.
    /// The key is the property name (e.g. "members"), the value is the JSON pointer
    /// fields to extract (e.g. ["/id"]).
    inline_expand: Option<(String, Vec<String>)>,

    /// Properties that must always be fetched via individual per-object requests
    /// (or batched). Used for additional navigation properties beyond `inline_expand`.
    per_object_expand: HashMap<String, Vec<String>>,
}

impl GraphObject {
    pub fn new<R, Q>(
        name: R,
        uri: R,
        query_params: Q,
        inline_expand: Option<(String, Vec<String>)>,
        per_object_expand: Option<HashMap<String, Vec<String>>>,
    ) -> Self
    where
        R: Into<String>,
        Q: Into<String>,
    {
        GraphObject {
            name: name.into(),
            uri: uri.into(),
            query_params: query_params.into(),
            inline_expand,
            per_object_expand: per_object_expand.unwrap_or_default(),
        }
    }
}

/// Enumerates Microsoft Graph data
/// This function retrieves various objects from the Microsoft Graph API
pub async fn enumerate_graph(
    collector: Arc<Collector>,
    resume_state: HashMap<String, (String, usize, f64)>,
) -> Result<(), CirroError> {
    info!("Starting Graph enumeration");

    // Initialize our token cache with a valid token
    {
        let mut cache = TOKEN_CACHE.lock().await;
        if cache.is_none() || cache.as_ref().unwrap().is_expired_or_not_set() {
            debug!("Pre-fetching initial Graph API token");
            match collector.msgraph_credential.get_token().await {
                Ok(token) => {
                    *cache = Some(token);
                    info!("MS Graph token retrieved successfully");
                }
                Err(e) => {
                    warn!(
                        "Failed to acquire MS Graph token - ending Graph enumeration: {}",
                        e
                    );
                    return Ok(());
                }
            }
        }
    }

    // Create all enumerators with their configurations.
    // `inline_expand` identifies the primary navigation property to fetch for
    // every object, and any additional expand properties go into
    // `per_object_expand`. All expand calls are executed per object and batched
    // via `$batch` when possible.
    let mut enumerators = vec![
        GraphObject::new("organization", "organization", "", None, None),
        GraphObject::new(
            "authorizationPolicy",
            "policies/authorizationPolicy",
            "",
            None,
            None,
        ),
        // users: all expands via $batch (no inline $expand — avoids $top=100 cap)
        GraphObject::new(
            "users",
            "users",
            "$top=999",
            None,
            Some(HashMap::from([("memberOf".into(), vec!["/id".into()])])),
        ),
        // groups: all expands via $batch
        GraphObject::new(
            "groups",
            "groups",
            "$top=999",
            None,
            Some(HashMap::from([
                ("members".into(), vec!["/id".into()]),
                ("owners".into(), vec!["/id".into()]),
                ("memberOf".into(), vec!["/id".into()]),
            ])),
        ),
        // applications: all expands via $batch
        GraphObject::new(
            "applications",
            "applications",
            "$top=999",
            None,
            Some(HashMap::from([
                ("owners".into(), vec!["/id".into()]),
                (
                    "federatedIdentityCredentials".into(),
                    vec![
                        "/id".into(),
                        "/name".into(),
                        "/description".into(),
                        "/issuer".into(),
                        "/subject".into(),
                        "/audiences".into(),
                    ],
                ),
            ])),
        ),
        // servicePrincipals: all expands via $batch
        GraphObject::new(
            "servicePrincipals",
            "servicePrincipals",
            "$top=999",
            None,
            Some(HashMap::from([
                ("memberOf".into(), vec!["/id".into()]),
                ("owners".into(), vec!["/id".into()]),
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
        // devices: all expands via $batch
        GraphObject::new(
            "devices",
            "devices",
            "$top=999",
            None,
            Some(HashMap::from([
                ("memberOf".into(), vec!["/id".into()]),
                ("registeredOwners".into(), vec!["/id".into()]),
                ("registeredUsers".into(), vec!["/id".into()]),
            ])),
        ),
        // directoryRoles: expand members
        GraphObject::new(
            "directoryRoles",
            "directoryRoles",
            "",
            Some(("members".into(), vec!["/id".into()])),
            None,
        ),
        // administrativeUnits: expand members and scopedRoleMembers
        GraphObject::new(
            "administrativeUnits",
            "administrativeUnits",
            "",
            Some(("members".into(), vec!["/id".into()])),
            Some(HashMap::from([(
                "scopedRoleMembers".into(),
                vec!["/roleId".into(), "/roleMemberInfo/id".into()],
            )])),
        ),
        // oauth2PermissionGrants
        GraphObject::new(
            "oauth2PermissionGrants",
            "oauth2PermissionGrants",
            "$top=999",
            None,
            None,
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
            None,
        ));
    }

    if collector.option_enum_flags.caps {
        enumerators.push(GraphObject::new(
            "conditionalAccessPolicies",
            "/identity/conditionalAccess/policies",
            "",
            None,
            None,
        ));

        enumerators.push(GraphObject::new(
            "namedLocations",
            "/identity/conditionalAccess/namedLocations",
            "",
            None,
            None,
        ));
    }

    // Cap at 4 for balance of parallelism vs throttle pressure.
    let concurrency = cmp::min(4, cmp::max(1, enumerators.len()));

    // Start a timer
    let start_time = Instant::now();

    // Use a concurrent stream with controlled concurrency for better resource usage
    let results = stream::iter(enumerators)
        .map(|enumerator| {
            let collector_clone = Arc::clone(&collector);
            let resume = resume_state.get(&enumerator.name).cloned();
            async move {
                match query_objects(enumerator, collector_clone, resume).await {
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

/// Queries objects from the Microsoft Graph API.
/// Processes each page of results immediately and writes to the database incrementally.
/// Uses inline `$expand` for one property per resource type (Option A), and batches
/// the remaining per-object expand calls via `$batch` (Option B).
async fn query_objects(
    graph_object: GraphObject,
    collector: Arc<Collector>,
    resume: Option<(String, usize, f64)>,
) -> Result<(), CirroError> {
    let start_time = std::time::Instant::now();
    let uri_path = &graph_object.uri;
    let graph_object_name = &graph_object.name;

    info!(
        "Querying Graph API for resource type: {}",
        graph_object_name
    );

    // Resume from saved state if available (uri, previously collected count, prior elapsed time)
    let (mut next_uri, mut total_fetched, prior_elapsed) =
        if let Some((uri, count, elapsed)) = resume {
            info!(
                "Resuming {} from saved state ({} objects, {:.1}s prior)",
                graph_object_name, count, elapsed
            );
            (uri, count, elapsed)
        } else {
            (
                format!("{}?{}", uri_path, graph_object.query_params),
                0,
                0.0,
            )
        };

    let inline_expand = &graph_object.inline_expand;
    let per_object_expand = &graph_object.per_object_expand;

    let table = if uri_path.starts_with("policies/") {
        "policies".to_string()
    } else {
        graph_object_name.to_string()
    };

    // Pre-resolve tenant ID once for policy objects instead of per-object inside the stream
    let tenant_id: Option<Arc<str>> = if uri_path.starts_with("policies/") {
        Some(
            collector
                .msgraph_credential
                .as_ref()
                .get_token()
                .await?
                .get_claims()?
                .tid
                .unwrap_or_else(|| "unknown_tenant".to_string())
                .into(),
        )
    } else {
        None
    };

    // Process each page as it arrives instead of accumulating all objects
    loop {
        debug!("Fetching data from: {}", next_uri);

        let response_data = paged_graph_request(&collector, &next_uri)
            .await
            .map_err(|e| {
                error!("Failed to fetch data from Graph API: {} - {}", &next_uri, e);
                e
            })?;

        if let Some(error) = response_data.get("@odata.error") {
            let error_message = error
                .get("message")
                .and_then(|m| m.get("value"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            error!("Graph API error: {}", error_message);
            return Err(CirroError::ODataError(error_message.to_string()));
        }

        // Process this page immediately, then discard — memory stays bounded to page size
        if let Some(values) = response_data.get("value").and_then(|v| v.as_array()) {
            let page_count = values.len();
            total_fetched += page_count;
            debug!(
                "Fetched {} objects for {}, total: {}",
                page_count, graph_object_name, total_fetched
            );

            let mut page_values: Vec<serde_json::Value> = values.to_vec();

            // Step 1: Remove any inline-expanded property data so the explicit
            // per-object expansion results become the single source of truth.
            if let Some((property, _)) = inline_expand {
                for value in page_values.iter_mut() {
                    if let Some(obj) = value.as_object_mut() {
                        obj.remove(property.as_str());
                    }
                }
            }

            // Step 2: Collect all per-object expand requests and batch them.
            // This includes the inline_expand property and all per_object_expand
            // properties for every object.
            let mut batch_requests: Vec<(usize, String, String, Vec<String>)> = Vec::new();

            for (idx, value) in page_values.iter().enumerate() {
                let object_id = value
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown_id");

                // Always expand the inline property explicitly so we are not
                // relying on the partial inline response shape.
                if let Some((property, json_pointers)) = inline_expand {
                    batch_requests.push((
                        idx,
                        property.clone(),
                        format!("/{}/{}/{}", uri_path, object_id, property),
                        json_pointers.clone(),
                    ));
                }

                // Add all per-object expand requests
                for (property, json_pointers) in per_object_expand {
                    batch_requests.push((
                        idx,
                        property.clone(),
                        format!("/{}/{}/{}", uri_path, object_id, property),
                        json_pointers.clone(),
                    ));
                }
            }

            // Step 3: Execute batch requests concurrently in chunks of BATCH_MAX_REQUESTS
            if !batch_requests.is_empty() {
                let num_chunks =
                    (batch_requests.len() + BATCH_MAX_REQUESTS - 1) / BATCH_MAX_REQUESTS;
                debug!(
                    "Expanding {} properties across {} objects for {} ({} $batch calls of up to {})",
                    per_object_expand.len() + usize::from(inline_expand.is_some()),
                    page_count,
                    graph_object_name,
                    num_chunks,
                    BATCH_MAX_REQUESTS,
                );

                // Own each chunk so it can be moved into concurrent async tasks
                let owned_chunks: Vec<(usize, Vec<(usize, String, String, Vec<String>)>)> =
                    batch_requests
                        .chunks(BATCH_MAX_REQUESTS)
                        .enumerate()
                        .map(|(i, c)| (i, c.to_vec()))
                        .collect();

                // Fire up to 10 $batch calls concurrently, then apply results
                let mut batch_results: Vec<_> = stream::iter(owned_chunks)
                    .map(|(chunk_idx, chunk)| {
                        let collector = Arc::clone(&collector);
                        async move {
                            let result = batch_graph_request(&collector, &chunk).await;
                            (chunk_idx, chunk, result)
                        }
                    })
                    .buffer_unordered(10)
                    .collect::<Vec<_>>()
                    .await;

                // Apply results sequentially (needs mutable page_values)
                batch_results.sort_by_key(|(idx, _, _)| *idx);

                for (_chunk_idx, chunk, result) in batch_results {
                    match result {
                        Ok(responses) => {
                            for (idx, property, json_pointers, response_value) in responses {
                                if let Some(value) = page_values.get_mut(idx) {
                                    apply_expand_response(
                                        value,
                                        &property,
                                        &json_pointers,
                                        &response_value,
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "Batch expand failed for {}: {}. Retrying as single-request batches.",
                                graph_object_name, e
                            );
                            // Retry each request as its own $batch call so we stay on the
                            // batch code path while isolating failures.
                            for (idx, property, url, json_pointers) in &chunk {
                                let single_request = vec![(
                                    *idx,
                                    property.clone(),
                                    url.clone(),
                                    json_pointers.clone(),
                                )];
                                match batch_graph_request(&collector, &single_request).await {
                                    Ok(responses) => {
                                        for (
                                            response_idx,
                                            response_property,
                                            response_json_pointers,
                                            response_value,
                                        ) in responses
                                        {
                                            if let Some(value) = page_values.get_mut(response_idx) {
                                                apply_expand_response(
                                                    value,
                                                    &response_property,
                                                    &response_json_pointers,
                                                    &response_value,
                                                );
                                            }
                                        }
                                    }
                                    Err(single_err) => {
                                        warn!(
                                            "Single-request batch expand failed for {}/{} (object index {}): {}",
                                            graph_object_name, property, idx, single_err
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Step 4: Build rows with IDs and write to DB
            let mut rows = Vec::with_capacity(page_count);
            for value in page_values {
                let id = if let Some(tid) = &tenant_id {
                    let policy_type = value
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown_id");
                    format!("{}_{}", tid, policy_type)
                } else {
                    value
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown_id")
                        .to_string()
                };
                rows.push((id, value));
            }

            // Write per-page — the DB writer channel can process and flush incrementally
            if !rows.is_empty() {
                collector
                    .write_values_batch_to_db(table.clone(), rows)
                    .await
                    .map_err(|e| {
                        error!(
                            "Failed to write batch for resource type {}: {}",
                            graph_object_name, e
                        );
                        e
                    })?;
            }
        }
        // response_data and page values are dropped here — memory freed each iteration

        // Check for next link
        if let Some(next_link) = response_data
            .get("@odata.nextLink")
            .and_then(|v| v.as_str())
        {
            next_uri = next_link.to_string();

            // Save pagination state so we can resume if interrupted
            if let Some(sender) = &collector.db_writer {
                let _ = sender
                    .send(DBWriteMessage::SaveState {
                        resource_type: graph_object_name.to_string(),
                        next_uri: next_uri.clone(),
                        total_collected: total_fetched,
                        elapsed_secs: prior_elapsed + start_time.elapsed().as_secs_f64(),
                    })
                    .await;
            }
        } else {
            // Enumeration complete — clear saved state for this resource type
            if let Some(sender) = &collector.db_writer {
                let _ = sender
                    .send(DBWriteMessage::ClearState {
                        resource_type: graph_object_name.to_string(),
                    })
                    .await;
            }
            break;
        }
    }

    let total_elapsed = prior_elapsed + start_time.elapsed().as_secs_f64();
    info!(
        "Completed Graph API query for resource type: {} ({:.1}s total) - {} objects processed",
        graph_object_name, total_elapsed, total_fetched
    );

    Ok(())
}

/// Applies the response data from a single expand request to the object,
/// extracting the specified fields via JSON pointers.
fn apply_expand_response(
    object: &mut serde_json::Value,
    property: &str,
    json_pointers: &[String],
    response_data: &serde_json::Value,
) {
    let values = match response_data.get("value").and_then(|v| v.as_array()) {
        Some(v) => v,
        None => return,
    };

    let mut extracted = Vec::new();
    for value in values {
        if json_pointers.len() == 1 {
            if let Some(expanded_value) = value.pointer(&json_pointers[0]) {
                extracted.push(expanded_value.clone());
            }
        } else {
            let mut map = serde_json::Map::new();
            for pointer in json_pointers {
                if let Some(expanded_value) = value.pointer(pointer) {
                    map.insert(
                        pointer.trim_start_matches('/').to_string(),
                        expanded_value.clone(),
                    );
                }
            }
            extracted.push(serde_json::Value::Object(map));
        }
    }

    if !extracted.is_empty() {
        if let Some(obj) = object.as_object_mut() {
            let entry = obj
                .entry(property)
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));
            if let Some(array) = entry.as_array_mut() {
                array.extend(extracted);
            }
        }
    }
}

/// Executes a batch of expand requests using the Graph `$batch` API endpoint.
async fn batch_graph_request(
    collector: &Collector,
    requests: &[(usize, String, String, Vec<String>)],
) -> Result<Vec<(usize, String, Vec<String>, serde_json::Value)>, CirroError> {
    // Build the batch request body
    let batch_requests: Vec<serde_json::Value> = requests
        .iter()
        .enumerate()
        .map(|(i, (_idx, _property, url, _pointers))| {
            serde_json::json!({
                "id": i.to_string(),
                "method": "GET",
                "url": url,
            })
        })
        .collect();

    let batch_body = serde_json::json!({ "requests": batch_requests });

    let batch_url = format!("{}/beta/$batch", collector.cloud_endpoints.msgraph_url);

    // Get the access token
    let token_str = {
        let mut cache = TOKEN_CACHE.lock().await;
        let need_new_token = match &*cache {
            Some(cached_token) => cached_token.is_expired_or_not_set(),
            None => true,
        };
        if need_new_token {
            let token = collector.msgraph_credential.get_token().await?;
            *cache = Some(token.clone());
            Arc::clone(&token.access_token)
        } else {
            Arc::clone(&cache.as_ref().unwrap().access_token)
        }
    };

    let response = HTTP_CLIENT
        .post(&batch_url)
        .bearer_auth(&*token_str)
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(60))
        .json(&batch_body)
        .send()
        .await
        .map_err(|e| CirroError::HttpError(format!("Batch request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(CirroError::HttpError(format!(
            "Batch HTTP {} - {}",
            status.as_u16(),
            error_text
        )));
    }

    let batch_response: serde_json::Value = response.json().await?;

    let mut results = Vec::new();
    if let Some(responses) = batch_response.get("responses").and_then(|v| v.as_array()) {
        for resp in responses {
            let id_str = resp.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let status = resp.get("status").and_then(|v| v.as_u64()).unwrap_or(500);

            if let Ok(batch_idx) = id_str.parse::<usize>() {
                if status >= 200 && status < 300 {
                    if let Some((obj_idx, property, _url, pointers)) = requests.get(batch_idx) {
                        // Follow pagination for each successful sub-response so callers
                        // receive one complete value set per requested expansion.
                        let mut body = resp.get("body").cloned().unwrap_or(serde_json::Value::Null);
                        let mut combined_values = body
                            .get("value")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();

                        let mut next_link = body
                            .get("@odata.nextLink")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        while let Some(url) = next_link {
                            let next_page = paged_graph_request(collector, &url).await?;
                            if let Some(values) = next_page.get("value").and_then(|v| v.as_array())
                            {
                                combined_values.extend(values.iter().cloned());
                            }
                            next_link = next_page
                                .get("@odata.nextLink")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }

                        if let Some(obj) = body.as_object_mut() {
                            obj.insert(
                                "value".to_string(),
                                serde_json::Value::Array(combined_values),
                            );
                            obj.remove("@odata.nextLink");
                        }

                        results.push((*obj_idx, property.clone(), pointers.clone(), body));
                    }
                } else {
                    debug!("Batch sub-request {} returned status {}", batch_idx, status);
                }
            }
        }
    }

    Ok(results)
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

            // Respect the Retry-After header from the Graph API response
            // https://learn.microsoft.com/en-us/graph/throttling#best-practices-to-handle-throttling
            let retry_after_secs = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(25);

            // Only log the first time any task hits the rate limit
            if !RATE_LIMIT_LOGGED.swap(true, Ordering::Relaxed) {
                info!(
                    "Graph API rate limit hit, backing off for {} seconds",
                    retry_after_secs
                );

                // The task that logs the message also starts a timer to reset the flag
                // Should be slightly longer than the backoff and other tasks will not log
                let reset_delay = retry_after_secs + 5;
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(reset_delay)).await;
                    RATE_LIMIT_LOGGED.store(false, Ordering::Relaxed);
                });
            }

            tokio::time::sleep(std::time::Duration::from_secs(retry_after_secs)).await;

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
