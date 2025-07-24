# HAS_KEY

Represents storage account key relationships (from enrichment).

**Direction:** `(storageAccount)-[:HAS_KEY]->(key)`

**Description:** Indicates that a storage account has access keys (discovered during enrichment).

**Properties:**

- Key name
- Key permissions
- Key type

## Query Examples

```cypher
// Find all storage account keys
MATCH path=(sa:StorageAccount)-[:HAS_KEY]->(key)
RETURN path

// Find key properties and permissions
MATCH (sa:StorageAccount)-[rel:HAS_KEY]->(key)
RETURN sa, key, rel.keyName, rel.permissions, rel.keyType
```
