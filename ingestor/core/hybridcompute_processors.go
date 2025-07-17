package core

func (i *CirroIngestor) ProcessHybridMachines() error {

	const resource_type = "microsoft.hybridcompute/machines"

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
		SET obj:HybridMachine:ArmResource
		SET obj += {
			id: row.id,
			location: row.location,
			name: row.name,
			type: row.type,
			adFqdn: row.properties.adFqdn,
			agentVersion: row.properties.agentVersion,
			clientPublicKey: row.properties.clientPublicKey,
			cloud: row.properties.cloudMetadata.provider,
			displayName: row.properties.displayName,
			dnsFqdn: row.properties.dnsFqdn,
			domainName: row.properties.domainName,
			lastStatusChange: row.properties.lastStatusChange,
			machineFqdn: row.properties.machineFqdn,
			osName: row.properties.osName,
			computerName: row.properties.osProfile.computerName,
			osSku: row.properties.osSku,
			osVersion: row.properties.osVersion,
			status: row.properties.status,
			vmId: row.properties.vmId,
			vmUuid: row.properties.vmUuid
		}

	WITH obj, row
	FOREACH (resource in row.resources |
		FOREACH (_ IN CASE WHEN lower(resource.type) = "microsoft.hybridcompute/machines/extensions" THEN [1] ELSE [] END |
			MERGE (res:HybridExtension {id: resource.id})
			SET res += {
				id: resource.id,
				name: resource.name,
				type: resource.type,
				location: resource.location,
				typeHandlerVersion: resource.properties.typeHandlerVersion,
				autoUpgradeMinorVersion: resource.properties.autoUpgradeMinorVersion,
				enableAutomaticUpgrade: resource.properties.enableAutomaticUpgrade,
				statusMessage: resource.properties.instanceView.status.message,
				provisioningState: resource.properties.provisioningState
			}
			MERGE (obj)-[:HAS_EXTENSION]->(res)
		)
	)

	FOREACH (interface in row.properties.networkProfile.networkInterfaces |
		FOREACH (ipAddress in interface.ipAddresses |
			MERGE (ip:HybridIPAddress {address: ipAddress.address})
			SET ip += {
				address: ipAddress.address,
				ipAddressVersion: ipAddress.ipAddressVersion,
				subnet: ipAddress.subnet.addressPrefix
			}
		)
	)
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}
