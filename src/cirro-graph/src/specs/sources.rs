use crate::errors::CirroGraphError;
use crate::specs::configs::GenericSpecSource;
use serde::de::DeserializeOwned;
use serde_yaml::Value as YamlValue;
use tera::{Context, Tera};

/// Convert constants YAML to Tera context for template rendering
pub fn context_from_constants_yaml(constants_yaml: &str) -> Result<Context, CirroGraphError> {
    let yaml: YamlValue = serde_yaml::from_str(constants_yaml)?;
    let json = serde_json::to_value(yaml)?;
    Ok(Context::from_value(json)?)
}

/// Generic disk-based spec source that works with any spec type
#[cfg(debug_assertions)]
pub struct GenericDiskSpecSource {
    pub constants_path: String,
}

#[cfg(debug_assertions)]
impl GenericDiskSpecSource {
    pub fn new(constants_path: String) -> Self {
        Self { constants_path }
    }

    fn load_constants(&self) -> Result<Context, CirroGraphError> {
        let raw = std::fs::read_to_string(&self.constants_path).map_err(|e| {
            CirroGraphError::Config(format!(
                "Failed to read constants file '{}': {}",
                self.constants_path, e
            ))
        })?;
        context_from_constants_yaml(&raw)
            .map_err(|e| CirroGraphError::Config(format!("Failed to parse constants YAML: {}", e)))
    }
}

#[cfg(debug_assertions)]
impl<T> GenericSpecSource<T> for GenericDiskSpecSource
where
    T: DeserializeOwned,
{
    fn load_specs(&self, disk_path: &str) -> Result<Vec<T>, CirroGraphError> {
        let tera = Tera::new(disk_path).map_err(|e| {
            CirroGraphError::Config(format!(
                "Failed to create Tera instance with pattern '{}': {}",
                disk_path, e
            ))
        })?;

        let template_names: Vec<_> = tera.get_template_names().collect();
        if template_names.is_empty() {
            return Err(CirroGraphError::Config(format!(
                "No templates found with pattern '{}'",
                disk_path
            )));
        }

        let ctx = self.load_constants().map_err(|e| {
            CirroGraphError::Config(format!(
                "Failed to load constants from '{}': {}",
                self.constants_path, e
            ))
        })?;

        let mut specs = Vec::new();
        for template_name in &template_names {
            let rendered = tera.render(template_name, &ctx).map_err(|e| {
                CirroGraphError::Config(format!(
                    "Failed to render template '{}': {}",
                    template_name, e
                ))
            })?;

            let spec: T = serde_yaml::from_str(&rendered).map_err(|e| {
                CirroGraphError::Config(format!(
                    "Failed to parse YAML from template '{}': {}\nRendered content:\n{}",
                    template_name, e, rendered
                ))
            })?;
            specs.push(spec);
        }

        Ok(specs)
    }
}

/// Generic embedded spec source that works with any spec type
#[cfg(not(debug_assertions))]
use rust_embed::Embed;

#[cfg(not(debug_assertions))]
#[derive(Embed)]
#[folder = "src/config"]
struct EmbeddedConfig;

#[cfg(not(debug_assertions))]
pub struct GenericEmbeddedSpecSource;

#[cfg(not(debug_assertions))]
impl GenericEmbeddedSpecSource {
    pub fn new() -> Self {
        Self
    }

    fn load_constants(&self) -> Result<Context, CirroGraphError> {
        let file = EmbeddedConfig::get("constants.yaml")
            .ok_or_else(|| CirroGraphError::Config("Missing embedded constants.yaml".into()))?;
        let s = std::str::from_utf8(file.data.as_ref())?;
        context_from_constants_yaml(s)
    }

    fn load_tera(&self) -> Result<Tera, CirroGraphError> {
        let mut tera = Tera::default();

        for path in EmbeddedConfig::iter() {
            let path = path.as_ref();

            if !(path.ends_with(".yaml.tera")) {
                continue;
            }

            let file = EmbeddedConfig::get(path)
                .ok_or_else(|| CirroGraphError::Config(format!("Missing embedded file {path}")))?;
            let content = std::str::from_utf8(file.data.as_ref())?;

            tera.add_raw_template(path, content)?;
        }

        Ok(tera)
    }
}

#[cfg(not(debug_assertions))]
impl<T> GenericSpecSource<T> for GenericEmbeddedSpecSource
where
    T: DeserializeOwned,
{
    fn load_specs(&self, embedded_prefix: &str) -> Result<Vec<T>, CirroGraphError> {
        let tera = self.load_tera()?;
        let ctx = self.load_constants()?;

        let mut specs = Vec::new();
        for template_name in tera.get_template_names() {
            if !template_name.starts_with(embedded_prefix) {
                continue;
            }
            let rendered = tera.render(template_name, &ctx)?;
            let spec: T = serde_yaml::from_str(&rendered)?;
            specs.push(spec);
        }

        Ok(specs)
    }
}
