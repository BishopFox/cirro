package core

func (i *CirroIngestor) ProcessCognitiveServicesAccount() error {

	const resource_type = "microsoft.cognitiveservices/accounts"

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
		SET obj:CognitiveServicesAccount:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			type: row.type,
			customSubdomain: row.properties.customSubDomainName,
			endpoint: row.properties.endpoint,
			publicNetworkAccess: row.properties.publicNetworkAccess
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)

}
