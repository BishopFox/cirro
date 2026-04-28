# ManagedHSM

Represents Azure Key Vault Managed HSM resources.

**Labels:** `:ManagedHSM`

**Properties:**

- `id` - Managed HSM resource ID (primary key)
- `enablePurgeProtection` - Whether purge protection is enabled
- `enableSoftDelete` - Whether soft delete is enabled
- `hsmUri` - Managed HSM URI
- `publicNetworkAccess` - Public network access setting
- `activationStatus` - Security domain activation status
- `activationStatusMessage` - Security domain activation status message
- `softDeleteRetentionInDays` - Soft delete retention period
- `statusMessage` - Service status message

## Relationships

### Incoming

- **GraphObject** → `IS_ADMIN` → **ManagedHSM** - Directory object configured as initial HSM administrator

## Examples

```cypher
// Find Managed HSM resources and their administrators
MATCH (go:GraphObject)-[:IS_ADMIN]->(hsm:ManagedHSM)
RETURN hsm.id, hsm.hsmUri, go.id
```