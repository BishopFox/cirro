use crate::graph::specs::SpecTrait;
use serde::Deserialize;

/// Specs for Cirro Azure resource ingestion
#[derive(Debug, Deserialize, Clone)]
pub struct CirroAzureIngestSpec {
    /// Human-friendly name
    pub name: String,

    /// Label for the resource type
    pub label: String,

    /// Table name (e.g. "resources", "subscriptions")
    pub table_name: String,

    /// Resource type identifier (e.g. "Microsoft.Compute/virtualMachines")
    pub resource_type: Option<String>,

    /// Properties to extract from the resource JSON
    pub properties: Vec<String>,

    /// Optional list of Label:property or Label:prop1+prop2 pairs to create uniqueness constraints on
    pub constraint_properties: Option<Vec<String>>,

    /// Optional list of Label:property or Label:prop1+prop2 pairs to create additional indexed properties
    pub index_properties: Option<Vec<String>>,

    /// Map of column values from SQL column values to Cypher parameters
    /// This is used to inject additional values into the cypher query
    pub column_mappings: Option<std::collections::HashMap<String, String>>,

    /// Cypher query to insert the resource nodes
    pub cypher: String,

    /// Optional: Priority of this resource when ingesting (lower numbers = higher priority)
    pub priority: Option<u32>,
}

impl SpecTrait for CirroAzureIngestSpec {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_label(&self) -> &str {
        &self.label
    }
}
