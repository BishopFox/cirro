# USED_BY

Represents reverse usage from a secret to a consuming resource.

**Direction:** `(resource)-[:USED_BY]->(consumer)`

**Description:** Currently used for Key Vault secret to Web Certificate linkage.

**Properties:** None

## Query Examples

```cypher
// Find web certificates by backing secret
MATCH (s:KeyVaultSecret)-[:USED_BY]->(wc:WebCertificate)
RETURN s.id, wc.name, wc.thumbprint
```
