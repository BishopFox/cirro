# SQL Resources

Azure SQL-related resources including SQL servers and databases.

## SqlServer

Represents Azure SQL servers.

**Labels:** `:ArmResource:SqlServer`

**Properties:**

- `id` - SQL server resource ID (primary key)
- `administratorLogin` - Administrator login name
- `fullyQualifiedDomainName` - Fully qualified domain name
- `publicNetworkAccess` - Public network access setting
- `restrictOutboundNetworkAccess` - Restrict outbound network access
- `state` - Server state
- `version` - SQL Server version

**Relationships:**
- `HAS_DB` → SqlDatabase

## SqlDatabase

Represents Azure SQL databases.

**Labels:** `:ArmResource:SqlDatabase`

**Properties:**

- `id` - SQL database resource ID (primary key)
- `collation` - Database collation
- `creationDate` - Database creation date
- `currentServiceObjectiveName` - Current service objective name
- `databaseId` - Database ID
- `isInfraEncryptionEnabled` - Infrastructure encryption enabled
- `maxSizeBytes` - Maximum size in bytes
- `status` - Database status

**Relationships:**
- `HAS_DB` ← SqlServer
