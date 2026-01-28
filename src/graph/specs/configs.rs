use crate::errors::CirroError;
use crate::graph::specs::{SpecLoader, SpecRegistry};

/// Configuration for a specific spec type and its path
#[derive(Debug, Clone)]
pub struct SpecConfig {
    pub name: &'static str,
    pub path_prefix: &'static str,
}

// Spec configurations for different types
pub const CIRRO_AZURE_SPEC_CONFIG: SpecConfig = SpecConfig {
    name: "Cirro Azure",
    path_prefix: "azure/",
};

pub const CIRRO_TAILSCALE_STATUS_SPEC_CONFIG: SpecConfig = SpecConfig {
    name: "Cirro Tailscale Status",
    path_prefix: "tailscale/status/",
};

pub const CIRRO_POST_PROCESSING_SPEC_CONFIG: SpecConfig = SpecConfig {
    name: "Cirro Post Processing",
    path_prefix: "post_processing/",
};

/// Registry of all known spec configurations
/// The order here defines the ingestion order
pub const ALL_SPEC_CONFIGS: &[&SpecConfig] = &[
    &CIRRO_AZURE_SPEC_CONFIG,
    &CIRRO_TAILSCALE_STATUS_SPEC_CONFIG,
    &CIRRO_POST_PROCESSING_SPEC_CONFIG,
];

/// Load all spec types automatically
pub fn load_all_specs() -> Result<SpecRegistry, CirroError> {
    SpecLoader::load_all_specs()
}
