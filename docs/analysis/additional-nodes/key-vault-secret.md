# KeyVaultSecret

Represents Azure Key Vault secret metadata.

**Labels:** `:KeyVaultSecret`

**Properties:**

- `id` - Secret ID (primary key)
- `name` - Secret name
- `managed` - Whether Azure manages the secret
- `contentType` - Secret content type
- `kid` - Related Key Vault key ID, when present
- `enabled` - Whether the secret is enabled
- `notBefore` - Time before which the secret is invalid
- `expires` - Expiration time
- `created` - Creation time
- `updated` - Last update time
- `recoverableDays` - Number of recoverable days
- `recoveryLevel` - Recovery level
- `tags` - Secret tags as `key:value` strings

**Relationships:**

- `HAS_SECRET` ← KeyVault - Parent Key Vault
- `HAS_VERSION` → KeyVaultSecretValue - Versioned secret values
- `USED_BY` → WebCertificate - Web certificates backed by this secret

## Example Queries

Find Key Vault secrets, their versions, and any web certificates that consume them:

```cypher
MATCH (kv:KeyVault)-[:HAS_SECRET]->(s:KeyVaultSecret)
OPTIONAL MATCH (s)-[:HAS_VERSION]->(v:KeyVaultSecretValue)
OPTIONAL MATCH (s)-[:USED_BY]->(wc:WebCertificate)
RETURN kv.name, s.name, collect(DISTINCT v.id) AS versions, collect(DISTINCT wc.name) AS webCertificates
```
