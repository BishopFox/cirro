package core

func (i *CirroIngestor) ProcessStorageAccounts() error {
	const resource_type = "microsoft.storage/storageaccounts"

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
		SET obj:StorageAccount:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			accessTier: row.properties.accessTier,
			primaryLocation : row.properties.primaryLocation,
			publicNetworkAccess: row.properties.publicNetworkAccess,
			defaultToOAuthAuthentication: row.properties.defaultToOAuthAuthentication,
			allowBlobPublicAccess: row.properties.allowBlobPublicAccess,
			allowCrossTenantReplication: row.properties.allowCrossTenantReplication,
			allowSharedKeyAccess: row.properties.allowSharedKeyAccess,
			supportsHttpsTrafficOnly: row.properties.supportsHttpsTrafficOnly,
			minimumTlsVersion: row.properties.minimumTlsVersion,
			networkAclBypass: row.properties.networkAcls.bypass,
			networkAclDefaultAction: row.properties.networkAcls.defaultAction,
			type: row.type
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}

func (i *CirroIngestor) ProcessClassicStorageAccounts() error {
	const resource_type = "microsoft.classicstorage/storageaccounts"

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
		SET obj:ClassicStorageAccount:ArmResource
		SET obj += {
			id: row.id,
			location: row.location,
			name: row.name,
			accountType: row.properties.accountType,
			creationTime: row.properties.creationTime,
			geoPrimaryRegion: row.properties.geoPrimaryRegion,
			geoSecondaryRegion: row.properties.geoSecondaryRegion,
			statusOfPrimaryRegion: row.properties.statusOfPrimaryRegion,
			statusOfSecondaryRegion: row.properties.statusOfSecondaryRegion,
			type: row.type
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}
