use crate::errors::CirroIngestError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process cognitive services accounts
    pub async fn process_cognitive_services_account(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.cognitiveservices/accounts";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:CognitiveServicesAccount
            SET obj += {
                customSubdomain: row.properties.customSubDomainName,
                endpoint: row.properties.endpoint,
                publicNetworkAccess: row.properties.publicNetworkAccess
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
