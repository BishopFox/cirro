# KeyVaultCertificateValue

Represents versioned Azure Key Vault certificate value objects, including certificate material and issuance policy metadata.

**Labels:** `:KeyVaultCertificateValue`

**Properties:**

- `id` - Certificate value version ID (primary key)
- `name` - Certificate name
- `cer` - Encoded certificate content
- `contentType` - Certificate content type
- `kid` - Related Key Vault key ID
- `sid` - Related Key Vault secret ID
- `x509Thumbprint` - Thumbprint value
- `x509ThumbprintHex` - Hex-encoded thumbprint
- `enabled` - Whether the certificate value is enabled
- `notBefore` - Time before which the certificate value is invalid
- `expires` - Expiration time
- `created` - Creation time
- `updated` - Last update time
- `recoverableDays` - Number of recoverable days
- `recoveryLevel` - Recovery level
- `issuerTransparency` - Certificate transparency setting from the issuer policy
- `issuerType` - Issuer certificate type
- `issuerName` - Issuer name
- `exportable` - Whether the private key is exportable
- `keyType` - Key algorithm or key type
- `secretContentType` - Content type used for the backing secret
- `keyUsage` - X.509 key usage values
- `x509SubjectName` - X.509 subject name from the policy
- `x509DnsNames` - DNS subject alternative names
- `x509Emails` - Email subject alternative names
- `x509Upns` - UPN subject alternative names
- `tags` - Certificate tags as `key:value` strings

**Relationships:**

- `HAS_VERSION` ← KeyVaultCertificate - Parent certificate metadata node

## Example Queries

Find versioned Key Vault certificate values and their issuance details:

```cypher
MATCH (c:KeyVaultCertificate)-[:HAS_VERSION]->(v:KeyVaultCertificateValue)
RETURN c.name, v.x509SubjectName, v.secretContentType, v.expires
```
