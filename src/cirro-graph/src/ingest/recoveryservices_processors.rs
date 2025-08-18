use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process recovery services vaults
    pub async fn process_recovery_services_vaults(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.recoveryservices/vaults";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:RecoveryVault
            SET obj += {
                bcdrSecurityLevel: row.properties.bcdrSecurityLevel,
                privateEndpointStateForBackup: row.properties.privateEndpointStateForBackup,
                privateEndpointStateForSiteRecovery: row.properties.privateEndpointStateForSiteRecovery,
                publicNetworkAccess: row.properties.publicNetworkAccess,
                crossSubscriptionRestoreState: row.properties.restoreSettings.crossSubscriptionRestoreSettings.crossSubscriptionRestoreState,
                multiUserAuthorization: row.properties.securitySettings.multiUserAuthorization,
                softDeleteState: row.properties.securitySettings.softDeleteSettings.softDeleteState,
                softDeleteRetentionPeriodInDays: row.properties.securitySettings.softDeleteSettings.softDeleteRetentionPeriodInDays
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
