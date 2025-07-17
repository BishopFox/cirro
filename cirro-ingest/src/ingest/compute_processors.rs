use crate::errors::CirroIngestError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process availability sets
    pub async fn process_availability_sets(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.compute/availabilitysets";
        let properties = vec!["/id", "/sku", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:AvailabilitySet
            SET obj += {
                sku: row.sku.name,
            }

            WITH obj, row WHERE row.managedBy IS NOT NULL
                MERGE (h:ArmResource {id: row.managedBy})
                MERGE (h)-[:MANAGES]->(obj)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process disks
    pub async fn process_disks(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.compute/disks";
        let properties = vec!["/id", "/managedBy", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:Disk
            SET obj += {
                managedBy: row.managedBy,
                diskSizeGB: row.properties.diskSizeGB,
                diskState: row.properties.diskState,
                osType: row.properties.osType,
                networkAccessPolicy: row.properties.networkAccessPolicy,
                publicNetworkAccess: row.properties.publicNetworkAccess
            }

            WITH obj, row WHERE row.managedBy IS NOT NULL
                MERGE (h:ArmResource {id: row.managedBy})
                MERGE (h)-[:HAS_DISK]->(obj)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process restore point collections
    pub async fn process_restore_point_collections(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.compute/restorepointcollections";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:RestorePointCollection

            MERGE (vm:ArmResource {id: row.properties.source.id})
            MERGE (h)-[:HAS_RESTOREPOINT]->(obj)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process snapshots
    pub async fn process_snapshots(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.compute/snapshots";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:Snapshot
            SET obj += {
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

            MERGE (h:ArmResource {id: row.properties.creationData.sourceResourceId})
            MERGE (h)-[:HAS_SNAPSHOT]->(obj)
        "#;
        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process SSH public keys
    pub async fn process_ssh_public_keys(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.compute/sshpublickeys";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:SSHPublicKey
            SET obj += {
                publicKey: row.properties.publicKey
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process virtual machines
    pub async fn process_virtual_machines(&self) -> Result<(), CirroIngestError> {
        let resource_type = "microsoft.compute/virtualmachines";
        let properties = vec!["/id", "/resources", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:VirtualMachine
            SET obj += {
                vmSize: row.properties.hardwareProfile.vmSize,
                adminUsername: row.properties.osProfile.adminUsername,
                allowExtensionOperations: row.properties.osProfile.allowExtensionOperations,
                computerName: row.properties.osProfile.computerName,
                vmId: row.properties.vmId,
                os: row.properties.storageProfile.osDisk.osType,
                image: row.properties.storageProfile.imageReference.offer,
                imageVersion: row.properties.storageProfile.imageReference.exactVersion
            }

            WITH obj, row WHERE row.properties.networkProfile.networkInterfaces IS NOT NULL
                UNWIND row.properties.networkProfile.networkInterfaces AS nic
                MERGE (n:ArmResource {id: nic.id})
                MERGE (obj)-[:HAS_NIC]->(n)
            
            WITH obj, row WHERE row.resources IS NOT NULL
                UNWIND row.resources AS resource
                WITH obj, resource WHERE toLower(resource.type) = "microsoft.compute/virtualmachines/extensions"
                    MERGE (e:VMExtension {id: resource.id})
                    SET e += {
                        id: resource.id,
                        name: resource.name,
                        type: resource.type,
                        location: resource.location,
                        provisioningState: resource.properties.provisioningState
                    }
                    MERGE (obj)-[:HAS_EXTENSION]->(e)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
