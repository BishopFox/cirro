# HybridPrivateLinkScope

Represents an Azure Arc private link scope resource used to connect hybrid machines over private endpoints.

**Labels:** `:ArmResource:HybridPrivateLinkScope`

**Properties:**

- `id` - Private link scope resource ID (primary key)
- `privateLinkScopeId` - Unique private link scope identifier
- `provisioningState` - Provisioning state
- `publicNetworkAccess` - Public network access setting

## Examples

```cypher
// Find private link scopes with public access enabled
MATCH (pls:HybridPrivateLinkScope)
WHERE pls.publicNetworkAccess = "Enabled"
RETURN pls.id, pls.privateLinkScopeId
```
