use crate::graph::specs::SpecTrait;
use serde::Deserialize;

/// Specs for Cirro Post-Processing ingestion
#[derive(Debug, Deserialize, Clone)]
pub struct CirroPostProcessingSpec {
    /// Human-friendly name
    pub name: String,

    /// Label. This will be empty for post-process specs
    pub label: String,

    /// Priority for execution order
    pub priority: u32,

    /// Cypher query to execute
    pub cypher: String,
}

impl SpecTrait for CirroPostProcessingSpec {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_label(&self) -> &str {
        &self.label
    }
}
