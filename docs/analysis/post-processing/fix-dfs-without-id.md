# Fix Storage Account DFS Without ID

Sets missing IDs on StorageAccountDFS nodes based on their parent StorageAccount.

**Priority:** 1001

## Details

Some DFS filesystem nodes may be created without an `id` property. This step constructs the ID by combining the parent StorageAccount's ID with the DFS filesystem name.

## Cypher

```cypher
MATCH (sa:StorageAccount)-[:HAS_DFS_FILESYSTEM]->(dfs:StorageAccountDFS)
WHERE dfs.id IS NULL
SET dfs.id = toLower(sa.id) + '/filesystem/' + toLower(dfs.name)
```
