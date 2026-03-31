# Normalize IDs

Lowercases all node IDs across the graph.

**Priority:** 0

## Details

Finds all nodes where the `id` property is not already lowercase and normalizes it. This ensures consistent ID matching across the graph.

## Cypher

```cypher
MATCH (n)
WHERE n.id IS NOT NULL AND n.id <> toLower(n.id)
SET n.id = toLower(n.id)
```
