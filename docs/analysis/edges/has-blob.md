# HAS_BLOB

Represents blob storage relationships.

**Direction:** `(container)-[:HAS_BLOB]->(blob)`

**Description:** Indicates that a container contains a specific blob.

**Properties:** Blob properties and metadata

## Query Examples

```cypher
// Find all blobs in a container
MATCH path=(container)-[:HAS_BLOB]->(blob)
RETURN path

// Find large blobs
MATCH (container)-[rel:HAS_BLOB]->(blob)
WHERE rel.size > 1000000  // Blobs larger than 1MB
RETURN container, blob, rel.size

// Find blob storage hierarchy
MATCH path=(sa:StorageAccount)-[:HAS_CONTAINER]->(container)-[:HAS_BLOB]->(blob)
RETURN path
```
