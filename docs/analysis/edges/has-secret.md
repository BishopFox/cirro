# HAS_SECRET

Represents Key Vault ownership of a Key Vault secret node.

**Direction:** `(keyVault)-[:HAS_SECRET]->(keyVaultSecret)`

**Description:** Created during Web Certificate ingestion when Key Vault-backed certificate metadata is present.

**Properties:** None

## Query Examples

```cypher
// Find key vault secrets associated with web certificates
MATCH (kv:KeyVault)-[:HAS_SECRET]->(s:KeyVaultSecret)-[:USED_BY]->(wc:WebCertificate)
RETURN kv.name, s.id, wc.name
```
