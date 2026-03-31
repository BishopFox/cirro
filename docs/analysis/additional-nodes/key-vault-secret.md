# KeyVaultSecret

Represents a reference to an Azure Key Vault secret. 

**Labels:** `:KeyVaultSecret`

**Properties:**

- `id` - Derived from `{keyVaultId}/secrets/{keyVaultSecretName}` (lowercased)

**Relationships:**

- `HAS_SECRET` ← KeyVault — The Key Vault that owns this secret reference
- `USED_BY` → WebCertificate — The web certificate backed by this secret

## Example Queries

Find all web certificates backed by Key Vault secrets, along with the owning Key Vault:

```cypher
MATCH (kv:KeyVault)-[:HAS_SECRET]->(kvs:KeyVaultSecret)-[:USED_BY]->(cert:WebCertificate)
RETURN kv.id AS keyVault, kvs.id AS secret, cert.id AS certificate
```
