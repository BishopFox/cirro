use clap::ValueEnum;
use serde::{Deserialize, Serialize};
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Opengraph,
    Grass,
}

//
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenGraphJson {
    // https://bloodhound.specterops.io/opengraph/schema
    graph: serde_json::Value,
}
