# KeyVaultSecretValue

Represents versioned Azure Key Vault secret values.

**Labels:** `:KeyVaultSecretValue`

**Properties:**

- `id` - Secret value version ID (primary key)
- `name` - Secret name
- `value` - Secret value collected from Key Vault
- `managed` - Whether Azure manages the secret
- `contentType` - Secret content type
- `kid` - Related Key Vault key ID, when present
- `enabled` - Whether the secret value is enabled
- `notBefore` - Time before which the secret value is invalid
- `expires` - Expiration time
- `created` - Creation time
- `updated` - Last update time
- `recoverableDays` - Number of recoverable days
- `recoveryLevel` - Recovery level
- `tags` - Secret tags as `key:value` strings

**Relationships:**

- `HAS_VERSION` ← KeyVaultSecret - Parent secret metadata node
- `MATCHES_SECRET` → ClientSecret - Matched client secrets discovered during post-processing

## Example Queries

Find versioned Key Vault secret values and any matched client secrets:

```cypher
MATCH (s:KeyVaultSecret)-[:HAS_VERSION]->(v:KeyVaultSecretValue)
OPTIONAL MATCH (v)-[:MATCHES_SECRET]->(c:ClientSecret)
RETURN s.name, v.id, c.hint
```
