package core

func (i *CirroIngestor) ProcessContainerRegistries() error {
	const resource_type = "microsoft.containerregistry/registries"

	var properties = []string{
		"id",
		"location",
		"name",
		"properties",
		"sku",
		"systemData",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch AS row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:StorageAccount:ArmResource
		SET obj += {
			id: row.id,
			sku: row.sku.name,
			location: row.location,
			name: row.name,
			type: row.type,
			adminUserEnabled: row.properties.adminUserEnabled,
			anonymousPullEnabled: row.properties.anonymousPullEnabled,
			creationDate: row.properties.creationDate,
			dataEndpointEnabled: row.properties.dataEndpointEnabled,
			dataEndpointHostNames: row.properties.dataEndpointHostNames,
			loginServer: row.properties.loginServer,
			metadataSearch: row.properties.metadataSearch,
			networkRuleBypassOptions: row.properties.networkRuleBypassOptions,
			publicNetworkAccess: row.properties.publicNetworkAccess,
			createdAt: row.systemData.createdAt,
			createdBy: row.systemData.createdBy,
			createdByType: row.systemData.createdByType,
			lastModifiedAt: row.systemData.lastModifiedAt,
			lastModifiedBy: row.systemData.lastModifiedBy,
			lastModifiedByType: row.systemData.lastModifiedByType
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}
