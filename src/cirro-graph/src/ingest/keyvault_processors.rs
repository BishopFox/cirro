use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process key vaults
    pub async fn process_keyvaults(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.keyvault/vaults";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:KeyVault
            SET obj += {
                enableRbacAuthorization: row.properties.enableRbacAuthorization,
                enableSoftDelete: row.properties.enableSoftDelete,
                enabledForDeployment: row.properties.enabledForDeployment,
                publicNetworkAccess: row.properties.publicNetworkAccess,
                softDeleteRetentionInDays: row.properties.softDeleteRetentionInDays,
                vaultUri: row.properties.vaultUri
            }

            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.properties.accessPolicies, []) AS policy
                    MERGE (p:GraphObject {id: policy.objectId})
                    MERGE (p)-[r:HAS_POLICY]->(obj)

                    SET r += {
                        certificates: policy.permissions.certificates,
                        keys: policy.permissions.keys,
                        secrets: policy.permissions.secrets
                    }
                    RETURN count(*) AS _
                }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
