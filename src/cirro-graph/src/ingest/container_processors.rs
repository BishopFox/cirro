use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process container registries
    pub async fn process_container_registries(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.containerregistry/registries";
        let properties = vec!["/id", "/sku", "/systemData", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:ContainerRegistry
            SET obj += {
                sku: row.sku.name,
                adminUserEnabled: row.properties.adminUserEnabled,
                anonymousPullEnabled: row.properties.anonymousPullEnabled,
                creationDate: row.properties.creationDate,
                dataEndpointEnabled: row.properties.dataEndpointEnabled,
                dataEndpointHostNames: row.properties.dataEndpointHostNames,
                loginServer: row.properties.loginServer,
                metadataSearch: row.properties.metadataSearch,
                networkRuleBypassOptions: row.properties.networkRuleBypassOptions,
                publicNetworkAccess: row.properties.publicNetworkAccess,
                createdAt: row.systemData.createdAt,
                createdBy: row.systemData.createdBy,
                createdByType: row.systemData.createdByType,
                lastModifiedAt: row.systemData.lastModifiedAt,
                lastModifiedBy: row.systemData.lastModifiedBy,
                lastModifiedByType: row.systemData.lastModifiedByType
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
