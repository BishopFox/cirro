use crate::errors::CirroError;
use crate::graph::ingest::ingestor::CirroIngestor;
use log::{debug, info};
use neo4rs::{BoltType, query};
use serde_json;

impl CirroIngestor {
    /// Process ingest
    pub async fn process_cirro_tailscale_status_ingest(
        &self,
        dry_run: bool,
    ) -> Result<(), CirroError> {
        info!(
            "Starting Cirro Tailscale Status ingest on file: {:?}",
            self.file.as_path().file_name().unwrap()
        );

        // Load the JSON data from the specified file
        let data = tokio::fs::read_to_string(&self.file)
            .await
            .map_err(|e| CirroError::IoError(e))?;

        // Parse the JSON data
        let json_data: serde_json::Value = serde_json::from_str(&data).map_err(|e| {
            CirroError::InvalidData(format!("Failed to parse Tailscale status JSON data: {}", e))
        })?;

        let bolt_data = BoltType::try_from(json_data)?;

        // Process the Tailscale Status spec
        // Status could be written in one spec, but multiple specs are used for modularity
        for spec in &self.specs.cirro_tailscale_status_specs {
            // If spec has a label, create constraints and indexes
            if !spec.label.is_empty() {
                if dry_run {
                    debug!(
                        "[dry-run] Would create constraints/indexes for label {}",
                        spec.label
                    );
                } else {
                    self.create_constraints_and_indexes_by_spec(spec).await?;
                }
            } else {
                debug!(
                    "Spec {} does not have a label defined, skipping constraint and index creation",
                    spec.name
                );
            }

            if dry_run {
                info!("[dry-run] Would process Tailscale spec: {}", spec.name);
                continue;
            }

            let mut result = self
                .graph
                .execute(query(&spec.cypher).param("status_data", bolt_data.clone()))
                .await
                .map_err(|e| {
                    CirroError::DatabaseError(format!(
                        "Failed to execute Tailscale status ingest query: {}",
                        e
                    ))
                })?;

            // There is only row returned with count
            let row = result.next().await.map_err(|e| {
                CirroError::DatabaseError(format!(
                    "Error retrieving row from Tailscale status ingest query: {}",
                    e
                ))
            })?;

            let count: i64 = row.unwrap().get("count").map_err(|e| {
                CirroError::DatabaseError(format!(
                    "Failed to convert count to i64 in Tailscale status ingest query result: {}",
                    e
                ))
            })?;

            info!("Processed {:>5} : {}", count, spec.name);
        }

        if dry_run {
            info!("[dry-run] Skipping Tailscale ingestion transaction finalization");
            return Ok(());
        }

        // Ensure all Tailscale ingestion transactions are committed
        debug!("Finalizing Tailscale ingestion transactions...");
        let mut final_txn = self.graph.start_txn().await.map_err(|e| {
            CirroError::DatabaseError(format!(
                "Failed to start Tailscale finalization transaction: {}",
                e
            ))
        })?;
        final_txn
            .run(neo4rs::query("RETURN 1"))
            .await
            .map_err(|e| {
                CirroError::DatabaseError(format!(
                    "Failed to execute Tailscale finalization query: {}",
                    e
                ))
            })?;
        final_txn.commit().await.map_err(|e| {
            CirroError::DatabaseError(format!(
                "Failed to commit Tailscale finalization transaction: {}",
                e
            ))
        })?;

        Ok(())
    }
}
