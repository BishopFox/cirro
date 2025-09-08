use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process dns zones
    pub async fn process_dns_zones(&self) -> Result<(), CirroGraphError> {
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
    pub async fn process_public_ip_addresses(&self) -> Result<(), CirroGraphError> {
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
                publicIPAllocationMethod: row.properties.publicIPAllocationMethod,
                domainNameLabel: row.properties.dnsSettings.domainNameLabel,
                fqdn: row.properties.dnsSettings.fqdn
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process network interfaces
    pub async fn process_network_interfaces(&self) -> Result<(), CirroGraphError> {
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

            WITH obj, row WHERE row.properties.virtualMachine.id IS NOT NULL
                MERGE (vm:ArmResource {id: row.properties.virtualMachine.id})
                SET vm:VirtualMachine
                MERGE (vm)-[:HAS_NIC]->(obj) 

            WITH obj, row WHERE row.properties.networkSecurityGroup IS NOT NULL
                MERGE (nsg:ArmResource {id: row.properties.networkSecurityGroup.id})
                SET nsg:NSG
                MERGE (obj)-[:HAS_NSG]->(nsg)

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
            RETURN count(*) AS _
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process network security groups
    pub async fn process_network_security_groups(&self) -> Result<(), CirroGraphError> {
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
            RETURN count(*) AS _
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process virtual networks
    pub async fn process_virtual_networks(&self) -> Result<(), CirroGraphError> {
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
                        addressPrefixes: subnet.properties.addressPrefixes,
                        privateEndpointNetworkPolicies: subnet.properties.privateEndpointNetworkPolicies,
                        privateLinkServiceNetworkPolicies: subnet.properties.privateLinkServiceNetworkPolicies
                    }
                    MERGE (obj)-[:HAS_SUBNET]->(s)

                    WITH obj, row, s, subnet WHERE subnet.properties.routeTable.id IS NOT NULL
                    MERGE (r:RouteTable {id: subnet.properties.routeTable.id})
                    MERGE (s)-[:HAS_ROUTE_TABLE]->(r)
                    RETURN count(*) AS _
                }
            
            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.properties.virtualNetworkPeerings, []) AS peer
                    MERGE (p:NetworkPeering {id: peer.id})
                    SET p += {
                        name: peer.name,
                        type: peer.type,
                        allowForwardedTraffic: peer.properties.allowForwardedTraffic,
                        allowGatewayTransit: peer.properties.allowGatewayTransit,
                        allowVirtualNetworkAccess: peer.properties.allowVirtualNetworkAccess,
                        doNotVerifyRemoteGateway: peer.properties.doNotVerifyRemoteGateway,
                        peerCompleteVnets: peer.properties.peerCompleteVnets,
                        peeringState: peer.properties.peeringState,
                        peeringSyncLevel: peer.properties.peeringSyncLevel,
                        remoteAddressPrefixes: peer.properties.remoteAddressSpace.addressPrefixes,
                        useRemoteGateways: peer.properties.useRemoteGateways
                    }
                    MERGE (obj)-[:HAS_PEERING]->(p)

                    WITH obj, row, peer, p
                    CALL {
                        WITH obj, row, peer, p
                        UNWIND coalesce(peer.properties.remoteGateways, []) AS gateway
                        MERGE (g:NetworkGateway {id: gateway.id})
                        MERGE (p)-[:HAS_GATEWAY]->(g)

                        MERGE (v:VirtualNetwork {id: peer.properties.remoteVirtualNetwork.id})
                        MERGE (p)-[:PEER_TO]->(v)
                        RETURN count(*) AS _
                    }
                    RETURN count(*) AS _
                }
            RETURN count(*) AS _
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process route tables
    pub async fn process_route_tables(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.network/routetables";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:RouteTable
            SET obj += {
                disableBgpRoutePropagation: row.properties.disableBgpRoutePropagation
            }

            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.properties.routes, []) AS route
                    MERGE (r:NetworkRoute {id: route.id})
                    SET r += {
                        name: route.name,
                        type: route.type,
                        addressPrefix: route.properties.addressPrefix,
                        nextHopType: route.properties.nextHopType,
                        nextHopIpAddress: route.properties.nextHopIpAddress,
                        hasBgpOverride: route.properties.hasBgpOverride
                    }
                    MERGE (obj)-[:HAS_ROUTE]->(r)
                    RETURN count(*) AS _
                }

            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.properties.subnets, []) AS subnet
                    MERGE (s:Subnet {id: subnet.id})
                    MERGE (s)-[:HAS_ROUTE_TABLE]->(obj)
                    RETURN count(*) AS _
                }
            RETURN count(*) AS _
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process bastion hosts
    pub async fn process_bastion_hosts(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.network/bastionhosts";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:BastionHost
            SET obj += {
                disableCopyPaste: row.properties.disableCopyPaste,
                dnsName: row.properties.dnsName,
                enableIpConnect: row.properties.enableIpConnect,
                enableKerberos: row.properties.enableKerberos,
                enablePrivateOnlyBastion: row.properties.enablePrivateOnlyBastion,
                enableSessionRecording: row.properties.enableSessionRecording,
                enableShareableLink: row.properties.enableShareableLink,
                enableTunneling: row.properties.enableTunneling,
                scaleUnits: row.properties.scaleUnits
            }

            WITH obj, row
            CALL {
                WITH obj, row
                UNWIND coalesce(row.properties.ipConfigurations, []) AS ipconfig
                MERGE (b:BastionIPConfig {id: ipconfig.id})
                SET b += {
                    privateIPAllocationMethod: ipconfig.properties.privateIPAllocationMethod
                }
                MERGE (obj)-[:HAS_CONFIG]->(b)

                MERGE (i:PublicIPAddress {id: ipconfig.properties.publicIPAddress.id})
                MERGE (b)-[:HAS_IP]->(i)
                
                MERGE (sub:Subnet {id: ipconfig.properties.subnet.id})
                MERGE (sub)-[:CONTAINS]->(b)

                RETURN count(*) AS _
            }
            RETURN count(*) AS _
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process private endpoints
    pub async fn process_private_endpoints(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.network/privateendpoints";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:PrivateEndpoint
            SET obj += {
                customNetworkInterfaceName: row.properties.customNetworkInterfaceName,
                ipVersionType: row.properties.ipVersionType
            }

            WITH obj, row
            CALL {
                WITH obj, row
                UNWIND coalesce(row.properties.customDnsConfigs, []) AS config
                MERGE (d:CustomDnsConfig {fqdn: config.fqdn})
                SET d += {
                    ipAddresses: config.properties.ipAddresses
                }
                MERGE (obj)-[:HAS_DNS_CONFIG]->(d)
                RETURN count(*) AS _
            }

            WITH obj, row
            CALL {
                WITH obj, row
                UNWIND coalesce(row.properties.networkInterfaces, []) AS interface
                MERGE (n:NetworkInterface {id: interface.id})
                RETURN count(*) AS _
            }

            WITH obj, row
            CALL {
                WITH obj, row 
                WITH obj, row WHERE row.properties.subnet.id IS NOT NULL
                MERGE (s:Subnet {id: row.properties.subnet.id})
                MERGE (s)-[:CONTAINS]->(obj)
                RETURN count(*) AS _
            }

            WITH obj, row
            CALL {
                WITH obj, row
                UNWIND coalesce(row.properties.privateLinkServiceConnections, []) AS conn
                MERGE (p:PrivateLinkServiceConnection {id: conn.id})
                SET p += {
                    name: conn.name,
                    type: conn.type,
                    groupIds: conn.properties.groupIds
                }
                MERGE (r:ArmResource {id: conn.properties.privateLinkServiceId})
                MERGE (r)-[:HAS_PRIVATE_ENDPOINT]->(obj)
                RETURN count(*) AS _
            }
            RETURN count(*) AS _
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process private dns zones
    pub async fn process_private_dns_zones(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.network/privateDnsZones";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:PrivateDnsZone

            SET obj += {
                internalId: row.properties.internalId,
                numberOfRecordSets: row.properties.numberOfRecordSets,
                numberOfVirtualNetworkLinks: row.properties.numberOfVirtualNetworkLinks,
                numberOfVirtualNetworkLinksWithRegistration: row.properties.numberOfVirtualNetworkLinksWithRegistration
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
