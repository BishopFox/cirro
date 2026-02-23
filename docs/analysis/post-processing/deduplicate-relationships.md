# Remove Duplicate Relationships

Merges duplicate relationships between the same pair of nodes.

**Priority:** 2

## Details

Finds all cases where multiple relationships of the same type exist between the same two nodes, and merges them using `apoc.refactor.mergeRelationships`. Properties are overwritten from the latest relationship.

## Cypher

```cypher
MATCH (a)-[r]->(b)
WITH a, b, type(r) AS relType, collect(r) AS rels
WHERE size(rels) > 1
CALL apoc.refactor.mergeRelationships(rels, {properties: 'overwrite'})
YIELD rel
RETURN 1
```
