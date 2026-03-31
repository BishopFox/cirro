# USES_KEY_VAULT

Represents a resource that uses a Key Vault for key material.

## Usage

- **DiskEncryptionSet** → `USES_KEY_VAULT` → **KeyVault**

## Examples

```cypher
MATCH (des:DiskEncryptionSet)-[:USES_KEY_VAULT]->(kv:KeyVault)
RETURN des.id, kv.vaultUri, des.activeKey
```
