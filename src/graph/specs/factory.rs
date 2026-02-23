use crate::errors::CirroError;
use crate::graph::specs::azure::types::CirroAzureIngestSpec;
use crate::graph::specs::configs::{
    ALL_SPEC_CONFIGS, CIRRO_AZURE_SPEC_CONFIG, CIRRO_POST_PROCESSING_SPEC_CONFIG,
    CIRRO_TAILSCALE_STATUS_SPEC_CONFIG, SpecConfig,
};
use crate::graph::specs::core::types::CirroPostProcessingSpec;
use crate::graph::specs::sources::CirroSpecSource;
use crate::graph::specs::tailscale::types::CirroTailscaleStatusIngestSpec;

/// Macro to generate SpecRegistry with automatic label filtering
macro_rules! define_spec_registry {
    (
        $(
            $field:ident: $type:ty
        ),* $(,)?
    ) => {
        /// Container for all loaded spec types
        #[derive(Debug)]
        pub struct SpecRegistry {
            $(
                pub $field: Vec<$type>,
            )*
        }

        impl SpecRegistry {
            /// Filter all specs in the registry by the provided labels
            pub fn filter_by_labels(mut self, labels: &[String]) -> Self {
                use crate::graph::specs::SpecTrait;

                $(
                    // Ignore filtering for post-processing specs since they need to always run
                    if stringify!($field) != "cirro_post_processing_specs" {
                        self.$field = self
                            .$field
                            .into_iter()
                            .filter(|spec| labels.contains(&spec.get_label().to_string()))
                            .collect();
                    }
                )*

                self
            }

            /// Create a new SpecRegistry with the provided spec vectors
            /// This is a convenience constructor to make building the registry clearer
            pub fn new(
                $(
                    $field: Vec<$type>,
                )*
            ) -> Self {
                Self {
                    $(
                        $field,
                    )*
                }
            }
        }
    };
}

// Define the registry with all spec types
define_spec_registry! {
    cirro_azure_specs: CirroAzureIngestSpec,
    cirro_tailscale_status_specs: CirroTailscaleStatusIngestSpec,
    cirro_post_processing_specs: CirroPostProcessingSpec,
}

/// Unified spec loader that can load any spec type
pub struct SpecLoader;

impl SpecLoader {
    /// Load specs of type T using the provided config
    pub fn load<T>(config: &SpecConfig) -> Result<Vec<T>, CirroError>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let source = CirroSpecSource::new();

        source.load_specs(config.path_prefix).map_err(|e| {
            CirroError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Failed to load config '{}' from path '{}': {}",
                    config.name, config.path_prefix, e
                ),
            ))
        })
    }

    /// Load all spec types automatically
    pub fn load_all_specs() -> Result<SpecRegistry, CirroError> {
        Self::load_all_specs_filtered(None)
    }

    /// Load all spec types and filter by labels if provided
    ///
    /// When adding a new spec type:
    /// 1. Add it to the define_spec_registry! macro invocation above
    /// 2. Add the loading logic for it in this method
    /// 3. Add the field to the SpecRegistry::new() call below
    pub fn load_all_specs_filtered(
        labels: Option<Vec<String>>,
    ) -> Result<SpecRegistry, CirroError> {
        let mut errors = Vec::new();

        // Load each spec type - add new spec types here
        let cirro_azure_specs = match Self::load(&CIRRO_AZURE_SPEC_CONFIG) {
            Ok(specs) => specs,
            Err(e) => {
                errors.push(e);
                Vec::new()
            }
        };

        let cirro_tailscale_status_specs = match Self::load(&CIRRO_TAILSCALE_STATUS_SPEC_CONFIG) {
            Ok(specs) => specs,
            Err(e) => {
                errors.push(e);
                Vec::new()
            }
        };

        let cirro_post_processing_specs = match Self::load(&CIRRO_POST_PROCESSING_SPEC_CONFIG) {
            Ok(specs) => specs,
            Err(e) => {
                errors.push(e);
                Vec::new()
            }
        };

        if !errors.is_empty() {
            return Err(CirroError::MultipleErrors(errors));
        }

        // Construct the registry - the order must match the macro definition
        let registry = SpecRegistry::new(
            cirro_azure_specs,
            cirro_tailscale_status_specs,
            cirro_post_processing_specs,
        );

        // Filter by labels if provided (automatic via macro-generated method)
        Ok(match labels {
            Some(labels) => registry.filter_by_labels(&labels),
            None => registry,
        })
    }

    /// Get information about all registered spec configurations
    pub fn list_all_configs() -> Vec<&'static SpecConfig> {
        ALL_SPEC_CONFIGS.to_vec()
    }
}
