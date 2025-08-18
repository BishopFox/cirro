# ArcSqlServer

Azure Arc-enabled SQL Server instances that extend Azure management to on-premises SQL Server instances.

**Labels:** `:ArmResource:ArcSqlServer`

## Properties

- `id` - Resource ID (inherited from ArmResource)
- `name` - Resource name (inherited from ArmResource)
- `type` - Resource type (inherited from ArmResource)
- `location` - Azure region (inherited from ArmResource)
- `tags` - Resource tags (inherited from ArmResource)
- `azureDefenderStatus` - Azure Defender protection status
- `collation` - SQL Server collation setting
- `currentVersion` - Current SQL Server version
- `edition` - SQL Server edition
- `instanceName` - SQL Server instance name
- `licenseType` - SQL Server license type
- `patchLevel` - Current patch level
- `productId` - SQL Server product identifier
- `status` - Current status of the SQL Server instance
- `tcpDynamicPorts` - TCP dynamic port configuration
- `tcpStaticPorts` - TCP static port configuration
- `vCore` - Virtual core count
- `version` - SQL Server version

## Relationships

- Connected to Azure Arc management services
