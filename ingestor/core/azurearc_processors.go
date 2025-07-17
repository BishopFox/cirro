package core

func (i *CirroIngestor) ProcessAzureArcSqlServers() error {
	const resource_type = "microsoft.azurearcdata/sqlserverinstances"

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
		SET obj:ArcSqlServer:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			type: row.type,
			azureDefenderStatus: row.properties.azureDefenderStatus,
			collation: row.properties.collation,
			currentVersion: row.properties.currentVersion,
			edition: row.properties.edition,
			instanceName: row.properties.instanceName,
			licenseType: row.properties.licenseType,
			patchLevel: row.properties.patchLevel,
			productId: row.properties.productId,
			status: row.properties.status,
			tcpDynamicPorts: row.properties.tcpDynamicPorts,
			tcpStaticPorts: row.properties.tcpStaticPorts,
			vCore: row.properties.vCore,
			version: row.properties.version
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)

		MERGE (hm:ArmResource {id: row.properties.containerResourceId})
		MERGE (hm)-[:HOSTS]->(obj)
	`
	return i.ProcessSpecificArmResource(query, properties, resource_type)

}
