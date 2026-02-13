# USES_STORAGE

Represents dependency on a storage account.

**Direction:** `(resource)-[:USES_STORAGE]->(storageAccount)`

**Description:** Connects resources (for example `PurviewAccount`) to storage accounts used for ingestion or service operations.

**Properties:** None

## Query Examples

```cypher
// Find purview accounts and backing storage accounts
MATCH (p:PurviewAccount)-[:USES_STORAGE]->(sa:StorageAccount)
RETURN p.name, sa.name, sa.id
```
