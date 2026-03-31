# WebCertificate

Represents Azure Web Certificates.

**Labels:** `:ArmResource:WebCertificate`

**Properties:**

- `id` - Resource ID (primary key)
- `cerBlob` - Certificate blob
- `expirationDate` - Expiration date
- `friendlyName` - Friendly name
- `hostNames` - Host names
- `issueDate` - Issue date
- `issuer` - Certificate issuer
- `keyVaultId` - Key Vault resource ID
- `keyVaultSecretName` - Key Vault secret name
- `keyVaultSecretStatus` - Key Vault secret status
- `password` - Password
- `pfxBlob` - PFX blob
- `publicKeyHash` - Public key hash
- `subjectName` - Subject name
- `thumbprint` - Certificate thumbprint
- `valid` - Whether the certificate is valid

## Relationships

### Outgoing

- **KeyVaultSecret** → `USED_BY` → **WebCertificate** - Key Vault secret used by this certificate
- **KeyVault** → `HAS_SECRET` → **KeyVaultSecret** - Key Vault containing the secret (created during ingestion)

## Examples

```cypher
// Find web certificates stored in Key Vault
MATCH (kv:KeyVault)-[:HAS_SECRET]->(kvs:KeyVaultSecret)-[:USED_BY]->(wc:WebCertificate)
RETURN wc.subjectName, wc.expirationDate, kv.name
```
