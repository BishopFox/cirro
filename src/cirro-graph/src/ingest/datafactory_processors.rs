use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process data factories
    pub async fn process_datafactories(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.datafactory/factories";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:DataFactory
            SET obj += {
                createTime: row.properties.createTime,
                publicNetworkAccess: row.properties.publicNetworkAccess,
                version: row.properties.version,
                repoAccountName: row.properties.repoConfiguration.accountName,
                repoBranch: row.properties.repoConfiguration.collaborationBranch,
                repoDisablePublish: row.properties.repoConfiguration.disablePublish,
                repoHostname: row.properties.repoConfiguration.hostname,
                repoLastCommitId: row.properties.repoConfiguration.lastCommitId,
                repoName: row.properties.repoConfiguration.repositoryName
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
