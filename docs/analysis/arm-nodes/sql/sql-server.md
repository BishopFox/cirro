# SqlServer

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
