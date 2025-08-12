use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process communication services
    pub async fn process_communication_services(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.communication/communicationservices";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:CommunicationServices
            SET obj += {
                dataLocation: row.properties.dataLocation,
			    hostName: row.properties.hostName,
			    immutableResourceId: row.properties.immutableResourceId
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
