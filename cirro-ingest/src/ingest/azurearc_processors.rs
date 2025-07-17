use crate::errors::CirroIngestError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process storage accounts
    pub async fn process_azurearc_sql_servers(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.azurearcdata/sqlserverinstances";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:AzureArcSqlServer
            SET obj += {
                azureDefenderStatus: row.properties.azureDefenderStatus,
                collation: row.properties.collation,
                currentVersion: row.properties.currentVersion,
                edition: row.properties.edition,
                instanceName: row.properties.instanceName,
                licenseType: row.properties.licenseType,
                patchLevel: row.properties.patchLevel,
                productId: row.properties.productId,
                status: row.properties.status,
                tcpDynamicPorts: row.properties.tcpDynamicPorts,
                tcpStaticPorts: row.properties.tcpStaticPorts,
                vCore: row.properties.vCore,
                version: row.properties.version
            }

            MERGE (h:ArmResource {id: row.properties.containerResourceId})
            MERGE (h)-[:HOSTS]->(obj)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
