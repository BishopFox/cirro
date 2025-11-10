use crate::collectors::arm::paged_arm_request;
use crate::{collect::Collector, errors::CirroError};
use log::{debug, info, warn};
use serde_json::{Map, Value};
use std::path::PathBuf;
use std::{fs::File, io::BufReader};

#[derive(serde::Deserialize, Debug, Clone)]
pub struct EnrichConfig {
    pub id: String,
    pub need_graph_token: bool,
    pub need_arm_token: bool,
    pub checks: Map<String, Value>,
}

impl EnrichConfig {
    /// Load enrichment configuration from a JSON file
    pub fn from_file(path: PathBuf) -> Result<EnrichConfig, CirroError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let config: EnrichConfig = serde_json::from_reader(reader)?;
        Ok(config)
    }
}

impl Collector {
    pub async fn launch_enrichments(&self) -> Result<(), CirroError> {
        if let Some(enrich_config) = &self.enrich_config {
            // Storage Account Key Enrichment
            if let Some(storage_accounts) = enrich_config
                .checks
                .get("storage_account_keys")
                .and_then(|v| v.as_array())
            {
                let storage_accounts: Vec<String> = storage_accounts
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                if !storage_accounts.is_empty() {
                    self.enrich_storage_account_keys(storage_accounts).await?;
                }
            }

            // More checks can be added here in the future
        } else {
            warn!("No enrichment configuration provided");
        }
        Ok(())
    }

    /// Fetch storage account keys
    pub async fn enrich_storage_account_keys(
        &self,
        storage_accounts: Vec<String>,
    ) -> Result<(), CirroError> {
        info!(
            "Performing storage account key enrichment for {} accounts",
            storage_accounts.len()
        );
        for storage_account_id in storage_accounts {
            let uri = format!(
                "{}{}/listKeys?api-version=2024-01-01",
                self.cloud_endpoints.arm_url, storage_account_id,
            );

            // Make the paginated request to fetch the keys
            match paged_arm_request(self, &uri, reqwest::Method::POST, None).await {
                Ok(response) => {
                    // Print the keys
                    info!("Fetched keys for storage account {}", storage_account_id);
                    debug!(
                        "Response: {}",
                        serde_json::to_string_pretty(&response).unwrap_or_default()
                    );
                    // Process the response and write to the database
                    self.write_enrichment_to_db(
                        "storage_account_keys".to_string(),
                        storage_account_id.clone(),
                        response.clone(),
                    )
                    .await?;
                }
                Err(e) => {
                    warn!("Failed to fetch keys for {}", storage_account_id);
                    debug!("Error: {}", e);
                }
            }
        }
        Ok(())
    }
}
