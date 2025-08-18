use crate::errors::CirroGraphError;
use crate::ingest::ingestor::CirroIngestor;

impl CirroIngestor {
    /// Process serverfarms
    pub async fn process_server_farms(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.web/serverfarms";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:ServerFarm
            SET obj += {
                adminRuntimeSiteName: row.properties.adminRuntimeSiteName,
                adminSiteName: row.properties.adminSiteName,
                createdTime: row.properties.createdTime,
                kind: row.properties.kind,
                mdmId: row.properties.mdmId,
                numberOfSites: row.properties.numberOfSites,
                planName: row.properties.planName,
                status: row.properties.status
            }
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }

    /// Process sites
    pub async fn process_sites(&self) -> Result<(), CirroGraphError> {
        let resource_type = "microsoft.web/sites";
        let properties = vec!["/id", "/properties"];

        let node_insert_query = r#"
            UNWIND $batch AS row
            MERGE (obj:ArmResource {id: row.id})
            SET obj:WebSite
            SET obj += {
                adminEnabled: row.properties.adminEnabled,
                clientCertEnabled: row.properties.clientCertEnabled,
                clientCertMode: row.properties.clientCertMode,
                defaultHostName: row.properties.defaultHostName,
                enabled: row.properties.enabled,
                enabledHostNames: row.properties.enabledHostNames,
                ftpUsername: row.properties.ftpUsername,
                ftpsHostName: row.properties.ftpsHostName,
                httpsOnly: row.properties.httpsOnly,
                inFlightFeatures: row.properties.inFlightFeatures,
                inboundIpAddress: row.properties.inboundIpAddress,
                inboundIpv6Address: row.properties.inboundIpv6Address,
                ipMode: row.properties.ipMode,
                keyVaultReferenceIdentity: row.properties.keyVaultReferenceIdentity,
                kind: row.properties.kind,
                lastModifiedTimeUtc: row.properties.lastModifiedTimeUtc,
                publicNetworkAccess: row.properties.publicNetworkAccess,
                linuxFxVersion: row.properties.siteConfig.linuxFxVersion
            }

            MERGE (sf:ServerFarm {id: row.properties.serverFarmId})
            MERGE (sf)-[:HOSTS_SITE]->(obj)
        "#;

        self.process_specific_arm_resource(resource_type, node_insert_query, properties)
            .await?;
        Ok(())
    }
}
