# HAS_CONTAINER

Represents storage container relationships.

**Direction:** `(storageAccount)-[:HAS_CONTAINER]->(container)`

**Description:** Indicates that a storage account contains a specific container.

**Properties:** Container access level and properties

## Query Examples

```cypher
// Find all containers in a storage account
MATCH path=(sa:StorageAccount)-[:HAS_CONTAINER]->(container)
RETURN path

// Find public containers
MATCH path=(sa:StorageAccount)-[rel:HAS_CONTAINER]->(container)
WHERE rel.accessLevel = 'public'
RETURN path

// Find storage accounts with many containers
MATCH (sa:StorageAccount)-[:HAS_CONTAINER]->(container)
WITH sa, COUNT(container) as containerCount
WHERE containerCount > 10
RETURN sa, containerCount
```
