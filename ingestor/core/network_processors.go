package core

func (i *CirroIngestor) ProcessDnsZones() error {
	const resource_type = "microsoft.network/dnszones"

	var properties = []string{
		"id",
		"kind",
		"location",
		"name",
		"properties",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:DNSZone:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			type: row.type,
			nameServers: row.properties.nameServers
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}

func (i *CirroIngestor) ProcessPublicIpAddresses() error {
	const resource_type = "microsoft.network/publicipaddresses"

	var properties = []string{
		"id",
		"kind",
		"location",
		"name",
		"properties",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:PublicIPAddress:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			type: row.type,
			idleTimeoutInMinutes: row.properties.idleTimeoutInMinutes,
			ipAddress: row.properties.ipAddress,
			publicIPAddressVersion: row.properties.publicIPAddressVersion,
			publicIPAllocationMethod: row.properties.publicIPAllocationMethod
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)
}

func (i *CirroIngestor) ProcessNetworkInterfaces() error {
	const resource_type = "microsoft.network/networkinterfaces"

	var properties = []string{
		"id",
		"kind",
		"location",
		"name",
		"properties",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:NetworkInterface:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			type: row.type,
			macAddress: row.properties.macAddress,
			allowPort25Out: row.properties.allowPort25Out,
			appliedDnsServers: row.properties.dnsSettings.appliedDnsServers,
			dnsServers: row.properties.dnsSettings.dnsServers
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)

		FOREACH  (ipconfig in row.properties.ipConfigurations |
			MERGE (ipcon:IpConfiguration {id: ipconfig.id})
			MERGE (obj)-[:HAS_IPCONFIG]->(ipcon)

			SET ipcon += {
				name: ipconfig.name,
				type: ipconfig.type,
				privateIPAddress: ipconfig.properties.privateIPAddress,
				privateIPAddressVersion: ipconfig.properties.privateIPAddressVersion,
				privateIPAllocationMethod: ipconfig.properties.privateIPAllocationMethod
			}

			MERGE (subnet:Subnet {id: ipconfig.properties.subnet.id})
			MERGE (subnet)-[:CONTAINS]->(ipcon)
			
			FOREACH(_ IN CASE WHEN ipconfig.properties.publicIPAddress.id IS NOT NULL THEN [1] ELSE [] END|
				MERGE (publicIP:ArmResource {id: ipconfig.properties.publicIPAddress.id})
				SET publicIP:PublicIPAddress:ArmResource
				MERGE (ipcon)-[:HAS_IP]->(publicIP)
			)

		)
	WITH obj, row
		WHERE row.properties.virtualMachine.id IS NOT NULL
		MERGE (vm:ArmResource {id: row.properties.virtualMachine.id})
		SET vm:VirtualMachine:ArmResource
		MERGE (vm)-[:HAS_INTERFACE]->(obj)
	
	WITH obj, row
		WHERE row.properties.networkSecurityGroup.id IS NOT NULL
		MERGE (nsg:ArmResource {id: row.properties.networkSecurityGroup.id})
		SET nsg:NSG:ArmResource
		MERGE (obj)-[:HAS_NSG]->(nsg)
	`
	return i.ProcessSpecificArmResource(query, properties, resource_type)
}

func (i *CirroIngestor) ProcessNetworkSecurityGroups() error {
	const resource_type = "microsoft.network/networksecuritygroups"

	var properties = []string{
		"id",
		"kind",
		"location",
		"name",
		"properties",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:NSG:ArmResource
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

		FOREACH  (rule in row.properties.defaultSecurityRules |
			MERGE (nsgrule:NSGRule {id: rule.id})
			MERGE (obj)-[:HAS_RULE]->(nsgrule)
			SET nsgrule += {
				name: rule.name,
				type: rule.type,
				access: rule.properties.access,
				description: rule.properties.description,
				destinationAddressPrefix: rule.properties.destinationAddressPrefix,
				destinationAddressPrefixes: rule.properties.destinationAddressPrefixes,
				destinationPortRange: rule.properties.destinationPortRange,
				destinationPortRanges: rule.properties.destinationPortRanges,
				direction: rule.properties.direction,
				priority: rule.properties.priority,
				protocol: rule.properties.protocol,
				sourceAddressPrefix: rule.properties.sourceAddressPrefix,
				sourceAddressPrefixes: rule.properties.sourceAddressPrefixes,
				sourcePortRange: rule.properties.sourcePortRange,
				sourcePortRanges: rule.properties.sourcePortRanges
			}
		)

		FOREACH  (rule in row.properties.securityRules |
			MERGE (nsgrule:NSGRule {id: rule.id})
			MERGE (obj)-[:HAS_RULE]->(nsgrule)
			SET nsgrule += {
				name: rule.name,
				type: rule.type,
				access: rule.properties.access,
				description: rule.properties.description,
				destinationAddressPrefix: rule.properties.destinationAddressPrefix,
				destinationAddressPrefixes: rule.properties.destinationAddressPrefixes,
				destinationPortRange: rule.properties.destinationPortRange,
				destinationPortRanges: rule.properties.destinationPortRanges,
				direction: rule.properties.direction,
				priority: rule.properties.priority,
				protocol: rule.properties.protocol,
				sourceAddressPrefix: rule.properties.sourceAddressPrefix,
				sourceAddressPrefixes: rule.properties.sourceAddressPrefixes,
				sourcePortRange: rule.properties.sourcePortRange,
				sourcePortRanges: rule.properties.sourcePortRanges
			}
		)
	`
	return i.ProcessSpecificArmResource(query, properties, resource_type)
}

func (i *CirroIngestor) ProcessVirtualNetworks() error {
	const resource_type = "microsoft.network/virtualnetworks"

	var properties = []string{
		"id",
		"location",
		"name",
		"properties",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:VirtualNetwork:ArmResource
		SET obj += {
			id: row.id,
			location: row.location,
			name: row.name,
			type: row.type,
			addressPrefixes: row.properties.addressSpace.addressPrefixes
		}
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)

		FOREACH  (subnet in row.properties.subnets |
			MERGE (s:Subnet {id: subnet.id})
			MERGE (obj)-[:CONTAINS]->(s)
			SET s += {
				name: subnet.name,
				type: subnet.type,
				addressPrefix: subnet.properties.addressPrefix,
				privateEndpointNetworkPolicies: subnet.properties.privateEndpointNetworkPolicies,
				privateLinkServiceNetworkPolicies: subnet.properties.privateLinkServiceNetworkPolicies
			}
		)
	`
	return i.ProcessSpecificArmResource(query, properties, resource_type)
}
