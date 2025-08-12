use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process user assigned identities
    pub async fn process_user_assigned_identities(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.managedidentity/userassignedidentities";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:UserAssignedIdentity
            SET obj += {
                clientId: row.properties.clientId,
                tenantId: row.properties.tenantId
            }
            MERGE (o:GraphObject {appId: row.properties.clientId})
            MERGE (obj)-[:HAS_IDENTITY]->(o)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
