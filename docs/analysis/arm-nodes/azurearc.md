# Azure Arc Resources

Azure Arc-enabled resources that extend Azure management to hybrid and multi-cloud environments.

## AzureArcSqlServer

Represents Azure Arc-enabled SQL server instances.

**Labels:** `:ArmResource:AzureArcSqlServer`

**Properties:**

- `id` - Arc SQL server resource ID (primary key)
- `azureDefenderStatus` - Azure Defender status
- `collation` - Database collation
- `currentVersion` - Current SQL Server version
- `edition` - SQL Server edition
- `instanceName` - SQL Server instance name
- `licenseType` - License type
- `patchLevel` - Patch level
- `productId` - Product ID
- `status` - Server status
- `tcpDynamicPorts` - TCP dynamic ports
- `tcpStaticPorts` - TCP static ports
- `vCore` - Virtual core count
- `version` - SQL Server version

**Relationships:**
- `HOSTS` ← ArmResource (container resource hosting this SQL server)

**Note:** This processor handles Arc-enabled SQL servers that are managed through Azure Arc, extending Azure management capabilities to on-premises and multi-cloud SQL Server instances.
