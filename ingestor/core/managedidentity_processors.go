package core

func (i *CirroIngestor) ProcessUserAssignedIdentities() error {
	const resource_type = "microsoft.managedidentity/userassignedidentities"

	var properties = []string{
		"id",
		"location",
		"name",
		"properties",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch AS row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:UAIdentity:ArmResource
		SET obj += {
			id: row.id,
			location: row.location,
			name: row.name,
			type: row.type,
			clientId: row.properties.clientId,
			tenantId: row.properties.tenantId
		}
		MERGE (p:GraphObject {id: row.properties.principalId})
		MERGE (p)-[:HAS_IDENTITY]->(obj)
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}
