# KeyVaultCertificate

Represents Azure Key Vault certificate metadata.

**Labels:** `:KeyVaultCertificate`

**Properties:**

- `id` - Certificate ID (primary key)
- `name` - Certificate name
- `subject` - Certificate subject
- `x509Thumbprint` - Thumbprint value
- `x509ThumbprintHex` - Hex-encoded thumbprint
- `enabled` - Whether the certificate is enabled
- `notBefore` - Time before which the certificate is invalid
- `expires` - Expiration time
- `created` - Creation time
- `updated` - Last update time
- `recoverableDays` - Number of recoverable days
- `recoveryLevel` - Recovery level
- `tags` - Certificate tags as `key:value` strings

**Relationships:**

- `HAS_CERTIFICATE` ← KeyVault - Parent Key Vault
- `HAS_VERSION` → KeyVaultCertificateValue - Versioned certificate value objects
- `MATCHES_CERT` → Certificate - Matched application or service principal certificates by thumbprint

## Example Queries

Find Key Vault certificates, their versions, and any matched Entra certificates:

```cypher
MATCH (kv:KeyVault)-[:HAS_CERTIFICATE]->(c:KeyVaultCertificate)
OPTIONAL MATCH (c)-[:HAS_VERSION]->(v:KeyVaultCertificateValue)
OPTIONAL MATCH (c)-[:MATCHES_CERT]->(cert:Certificate)
RETURN kv.name, c.name, collect(DISTINCT v.id) AS versions, collect(DISTINCT cert.thumbprint) AS matchedThumbprints
```
