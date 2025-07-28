use crate::errors::CirroIngestError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process sql servers
    pub async fn process_sql_servers(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.sql/servers";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:SqlServer
            SET obj += {
                administratorLogin: row.properties.administratorLogin,
                fullyQualifiedDomainName: row.properties.fullyQualifiedDomainName,
                publicNetworkAccess: row.properties.publicNetworkAccess,
                restrictOutboundNetworkAccess: row.properties.restrictOutboundNetworkAccess,
                state: row.properties.state,
                version: row.properties.version
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process sql databases
    // TODO: The server this database belongs to should come from the id property, not managedBy.
    // https://github.com/memgraph/mage/issues/642
    pub async fn process_sql_databases(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.sql/servers/databases";
        let properties = vec!["/id", "/properties", "/managedBy"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:SqlDatabase
            SET obj += {
                collation: row.properties.collation,
                creationDate: row.properties.creationDate,
                currentServiceObjectiveName: row.properties.currentServiceObjectiveName,
                databaseId: row.properties.databaseId,
                isInfraEncryptionEnabled: row.properties.isInfraEncryptionEnabled,
                maxSizeBytes: row.properties.maxSizeBytes,
                status: row.properties.status
            }
            WITH obj, row WHERE row.managedBy IS NOT NULL
            MERGE (s:ArmResource {id: toLower(row.managedBy)})
            MERGE (s)-[:HAS_DB]->(obj)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
