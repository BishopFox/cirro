package core

func (i *CirroIngestor) ProcessAutomationAccounts() error {

	const resource_type = "microsoft.automation/automationaccounts"

	var properties = []string{
		"id",
		"kind",
		"location",
		"name",
		"properties",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch AS row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:AutomationAccount:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			type: row.type,
			creationTime: row.properties.creationTime,
			lastModifiedTime: row.properties.lastModifiedTime
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)

}
