use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process availability sets
    pub async fn process_availability_sets(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.compute/availabilitysets";
        let properties = vec!["/id", "/sku", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:AvailabilitySet
            SET obj += {
                sku: row.sku.name,
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process disks
    pub async fn process_disks(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.compute/disks";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:Disk
            SET obj += {
                diskSizeGB: row.properties.diskSizeGB,
                diskState: row.properties.diskState,
                osType: row.properties.osType,
                networkAccessPolicy: row.properties.networkAccessPolicy,
                publicNetworkAccess: row.properties.publicNetworkAccess,
                timeCreated: row.properties.timeCreated,
                uniqueId: row.properties.uniqueId
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process restore point collections
    pub async fn process_restore_point_collections(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.compute/restorepointcollections";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:RestorePointCollection

            MERGE (vm:ArmResource {id: row.properties.source.id})
            MERGE (vm)-[:HAS_RESTOREPOINT]->(obj)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process snapshots
    pub async fn process_snapshots(&self) -> Result<(), CirroGraphError> {
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
    pub async fn process_ssh_public_keys(&self) -> Result<(), CirroGraphError> {
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
    pub async fn process_virtual_machines(&self) -> Result<(), CirroGraphError> {
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

            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.properties.networkProfile.networkInterfaces, []) AS nic
                    MERGE (n:ArmResource {id: nic.id})
                    MERGE (obj)-[:HAS_NIC]->(n)
                    RETURN count(*) AS _
                }
            
            WITH obj, row
                CALL {
                    WITH obj, row
                    UNWIND coalesce(row.resources, []) AS resource
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
                    RETURN count(*) AS _
                }
            RETURN count(*) AS _
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process virtual machine extensions
    pub async fn process_virtual_machine_extensions(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.compute/virtualmachines/extensions";
        let properties = vec!["/id", "/properties"];

        // VMExtensions aren't a primary resource type so remove resource group relationship
        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:VMExtension {id: row.id})

            SET obj += {
                autoUpgradeMinorVersion: row.properties.autoUpgradeMinorVersion,
                provisioningState: row.properties.provisioningState,
                publisher: row.properties.publisher,
                triggerForceUpgrade: row.properties.triggerForceUpgrade,
                type: row.properties.type,
                vmType: row.properties.settings.vmType,
                objectStr: row.properties.settings.objectStr
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process virtual machine applications
    pub async fn process_vm_applications(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.compute/virtualmachines/vmapplications";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:VMApplication
            SET obj += {
                enableAutomaticUpgrade: row.properties.enableAutomaticUpgrade,
                manuallyManaged: row.properties.manuallyManaged,
                treatFailureAsDeploymentFailure: row.properties.treatFailureAsDeploymentFailure
            }
            MERGE (o:GalleryAppVersion {id: row.properties.packageReferenceId})
            MERGE (obj)-[:REFERENCES_PACKAGE]->(o)

            WITH obj, row, split(toLower(obj.id), '/vmapplications/')[0] AS vmId
                MERGE (vm:VirtualMachine {id: vmId})
                MERGE (vm)-[:HAS_VMAPP]->(obj)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process galleries
    pub async fn process_galleries(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.compute/galleries";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:Gallery
            SET obj += {
                uniqueName: row.properties.identifier.uniqueName
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process gallery applications
    pub async fn process_gallery_applications(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.compute/galleries/applications";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:GalleryApp
            SET obj += {
                supportedOSType: row.properties.supportedOSType
            }

            WITH obj, row
            WITH obj, row, split(row.id, '/applications/')[0] AS galleryId
            MERGE (g:Gallery {id: galleryId})
            MERGE (g)-[:HAS_APPLICATION]->(obj)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process gallery application versions
    pub async fn process_gallery_application_versions(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.compute/galleries/applications/versions";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:GalleryAppVersion
            SET obj += {
                excludeFromLatest: row.properties.publishingProfile.excludeFromLatest,
                installAction: row.properties.publishingProfile.manageActions.install,
                removeAction: row.properties.publishingProfile.manageActions.remove,
                publishedDate: row.properties.publishingProfile.publishedDate,
                packageFileName: row.properties.publishingProfile.settings.packageFileName,
                scriptBehaviorAfterReboot: row.properties.publishingProfile.settings.scriptBehaviorAfterReboot,
                source: row.properties.publishingProfile.settings.source.mediaLink
            }

            WITH obj, row
            WITH obj, row, split(row.id, '/versions/')[0] AS galleryAppId
            MERGE (g:GalleryApp {id: galleryAppId})
            MERGE (g)-[:HAS_VERSION]->(obj)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
