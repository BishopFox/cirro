use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process automation accounts
    pub async fn process_automation_accounts(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.automation/automationaccounts";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:AutomationAccount
            SET obj += {
                creationTime: row.properties.creationTime,
			    lastModifiedTime: row.properties.lastModifiedTime,
                registrationUrl: row.properties.registrationUrl,
                automationHybridServiceUrl: row.properties.automationHybridServiceUrl
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process automation runbooks
    pub async fn process_automation_runbooks(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.automation/automationaccounts/runbooks";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:AutomationRunbook
            SET obj += {
                creationTime: row.properties.creationTime,
			    lastModifiedTime: row.properties.lastModifiedTime,
                description: row.properties.description,
                jobCount: row.properties.jobCount,
                runbookType: row.properties.runbookType,
                state: row.properties.state,
                runtimeEnvironment: row.properties.runtimeEnvironment
            }

            WITH obj, row, split(toLower(obj.id), '/runbooks/')[0] AS runbookId
                MERGE (a:AutomationAccount {id: runbookId})
                MERGE (a)-[:HAS_RUNBOOK]->(obj)

            WITH obj, row
                CALL {
                    WITH obj, row
                    WITH obj, row, coalesce(row.properties.parameters, {}) AS parameters
                    UNWIND keys(parameters) AS paramName
                        WITH obj, paramName, parameters[paramName] AS paramMap

                        // Create a unique identifier for the parameter to avoid name collisions
                        MERGE (p:RunbookParameter {id: obj.id + "/param/" + paramName})
                        SET p += {
                            name: paramName,
                            defaultValue: paramMap.defaultValue,
                            isMandatory: paramMap.isMandatory,
                            position: paramMap.position,
                            type: paramMap.type
                        }
                        MERGE (obj)-[:HAS_PARAMETER]->(p)
                    RETURN count(*) AS _
                }
            RETURN count(*) AS _
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
