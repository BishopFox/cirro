use crate::errors::CirroGraphError;
use crate::specs::{SpecLoader, SpecRegistry};
use serde::de::DeserializeOwned;

/// Generic trait for loading specs of any type
pub trait GenericSpecSource<T: DeserializeOwned> {
    fn load_specs(&self, path_filter: &str) -> Result<Vec<T>, CirroGraphError>;
}

/// Configuration for a specific spec type and its path
#[derive(Debug, Clone)]
pub struct SpecConfig {
    pub name: &'static str,
    pub disk_path: &'static str,
    pub embedded_prefix: &'static str,
}

// Spec configurations for different types
pub const CIRRO_AZURE_SPEC_CONFIG: SpecConfig = SpecConfig {
    name: "Cirro Azure",
    disk_path: concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/config/azure/**/*.tera.yaml"
    ),
    embedded_prefix: "azure/",
};

pub const CIRRO_TAILSCALE_STATUS_SPEC_CONFIG: SpecConfig = SpecConfig {
    name: "Cirro Tailscale Status",
    disk_path: concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/config/tailscale/status/*.tera.yaml"
    ),
    embedded_prefix: "tailscale/status/",
};

pub const CIRRO_POST_PROCESSING_SPEC_CONFIG: SpecConfig = SpecConfig {
    name: "Cirro Post Processing",
    disk_path: concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/config/post_processing/*.tera.yaml"
    ),
    embedded_prefix: "post_processing/",
};

/// Registry of all known spec configurations
/// The order here defines the ingestion order
pub const ALL_SPEC_CONFIGS: &[&SpecConfig] = &[
    &CIRRO_AZURE_SPEC_CONFIG,
    &CIRRO_TAILSCALE_STATUS_SPEC_CONFIG,
    &CIRRO_POST_PROCESSING_SPEC_CONFIG,
];

/// Load all spec types automatically
pub fn load_all_specs() -> Result<SpecRegistry, CirroGraphError> {
    SpecLoader::load_all_specs()
}
