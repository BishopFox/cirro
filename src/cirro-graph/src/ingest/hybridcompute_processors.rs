use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process hybrid machines
    pub async fn process_hybrid_machines(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.hybridcompute/machines";
        let properties = vec!["/id", "/resources", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:HybridMachine
            SET obj += {
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
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.properties.networkProfile.networkInterfaces, []) AS nic
                    WITH obj, row, nic
                        UNWIND coalesce(nic.ipAddresses, []) AS ipAddress
                        MERGE (ip:HybridIPAddress {address: ipAddress.address})
                        SET ip += {
                            ipAddressVersion: ipAddress.ipAddressVersion,
                            subnet: ipAddress.subnet.addressPrefix
                        }
                        MERGE (obj)-[:HAS_IP]->(ip)
                    RETURN count(*) AS _
                }
            
            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.resources, []) AS resource
                    WITH obj, resource WHERE toLower(resource.type) = "microsoft.hybridcompute/machines/extensions"
                        MERGE (e:HybridExtension {id: resource.id})
                        SET e += {
                            name: resource.name,
                            type: resource.type,
                            location: resource.location,
                            typeHandlerVersion: resource.properties.typeHandlerVersion,
                            autoUpgradeMinorVersion: resource.properties.autoUpgradeMinorVersion,
                            enableAutomaticUpgrade: resource.properties.enableAutomaticUpgrade,
                            statusMessage: resource.properties.instanceView.status.message,
                            provisioningState: resource.properties.provisioningState
                        }
                        MERGE (obj)-[:HAS_EXTENSION]->(e)
                    RETURN count(*) AS _
                }
            RETURN count(*) AS _
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
