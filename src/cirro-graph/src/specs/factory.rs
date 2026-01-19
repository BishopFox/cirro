use crate::errors::CirroGraphError;
use crate::specs::azure::types::CirroAzureIngestSpec;
use crate::specs::configs::{
    ALL_SPEC_CONFIGS, CIRRO_AZURE_SPEC_CONFIG, GenericSpecSource, SpecConfig,
};

#[cfg(debug_assertions)]
use crate::specs::sources::GenericDiskSpecSource;

#[cfg(not(debug_assertions))]
use crate::specs::sources::GenericEmbeddedSpecSource;

/// Container for all loaded spec types
#[derive(Debug)]
pub struct SpecRegistry {
    pub cirro_azure_specs: Vec<CirroAzureIngestSpec>,
}

/// Unified spec loader that can load any spec type
pub struct SpecLoader;

impl SpecLoader {
    /// Internal helper to create a spec source
    fn create_source<T>() -> Box<dyn GenericSpecSource<T>>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        #[cfg(debug_assertions)]
        {
            Box::new(GenericDiskSpecSource::new(
                concat!(env!("CARGO_MANIFEST_DIR"), "/src/config/constants.yml").to_string(),
            ))
        }

        #[cfg(not(debug_assertions))]
        {
            Box::new(GenericEmbeddedSpecSource::new())
        }
    }

    /// Load specs of type T using the provided config
    pub fn load<T>(config: &SpecConfig) -> Result<Vec<T>, CirroGraphError>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let source = Self::create_source::<T>();

        #[cfg(debug_assertions)]
        let path = config.disk_path;
        #[cfg(not(debug_assertions))]
        let path = config.embedded_prefix;

        source.load_specs(path).map_err(|e| {
            CirroGraphError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Failed to load config '{}' from path '{}': {}",
                    config.name, path, e
                ),
            ))
        })
    }

    /// Load all spec types automatically
    pub fn load_all_specs() -> Result<SpecRegistry, CirroGraphError> {
        let mut errors = Vec::new();

        let cirro_azure_specs = match Self::load(&CIRRO_AZURE_SPEC_CONFIG) {
            Ok(specs) => specs,
            Err(e) => {
                errors.push(e);
                Vec::new()
            }
        };

        if !errors.is_empty() {
            return Err(CirroGraphError::MultipleErrors(errors));
        }

        Ok(SpecRegistry { cirro_azure_specs })
    }

    /// Get information about all registered spec configurations
    pub fn list_all_configs() -> Vec<&'static SpecConfig> {
        ALL_SPEC_CONFIGS.to_vec()
    }
}
