use crate::errors::CirroIngestError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process dns zones
    pub async fn process_dns_zones(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.network/dnszones";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:DnsZone
            SET obj += {
                nameServers: row.properties.nameServers,
                numberOfRecordSets: row.properties.numberOfRecordSets,
                zoneType: row.properties.zoneType
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process public IP addresses
    pub async fn process_public_ip_addresses(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.network/publicipaddresses";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:PublicIPAddress
            SET obj += {
                idleTimeoutInMinutes: row.properties.idleTimeoutInMinutes,
                ipAddress: row.properties.ipAddress,
                publicIPAddressVersion: row.properties.publicIPAddressVersion,
                publicIPAllocationMethod: row.properties.publicIPAllocationMethod
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process network interfaces
    pub async fn process_network_interfaces(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.network/networkinterfaces";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:NetworkInterface
            SET obj += {
                macAddress: row.properties.macAddress,
                allowPort25Out: row.properties.allowPort25Out,
                appliedDnsServers: row.properties.dnsSettings.appliedDnsServers,
                dnsServers: row.properties.dnsSettings.dnsServers
            }

            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.properties.ipConfigurations, []) AS ipconfig
                    MERGE (ipconnode:IPConfig {id: ipconfig.id})
                    SET ipconnode += {
                        name: ipconfig.name,
                        type: ipconfig.type,
                        privateIPAddress: ipconfig.properties.privateIPAddress,
                        privateIPAddressVersion: ipconfig.properties.privateIPAddressVersion,
                        privateIPAllocationMethod: ipconfig.properties.privateIPAllocationMethod
                    }
                    MERGE (obj)-[:HAS_CONFIG]->(ipconnode)
                    
                    MERGE (subnet:Subnet {id: ipconfig.properties.subnet.id})
                    MERGE (subnet)-[:CONTAINS]->(ipconnode)

                    WITH obj, row, ipconnode, ipconfig WHERE ipconfig.properties.publicIPAddress IS NOT NULL
                        MERGE (pubip:ArmResource {id: ipconfig.properties.publicIPAddress.id})
                        SET pubip:PublicIPAddress
                        MERGE (ipconnode)-[:HAS_IP]->(pubip)
                    RETURN count(*) AS _
                }

            WITH obj, row WHERE row.properties.virtualMachine.id IS NOT NULL
                MERGE (vm:ArmResource {id: row.properties.virtualMachine.id})
                SET vm:VirtualMachine
                MERGE (vm)-[:HAS_NIC]->(obj) 
            
            WITH obj, row WHERE row.properties.networkSecurityGroup IS NOT NULL
                MERGE (nsg:ArmResource {id: row.properties.networkSecurityGroup.id})
                SET nsg:NSG
                MERGE (obj)-[:HAS_NSG]->(nsg)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process network security groups
    pub async fn process_network_security_groups(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.network/networksecuritygroups";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:NSG

            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.properties.defaultSecurityRules, []) AS rule
                    MERGE (r:NSGRule {id: rule.id})
                    SET r += {
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
                    MERGE (obj)-[:HAS_RULE]->(r)
                    RETURN count(*) AS _
                }
            
            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.properties.securityRules, []) AS rule
                    MERGE (r:NSGRule {id: rule.id})
                    SET r += {
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
                    MERGE (obj)-[:HAS_RULE]->(r)
                    RETURN count(*) AS _
                }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process virtual networks
    pub async fn process_virtual_networks(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.network/virtualnetworks";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:VirtualNetwork
            SET obj += {
                addressPrefixes: row.properties.addressSpace.addressPrefixes
            }

            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.properties.subnets, []) AS subnet
                    MERGE (s:Subnet {id: subnet.id})
                    SET s += {
                        name: subnet.name,
                        type: subnet.type,
                        addressPrefix: subnet.properties.addressPrefix,
                        privateEndpointNetworkPolicies: subnet.properties.privateEndpointNetworkPolicies,
                        privateLinkServiceNetworkPolicies: subnet.properties.privateLinkServiceNetworkPolicies
                    }
                    MERGE (obj)-[:HAS_SUBNET]->(s)
                    RETURN count(*) AS _
                }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
