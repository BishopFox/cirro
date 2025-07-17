package core

func (i *CirroIngestor) ProcessCommunicationServices() error {

	const resource_type = "microsoft.communication/communicationservices"

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
		SET obj:CommunicationServices:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			type: row.type,
			dataLocation: row.properties.dataLocation,
			hostName: row.properties.hostName,
			immutableResourceId: row.properties.immutableResourceId
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)

}
