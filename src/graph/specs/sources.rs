use crate::errors::CirroError;
use rust_embed::Embed;
use serde::de::DeserializeOwned;
use serde_yaml::Value as YamlValue;
use tera::{Context, Tera};

/// Convert constants YAML to Tera context for template rendering
pub fn context_from_constants_yaml(constants_yaml: &str) -> Result<Context, CirroError> {
    let yaml: YamlValue = serde_yaml::from_str(constants_yaml)?;
    let json = serde_json::to_value(yaml)?;
    Ok(Context::from_value(json)?)
}

// In debug mode, rust-embed reads from disk; in release mode, files are embedded
#[derive(Embed)]
#[folder = "src/graph/config"]
struct EmbeddedConfig;

pub struct CirroSpecSource;

impl CirroSpecSource {
    pub fn new() -> Self {
        Self
    }

    fn load_constants(&self) -> Result<Context, CirroError> {
        let file = EmbeddedConfig::get("constants.yaml")
            .ok_or_else(|| CirroError::Config("Missing constants.yaml".into()))?;
        let s = std::str::from_utf8(file.data.as_ref())?;
        context_from_constants_yaml(s)
    }

    fn load_tera(&self) -> Result<Tera, CirroError> {
        let mut tera = Tera::default();

        for path in EmbeddedConfig::iter() {
            let path = path.as_ref();

            if !path.ends_with(".tera.yaml") {
                continue;
            }

            let file = EmbeddedConfig::get(path)
                .ok_or_else(|| CirroError::Config(format!("Missing file {path}")))?;
            let content = std::str::from_utf8(file.data.as_ref())?;

            tera.add_raw_template(path, content)?;
        }

        Ok(tera)
    }

    pub fn load_specs<T>(&self, prefix: &str) -> Result<Vec<T>, CirroError>
    where
        T: DeserializeOwned,
    {
        let tera = self.load_tera()?;
        let ctx = self.load_constants()?;

        let mut specs = Vec::new();
        for template_name in tera.get_template_names() {
            if !template_name.starts_with(prefix) {
                continue;
            }
            let rendered = tera.render(template_name, &ctx)?;
            let spec: T = serde_yaml::from_str(&rendered)?;
            specs.push(spec);
        }

        Ok(specs)
    }
}
