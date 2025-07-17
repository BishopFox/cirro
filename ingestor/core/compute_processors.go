package core

func (i *CirroIngestor) ProcessAvailabilitySets() error {

	const resource_type = "microsoft.compute/availabilitysets"

	var properties = []string{
		"id",
		"location",
		"name",
		"properties",
		"sku",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch AS row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:AvailabilitySet:ArmResource
		SET obj += {
			id: row.id,
			location: row.location,
			name: row.name,
			sku: row.sku.name,
			type: row.type
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	WITH obj, row
		WHERE row.managedBy IS NOT NULL
			MERGE (managedBy:ArmResource {id: row.managedBy})
			MERGE (managedBy)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}

func (i *CirroIngestor) ProcessDisks() error {

	const resource_type = "microsoft.compute/disks"

	var properties = []string{
		"id",
		"properties",
		"managedBy",
	}

	const query = `/*cypher*/
	UNWIND $batch AS row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:Disk:ArmResource
		SET obj += {
			managedBy: row.managedBy,
			diskSizeGB: row.properties.diskSizeGB,
			diskState: row.properties.diskState,
			osType: row.properties.osType,
			networkAccessPolicy: row.properties.networkAccessPolicy,
			publicNetworkAccess: row.properties.publicNetworkAccess
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	WITH obj, row
		WHERE row.managedBy IS NOT NULL
			MERGE (managedBy:ArmResource {id: row.managedBy})
			MERGE (managedBy)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}

func (i *CirroIngestor) ProcessRestorePointCollections() error {

	const resource_type = "microsoft.compute/restorepointcollections"

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
		SET obj:RestorePointCollection:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			type: row.type
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)

		MERGE (vm:ArmResource {id: row.properties.source.id})
		MERGE (vm)-[:HAS_RESTOREPOINT]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)

}

func (i *CirroIngestor) ProcessSnapshots() error {

	const resource_type = "microsoft.compute/snapshots"

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
		SET obj:Snapshot:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			type: row.type,
			createOption: row.properties.creationData.createOption,
			dataAccessAuthMode: row.properties.dataAccessAuthMode,
			diskSizeBytes: row.properties.diskSizeBytes,
			diskSizeGB: row.properties.diskSizeGB,
			diskState: row.properties.diskState,
			hyperVGeneration: row.properties.hyperVGeneration,
			incremental: row.properties.incremental,
			networkAccessPolicy: row.properties.networkAccessPolicy,
			osType: row.properties.osType,
			publicNetworkAccess: row.properties.publicNetworkAccess
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)

		MERGE (disk:ArmResource {id: row.properties.creationData.sourceResourceId})
		MERGE (disk)-[:HAS_SNAPSHOT]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)

}

func (i *CirroIngestor) ProcessSSHPublicKeys() error {
	const resource_type = "microsoft.compute/sshpublickeys"

	var properties = []string{
		"id",
		"properties",
	}

	const query = `/*cypher*/
	UNWIND $batch AS row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:Disk:ArmResource
		SET obj += {
			publicKey: row.properties.publicKey
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}

func (i *CirroIngestor) ProcessVirtualMachines() error {
	const resource_type = "microsoft.compute/virtualmachines"

	var properties = []string{
		"id",
		"location",
		"name",
		"properties",
		"resources",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch AS row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:VirtualMachine:ArmResource
		SET obj += {
			id: row.id,
			location: row.location,
			name: row.name,
			type: row.type,
			vmSize: row.properties.hardwareProfile.vmSize,
			adminUsername: row.properties.osProfile.adminUsername,
			allowExtensionOperations: row.properties.osProfile.allowExtensionOperations,
			computerName: row.properties.osProfile.computerName,
			vmId: row.properties.vmId,
			os: row.properties.storageProfile.osDisk.osType,
			image: row.properties.storageProfile.imageReference.offer,
			imageVersion: row.properties.storageProfile.imageReference.exactVersion
		}
	FOREACH (interface in row.properties.networkProfile.networkInterfaces |
		MERGE (nic:ArmResource {id: interface.id})
		MERGE (obj)-[:HAS_NIC]->(nic)
	)

	WITH obj, row
	FOREACH (resource in row.resources |
		FOREACH (_ IN CASE WHEN lower(resource.type) = "microsoft.compute/virtualmachines/extensions" THEN [1] ELSE [] END |
			MERGE (res:VMExtension {id: resource.id})
			SET res += {
				id: resource.id,
				name: resource.name,
				type: resource.type,
				location: resource.location,
				provisioningState: resource.properties.provisioningState
			}
			MERGE (obj)-[:HAS_EXTENSION]->(res)
		)
	)
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}

func (i *CirroIngestor) ProcessVirtualMachineExtensions() error {
	const resource_type = "microsoft.compute/virtualmachines/extensions"

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
		SET obj:VMExtension:ArmResource
		SET obj += {
			id: row.id,
			location: row.location,
			name: row.name,
			type: row.type,
			provisioningState: row.properties.provisioningState
		}
	WITH obj, row
		MERGE (vm:ArmResource {id: row.vm_id})
		MERGE (vm)-[:HAS_EXTENSION]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}
