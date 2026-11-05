use super::graph::{GraphPager, graph_get_response};
use super::restricted_guest_apps::{application_name, microsoft_apps, well_known_apps};
use crate::collect::azure::collect::Collector;
use crate::errors::CirroError;

use log::{debug, info, warn};
use reqwest::StatusCode;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

#[derive(Default)]
struct CrawlStats {
    seeds_found: usize,
    seeds_missing: usize,
    seeds_blocked: usize,
    applications: usize,
    oauth_grants: usize,
    assignments: usize,
    relationship_failures: usize,
    unresolved_service_principals: usize,
}

#[derive(Default)]
struct CrawlState {
    service_principals: HashMap<String, Value>,
    app_id_to_sp_id: HashMap<String, String>,
    neighbors: HashMap<String, Vec<String>>,
    unavailable_service_principals: HashSet<String>,
    probed_application_ids: HashSet<String>,
    written_grant_ids: HashSet<String>,
    pending_assignments: HashMap<String, Vec<Value>>,
    stats: CrawlStats,
}

enum OptionalResponse {
    Found(Value),
    Forbidden,
    NotFound,
}

struct CollectionResponse {
    values: Vec<Value>,
    incomplete: bool,
}

/// Enumerates service principals and applications through relationship surfaces
/// that remain readable to a restricted Entra ID guest.
pub async fn enumerate_restricted_guest(collector: Arc<Collector>) -> Result<(), CirroError> {
    let seed_apps = restricted_seed_apps(&collector);
    let mut state = CrawlState::default();
    let mut visited = HashSet::new();

    info!(
        "Starting restricted-guest Graph enumeration with {} application ID seed(s)",
        seed_apps.len()
    );

    for (seed_app_id, display_name) in seed_apps {
        info!("Enumerating {}: {}", seed_app_id, display_name);

        let Some(root_id) =
            resolve_seed_service_principal(&collector, &seed_app_id, &mut state).await?
        else {
            continue;
        };

        let mut queue = VecDeque::from([root_id]);

        while let Some(service_principal_id) = queue.pop_front() {
            if !visited.insert(service_principal_id.clone()) {
                continue;
            }

            let neighbors =
                process_service_principal(&collector, &service_principal_id, &mut state).await?;

            for neighbor_id in neighbors {
                if !visited.contains(&neighbor_id) {
                    queue.push_back(neighbor_id);
                }
            }
        }
    }

    info!(
        "Restricted-guest Graph enumeration completed: {} seed(s) found, {} missing, {} blocked; {} service principal(s), {} application(s), {} OAuth grant(s), {} app-role assignment(s), {} incomplete relationship(s), {} unresolved service principal(s)",
        state.stats.seeds_found,
        state.stats.seeds_missing,
        state.stats.seeds_blocked,
        state.service_principals.len(),
        state.stats.applications,
        state.stats.oauth_grants,
        state.stats.assignments,
        state.stats.relationship_failures,
        state.stats.unresolved_service_principals,
    );

    Ok(())
}

pub(crate) fn microsoft_application_count() -> usize {
    microsoft_apps().len()
}

fn restricted_seed_apps(collector: &Collector) -> Vec<(String, String)> {
    let configured = &collector.option_enum_flags.restricted_appids;
    if !configured.is_empty() {
        let mut seen = HashSet::new();
        return configured
            .iter()
            .map(|app_id| app_id.to_ascii_lowercase())
            .filter(|app_id| seen.insert(app_id.clone()))
            .map(|app_id| {
                let display_name = application_name(&app_id)
                    .unwrap_or("Unknown application")
                    .to_string();
                (app_id, display_name)
            })
            .collect();
    }

    if collector.option_enum_flags.restricted_guest_msapps {
        return microsoft_apps()
            .iter()
            .map(|(app_id, catalog_name)| {
                let display_name = application_name(app_id).unwrap_or(catalog_name).to_string();
                (app_id.clone(), display_name)
            })
            .collect();
    }

    well_known_apps()
        .iter()
        .map(|(app_id, display_name)| ((*app_id).to_string(), (*display_name).to_string()))
        .collect()
}

async fn resolve_seed_service_principal(
    collector: &Collector,
    app_id: &str,
    state: &mut CrawlState,
) -> Result<Option<String>, CirroError> {
    if let Some(service_principal_id) = state.app_id_to_sp_id.get(app_id) {
        state.stats.seeds_found += 1;
        return Ok(Some(service_principal_id.clone()));
    }

    let uri = format!("servicePrincipals(appId='{}')", app_id);
    match get_optional_object(collector, &uri).await? {
        OptionalResponse::Found(service_principal) => {
            let service_principal_id = register_service_principal(state, service_principal)?;
            state.stats.seeds_found += 1;
            debug!(
                "Resolved restricted-guest seed app ID {} to service principal {}",
                app_id, service_principal_id
            );
            Ok(Some(service_principal_id))
        }
        OptionalResponse::Forbidden => {
            state.stats.seeds_blocked += 1;
            warn!(
                "Authorization prevented alternate-key lookup for application ID {}; presence is indeterminate",
                app_id
            );
            Ok(None)
        }
        OptionalResponse::NotFound => {
            state.stats.seeds_missing += 1;
            debug!(
                "No service principal was found for application ID {}",
                app_id
            );
            Ok(None)
        }
    }
}

async fn process_service_principal(
    collector: &Collector,
    service_principal_id: &str,
    state: &mut CrawlState,
) -> Result<Vec<String>, CirroError> {
    if let Some(neighbors) = state.neighbors.get(service_principal_id) {
        return Ok(neighbors.clone());
    }
    if state
        .unavailable_service_principals
        .contains(service_principal_id)
    {
        return Ok(Vec::new());
    }

    if !state.service_principals.contains_key(service_principal_id) {
        let uri = format!("servicePrincipals/{}", service_principal_id);
        match get_optional_object(collector, &uri).await? {
            OptionalResponse::Found(service_principal) => {
                register_service_principal(state, service_principal)?;
            }
            OptionalResponse::Forbidden | OptionalResponse::NotFound => {
                state
                    .unavailable_service_principals
                    .insert(service_principal_id.to_string());
                state.stats.unresolved_service_principals += 1;
                warn!(
                    "Could not resolve discovered service principal {}; skipping its relationships",
                    service_principal_id
                );
                return Ok(Vec::new());
            }
        }
    }

    let app_id = state
        .service_principals
        .get(service_principal_id)
        .and_then(|value| value.get("appId"))
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase);

    if let Some(app_id) = app_id {
        probe_application(collector, &app_id, state).await?;
    }

    let inbound =
        fetch_sp_relationship(collector, service_principal_id, "appRoleAssignedTo").await?;
    let outbound =
        fetch_sp_relationship(collector, service_principal_id, "appRoleAssignments").await?;
    let oauth_grants =
        fetch_sp_relationship(collector, service_principal_id, "oauth2PermissionGrants").await?;
    let owners = fetch_sp_relationship(collector, service_principal_id, "owners").await?;

    state.stats.relationship_failures += usize::from(inbound.incomplete)
        + usize::from(outbound.incomplete)
        + usize::from(oauth_grants.incomplete)
        + usize::from(owners.incomplete);

    let mut neighbors = Vec::new();
    let mut seen_neighbors = HashSet::new();
    let mut inbound_assignments = Vec::new();

    for assignment in &inbound.values {
        if let Some(normalized) = normalize_assignment(assignment, None) {
            push_unique_assignment(&mut inbound_assignments, normalized);
            state.stats.assignments += 1;
        }

        let is_service_principal = assignment
            .get("principalType")
            .and_then(Value::as_str)
            .is_some_and(|value| value.eq_ignore_ascii_case("ServicePrincipal"));
        if is_service_principal
            && let Some(principal_id) = assignment
                .get("principalId")
                .and_then(Value::as_str)
                .map(str::to_ascii_lowercase)
        {
            push_unique_neighbor(&mut neighbors, &mut seen_neighbors, principal_id);
        }
    }

    let mut cached_targets_to_rewrite = HashSet::new();
    for assignment in &outbound.values {
        let Some(resource_id) = assignment
            .get("resourceId")
            .and_then(Value::as_str)
            .map(str::to_ascii_lowercase)
        else {
            continue;
        };

        push_unique_neighbor(&mut neighbors, &mut seen_neighbors, resource_id.clone());

        if let Some(normalized) = normalize_assignment(assignment, Some(service_principal_id)) {
            let pending = state
                .pending_assignments
                .entry(resource_id.clone())
                .or_default();
            push_unique_assignment(pending, normalized);
            state.stats.assignments += 1;
            if state.service_principals.contains_key(&resource_id) {
                cached_targets_to_rewrite.insert(resource_id);
            }
        }
    }

    let mut grant_rows = Vec::new();
    for grant in oauth_grants.values {
        if let Some(resource_id) = grant
            .get("resourceId")
            .and_then(Value::as_str)
            .map(str::to_ascii_lowercase)
        {
            push_unique_neighbor(&mut neighbors, &mut seen_neighbors, resource_id);
        }

        let Some(grant_id) = grant.get("id").and_then(Value::as_str) else {
            warn!(
                "Skipping OAuth2 permission grant without an ID on service principal {}",
                service_principal_id
            );
            continue;
        };
        let grant_id = grant_id.to_ascii_lowercase();
        if state.written_grant_ids.insert(grant_id.clone()) {
            grant_rows.push((grant_id, grant));
        }
    }
    if !grant_rows.is_empty() {
        state.stats.oauth_grants += grant_rows.len();
        collector
            .write_values_batch_to_db("oauth2PermissionGrants".to_string(), grant_rows)
            .await?;
    }

    let mut owner_ids = Vec::new();
    let mut seen_owner_ids = HashSet::new();
    for owner in owners.values {
        let Some(owner_id) = owner
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_ascii_lowercase)
        else {
            continue;
        };

        if seen_owner_ids.insert(owner_id.clone()) {
            owner_ids.push(owner_id.clone());
        }

        let is_service_principal = owner
            .get("@odata.type")
            .and_then(Value::as_str)
            .is_some_and(|value| value == "#microsoft.graph.servicePrincipal");
        if is_service_principal {
            push_unique_neighbor(&mut neighbors, &mut seen_neighbors, owner_id);
        }
    }

    let pending_for_current = state
        .pending_assignments
        .get(service_principal_id)
        .cloned()
        .unwrap_or_default();
    let current_snapshot = {
        let service_principal = state
            .service_principals
            .get_mut(service_principal_id)
            .ok_or_else(|| {
                CirroError::InvalidData(format!(
                    "Missing cached service principal {}",
                    service_principal_id
                ))
            })?;
        let service_principal_object = service_principal.as_object_mut().ok_or_else(|| {
            CirroError::InvalidData("Cached Graph object was not a JSON object".to_string())
        })?;
        service_principal_object.insert(
            "owners".to_string(),
            Value::Array(owner_ids.iter().cloned().map(Value::String).collect()),
        );
        service_principal_object.insert("appRoleAssignedTo".to_string(), Value::Array(Vec::new()));
        merge_assignment_property(service_principal, &inbound_assignments)?;
        merge_assignment_property(service_principal, &pending_for_current)?;
        service_principal.clone()
    };
    collector
        .write_value_to_db(
            "servicePrincipals".to_string(),
            service_principal_id.to_string(),
            current_snapshot,
        )
        .await?;

    for target_id in cached_targets_to_rewrite {
        let pending = state
            .pending_assignments
            .get(&target_id)
            .cloned()
            .unwrap_or_default();
        let Some(target) = state.service_principals.get_mut(&target_id) else {
            continue;
        };
        merge_assignment_property(target, &pending)?;
        let target_snapshot = target.clone();
        collector
            .write_value_to_db("servicePrincipals".to_string(), target_id, target_snapshot)
            .await?;
    }

    state
        .neighbors
        .insert(service_principal_id.to_string(), neighbors.clone());

    Ok(neighbors)
}

async fn probe_application(
    collector: &Collector,
    app_id: &str,
    state: &mut CrawlState,
) -> Result<(), CirroError> {
    if !state.probed_application_ids.insert(app_id.to_string()) {
        return Ok(());
    }

    let uri = format!("applications(appId='{}')", app_id);
    match get_optional_object(collector, &uri).await? {
        OptionalResponse::Found(application) => {
            let application_id = required_id(&application, "application")?;
            collector
                .write_value_to_db("applications".to_string(), application_id, application)
                .await?;
            state.stats.applications += 1;
        }
        OptionalResponse::Forbidden => {
            debug!(
                "Backing application lookup for app ID {} was forbidden",
                app_id
            );
        }
        OptionalResponse::NotFound => {
            debug!("No local application exists for app ID {}", app_id);
        }
    }

    Ok(())
}

async fn get_optional_object(
    collector: &Collector,
    uri: &str,
) -> Result<OptionalResponse, CirroError> {
    let response = graph_get_response(collector, uri).await?;
    if response.status == StatusCode::FORBIDDEN {
        return Ok(OptionalResponse::Forbidden);
    }
    if response.status == StatusCode::NOT_FOUND {
        return Ok(OptionalResponse::NotFound);
    }

    let body = response.into_success()?;
    if !body.is_object() {
        return Err(CirroError::InvalidData(format!(
            "Graph response for {} was not an object",
            uri
        )));
    }

    Ok(OptionalResponse::Found(body))
}

async fn fetch_sp_relationship(
    collector: &Collector,
    service_principal_id: &str,
    relationship: &str,
) -> Result<CollectionResponse, CirroError> {
    fetch_collection(
        collector,
        &format!(
            "servicePrincipals/{}/{}?$top=999",
            service_principal_id, relationship
        ),
        service_principal_id,
        relationship,
    )
    .await
}

async fn fetch_collection(
    collector: &Collector,
    initial_uri: &str,
    service_principal_id: &str,
    relationship: &str,
) -> Result<CollectionResponse, CirroError> {
    let mut values = Vec::new();
    let mut incomplete = false;
    let mut pager = GraphPager::new(initial_uri);

    while let Some(response) = pager.next_page(collector).await? {
        let status = response.status;
        if status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND {
            debug!(
                "Relationship request for {}/{} returned HTTP {}; keeping {} collected item(s)",
                service_principal_id,
                relationship,
                status.as_u16(),
                values.len()
            );
            incomplete = true;
            break;
        }

        let body = response.into_success()?;
        let page_values = body.get("value").and_then(Value::as_array).ok_or_else(|| {
            CirroError::InvalidData(format!(
                "Graph collection response for {}/{} did not contain a value array",
                service_principal_id, relationship
            ))
        })?;
        values.extend(page_values.iter().cloned());
    }

    Ok(CollectionResponse { values, incomplete })
}

fn register_service_principal(
    state: &mut CrawlState,
    service_principal: Value,
) -> Result<String, CirroError> {
    let service_principal_id = required_id(&service_principal, "service principal")?;
    if let Some(app_id) = service_principal
        .get("appId")
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase)
    {
        state
            .app_id_to_sp_id
            .insert(app_id, service_principal_id.clone());
    }
    state
        .service_principals
        .entry(service_principal_id.clone())
        .or_insert(service_principal);
    Ok(service_principal_id)
}

fn required_id(value: &Value, object_type: &str) -> Result<String, CirroError> {
    value
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| {
            CirroError::InvalidData(format!(
                "Graph {} response did not contain an ID",
                object_type
            ))
        })
}

fn normalize_assignment(assignment: &Value, fallback_principal_id: Option<&str>) -> Option<Value> {
    let principal_id = assignment
        .get("principalId")
        .and_then(Value::as_str)
        .or(fallback_principal_id)?
        .to_ascii_lowercase();
    let app_role_id = assignment
        .get("appRoleId")
        .and_then(Value::as_str)?
        .to_ascii_lowercase();

    Some(json!({
        "principalId": principal_id,
        "appRoleId": app_role_id,
    }))
}

fn merge_assignment_property(object: &mut Value, additions: &[Value]) -> Result<(), CirroError> {
    if additions.is_empty() {
        return Ok(());
    }
    let object = object.as_object_mut().ok_or_else(|| {
        CirroError::InvalidData("Cached Graph object was not a JSON object".to_string())
    })?;
    let assignments = object
        .entry("appRoleAssignedTo")
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| {
            CirroError::InvalidData(
                "Cached Graph object appRoleAssignedTo property was not an array".to_string(),
            )
        })?;
    for addition in additions {
        push_unique_assignment(assignments, addition.clone());
    }
    Ok(())
}

fn push_unique_assignment(assignments: &mut Vec<Value>, assignment: Value) {
    let key = assignment_key(&assignment);
    if assignments
        .iter()
        .all(|existing| assignment_key(existing) != key)
    {
        assignments.push(assignment);
    }
}

fn assignment_key(assignment: &Value) -> String {
    let principal_id = assignment
        .get("principalId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let app_role_id = assignment
        .get("appRoleId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if principal_id.is_empty() && app_role_id.is_empty() {
        assignment.to_string()
    } else {
        format!("{}:{}", principal_id, app_role_id)
    }
}

fn push_unique_neighbor(
    neighbors: &mut Vec<String>,
    seen: &mut HashSet<String>,
    neighbor_id: String,
) {
    if seen.insert(neighbor_id.clone()) {
        neighbors.push(neighbor_id);
    }
}
