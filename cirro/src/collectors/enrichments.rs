use crate::collectors::arm::paged_arm_request;
use crate::{collect::Collector, errors::CirroError};
use log::{debug, info, warn};
use rusqlite::Connection;

impl Collector {
    /// Fetch storage account keys
    pub async fn enrich_storage_account_keys(
        &self,
        sqlite_db: Connection,
    ) -> Result<(), CirroError> {
        // Get all the storage accounts from the database
        let query =
            "SELECT id FROM resources WHERE resource_type = 'microsoft.storage/storageaccounts'";

        let mut stmt = sqlite_db.prepare(query)?;
        let storage_accounts: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, rusqlite::Error>>()?;

        if storage_accounts.is_empty() {
            info!("No storage accounts found for storage key enrichment");
            return Ok(());
        }

        info!(
            "Found {} storage accounts for key enrichment",
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
                    info!(
                        "Fetched keys for storage account {}: \n{}",
                        storage_account_id,
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
