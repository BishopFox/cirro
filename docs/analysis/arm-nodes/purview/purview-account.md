# PurviewAccount

Represents Microsoft Purview Account resources.

**Labels:** `:ArmResource:PurviewAccount`

**Properties:**

- `id` - Resource ID (primary key)
- `awsExternalId` - AWS external ID for cloud connectors
- `createdAt` - Creation timestamp
- `createdBy` - Created by
- `defaultDomain` - Default domain
- `endpoints` - List of endpoint URLs
- `friendlyName` - Friendly name
- `managedResourcesPublicNetworkAccess` - Managed resources public network access
- `publicNetworkAccess` - Public network access setting

## Relationships

### Outgoing

- **PurviewAccount** → `USES_STORAGE` → **StorageAccount** - Ingestion storage account used by Purview

## Examples

```cypher
// Find Purview accounts and their storage
MATCH (p:PurviewAccount)-[:USES_STORAGE]->(sa:StorageAccount)
RETURN p.name, p.friendlyName, sa.name
```
