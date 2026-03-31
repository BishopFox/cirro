# Remove Duplicate Nodes

Merges duplicate nodes across the entire graph that share the same `id`.

**Priority:** 0

## Details

This is a general-purpose deduplication step that finds any nodes with matching IDs and merges them. Properties are combined and relationships are merged. IDs are normalized to lowercase.

## Cypher

```cypher
MATCH (n)
WHERE n.id IS NOT NULL
WITH toLower(n.id) AS lid, collect(n) AS ns
WHERE size(ns) > 1
UNWIND tail(ns) AS d
CALL apoc.refactor.mergeNodes([head(ns), d], {properties:'combine', mergeRels:true})
YIELD node
SET head(ns).id = lid
```
