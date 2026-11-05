use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

const MICROSOFT_GRAPH_APP_COUNT: usize = 1_753;

/*
 * Publicly documented, well-known Microsoft application IDs used as the
 * default starting points for restricted-guest enumeration. Keep the
 * display-name comments beside each ID so this map remains easy to review.
 */
static WELL_KNOWN_APP_IDS: Lazy<BTreeMap<&'static str, &'static str>> = Lazy::new(|| {
    BTreeMap::from([
        // Windows Azure Active Directory
        (
            "00000002-0000-0000-c000-000000000000",
            "Windows Azure Active Directory",
        ),
        // Microsoft Graph
        ("00000003-0000-0000-c000-000000000000", "Microsoft Graph"),
        // Office 365 Exchange Online
        (
            "00000002-0000-0ff1-ce00-000000000000",
            "Office 365 Exchange Online",
        ),
        // Office 365 SharePoint Online
        (
            "00000003-0000-0ff1-ce00-000000000000",
            "Office 365 SharePoint Online",
        ),
        // Skype for Business Online
        (
            "00000004-0000-0ff1-ce00-000000000000",
            "Skype for Business Online",
        ),
        // Dataverse
        ("00000007-0000-0000-c000-000000000000", "Dataverse"),
        // Power BI Service
        ("00000009-0000-0000-c000-000000000000", "Power BI Service"),
        // Microsoft Intune
        ("0000000a-0000-0000-c000-000000000000", "Microsoft Intune"),
        // Microsoft Azure CLI
        (
            "04b07795-8ddb-461a-bbee-02f9e1bf7b46",
            "Microsoft Azure CLI",
        ),
        // Microsoft Azure PowerShell
        (
            "1950a258-227b-4e31-a9cf-717495945fc2",
            "Microsoft Azure PowerShell",
        ),
        // Microsoft Teams
        ("1fec8e78-bce4-4aaf-ab1b-5451cc387264", "Microsoft Teams"),
        // Azure DevOps
        ("499b84ac-1321-427f-aa17-267ca6975798", "Azure DevOps"),
        // Windows Azure Service Management API
        (
            "797f4846-ba00-4fd7-ba43-dac1f8f63013",
            "Windows Azure Service Management API",
        ),
        // Azure Portal
        ("c44b4083-3bb0-49c1-b47d-974e53cbdf3c", "Azure Portal"),
        // Azure Key Vault
        ("cfa8b339-82a2-471a-a3c9-0fc0be7a4093", "Azure Key Vault"),
        // Microsoft Office
        ("d3590ed6-52b3-4102-aeff-aad2292ab01c", "Microsoft Office"),
        // Microsoft Intune Enrollment
        (
            "d4ebce55-015a-49b5-a083-c84d1797ae8c",
            "Microsoft Intune Enrollment",
        ),
        // Azure Storage
        ("e406a681-f3d4-42a8-90b6-c2b029497af1", "Azure Storage"),
    ])
});

/// Microsoft application resource catalog from:
/// https://github.com/f-bader/entrascopes.com/blob/b0ccd972055ee7f78b446a5d75e14e7fe4c58ede/resources.json
///
/// Source SHA-256:
/// a20ea8fadb3668f86c1e352697b6b456c5928684c5e820543b5d5cc59ea82bb7
static MICROSOFT_GRAPH_APP_IDS: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
    let resources: Vec<MicrosoftGraphResource> =
        serde_json::from_str(include_str!("restricted_guest_msapps.json"))
            .expect("embedded Microsoft application catalog must be valid JSON");
    let resource_count = resources.len();
    assert_eq!(
        resource_count, MICROSOFT_GRAPH_APP_COUNT,
        "embedded Microsoft application catalog is incomplete"
    );
    let applications: BTreeMap<_, _> = resources
        .into_iter()
        .map(|resource| {
            let app_id = Uuid::parse_str(&resource.resource_id)
                .expect("embedded Microsoft application ID must be a valid UUID")
                .hyphenated()
                .to_string();
            let display_name = resource.display_name.trim();
            assert!(
                !display_name.is_empty(),
                "embedded Microsoft application name must not be empty"
            );
            (app_id, display_name.to_string())
        })
        .collect();

    assert_eq!(
        applications.len(),
        resource_count,
        "embedded Microsoft application catalog contains duplicate IDs"
    );
    applications
});

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MicrosoftGraphResource {
    resource_id: String,
    display_name: String,
}

pub(super) fn well_known_apps() -> &'static BTreeMap<&'static str, &'static str> {
    &WELL_KNOWN_APP_IDS
}

pub(super) fn microsoft_apps() -> &'static BTreeMap<String, String> {
    &MICROSOFT_GRAPH_APP_IDS
}

pub(super) fn application_name(app_id: &str) -> Option<&'static str> {
    WELL_KNOWN_APP_IDS
        .get(app_id)
        .copied()
        .or_else(|| MICROSOFT_GRAPH_APP_IDS.get(app_id).map(String::as_str))
}
