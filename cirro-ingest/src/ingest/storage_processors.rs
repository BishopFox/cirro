use crate::errors::CirroIngestError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process storage accounts
    pub async fn process_storage_accounts(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.storage/storageaccounts";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:StorageAccount
            SET obj += {
                accessTier: row.properties.accessTier,
                primaryLocation : row.properties.primaryLocation,
                publicNetworkAccess: row.properties.publicNetworkAccess,
                defaultToOAuthAuthentication: row.properties.defaultToOAuthAuthentication,
                allowBlobPublicAccess: row.properties.allowBlobPublicAccess,
                allowCrossTenantReplication: row.properties.allowCrossTenantReplication,
                allowSharedKeyAccess: row.properties.allowSharedKeyAccess,
                supportsHttpsTrafficOnly: row.properties.supportsHttpsTrafficOnly,
                minimumTlsVersion: row.properties.minimumTlsVersion,
                networkAclBypass: row.properties.networkAcls.bypass,
                networkAclDefaultAction: row.properties.networkAcls.defaultAction
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process classic storage accounts
    pub async fn process_classic_storage_accounts(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.classicstorage/storageaccounts";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:ClassicStorageAccount
            SET obj += {
                accountType: row.properties.accountType,
                creationTime: row.properties.creationTime,
                geoPrimaryRegion: row.properties.geoPrimaryRegion,
                geoSecondaryRegion: row.properties.geoSecondaryRegion,
                statusOfPrimaryRegion: row.properties.statusOfPrimaryRegion,
                statusOfSecondaryRegion: row.properties.statusOfSecondaryRegion,
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
