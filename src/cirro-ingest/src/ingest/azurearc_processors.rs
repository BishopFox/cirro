use crate::errors::CirroIngestError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process azure arc sql servers
    pub async fn process_azurearc_sql_servers(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.azurearcdata/sqlserverinstances";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:ArcSqlServer
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

    // Process azure arc sql server db
    // Disabled for now
    // https://github.com/memgraph/mage/issues/642
    // pub async fn process_azurearc_sql_database(&self) -> Result<(), CirroIngestError> {
    //     let resource_type = "microsoft.azurearcdata/sqlserverinstances/databases";
    //     let properties = vec!["/id", "/properties"];

    //     let node_insert_query = r#"
    //     UNWIND $batch AS row
    //     MERGE (obj:ArmResource {id: row.id})
    //     SET obj:AzureArcSqlDB
    //     SET obj += {
    //         dataFileSizeMB: row.properties.dataFileSizeMB
    //         databaseCreationDate: row.properties.databaseCreationDate
    //         lastDatabaseUploadTime: row.properties.lastDatabaseUploadTime
    //         state: row.properties.state

    //     WITH obj, row, split(row.id, '/')[7] AS instance_id
    //         WITH obj, row, text.join(instance_id,

    //     }
    //     "#;

    //     Ok(())
    // }
}
