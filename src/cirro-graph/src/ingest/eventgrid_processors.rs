use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process event grid system topics
    pub async fn process_event_grid_system_topics(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.eventgrid/systemTopics";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:EventGridTopic
            SET obj += {
                topicType: row.properties.topicType
            }
            MERGE (obj)-[:HAS_SOURCE]->(s:ArmResource {id: row.properties.source})

        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process event grid topics
    pub async fn process_event_grid_topics(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.eventgrid/topics";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:EventGridTopic
            SET obj += {
                dataResidencyBoundary: row.properties.dataResidencyBoundary,
                disableLocalAuth: row.properties.disableLocalAuth,
                endpoint: row.properties.endpoint,
                inputSchema: row.properties.inputSchema,
                minimumTlsVersionAllowed: row.properties.minimumTlsVersionAllowed,
                publicNetworkAccess: row.properties.publicNetworkAccess
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
