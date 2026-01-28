use crate::graph::specs::SpecTrait;
use serde::Deserialize;

/// Specs for Cirro Tailscale status JSON ingestion
#[derive(Debug, Deserialize, Clone)]
pub struct CirroTailscaleStatusIngestSpec {
    /// Human-friendly name
    pub name: String,

    /// Label for the resource type
    pub label: String,

    /// Cypher query to insert the resource nodes
    pub cypher: String,
}

impl SpecTrait for CirroTailscaleStatusIngestSpec {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_label(&self) -> &str {
        &self.label
    }
}
