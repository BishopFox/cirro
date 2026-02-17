# DiskEncryptionSet

Represents Azure Disk Encryption Sets (DES).

**Labels:** `:ArmResource:DiskEncryptionSet`

**Properties:**

- `id` - Resource ID (primary key)
- `activeKey` - Active key URL
- `encryptionType` - Encryption type
- `lastKeyRotationTimestamp` - Last key rotation time
- `rotationToLatestKeyVersionEnabled` - Automatic rotation enabled

## Relationships

### Outgoing
- **DiskEncryptionSet** → `USES_KEY_VAULT` → **KeyVault** - Key Vault that holds the active key

## Examples

```cypher
// DES with their Key Vaults
MATCH (des:DiskEncryptionSet)-[:USES_KEY_VAULT]->(kv:KeyVault)
RETURN des.id, kv.vaultUri, des.activeKey
```
