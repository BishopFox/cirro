# DocumentDB

Represents Azure Cosmos DB (DocumentDB) Database Accounts.

**Labels:** `:ArmResource:DocumentDB`

**Properties:**

- `id` - Resource ID (primary key)
- `enabledApiTypes` - Enabled API types
- `documentEndpoint` - Document endpoint
- `instanceId` - Instance ID
- `networkAclBypass` - Network ACL bypass setting
- `publicNetworkAccess` - Public network access setting
- `sqlEndpoint` - SQL endpoint
- `readLocations` - List of read location endpoints
- `writeLocations` - List of write location endpoints
- `ipRules` - List of IP rules
- `isVirtualNetworkFilterEnabled` - Whether virtual network filter is enabled

## Examples

```cypher
// Find Cosmos DB accounts with public access
MATCH (db:DocumentDB)
WHERE db.publicNetworkAccess = 'Enabled'
RETURN db.name, db.enabledApiTypes, db.documentEndpoint
```
