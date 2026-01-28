pub mod configs;
pub mod factory;
pub mod sources;

pub mod azure;
pub mod core;
pub mod tailscale;

pub use factory::{SpecLoader, SpecRegistry};

// Re-export core types and API
pub use azure::types::CirroAzureIngestSpec;
pub use tailscale::types::CirroTailscaleStatusIngestSpec;

/// Trait for specs that can be used to create constraints and indexes
pub trait SpecTrait {
    fn get_name(&self) -> &str;
    fn get_label(&self) -> &str;
}
