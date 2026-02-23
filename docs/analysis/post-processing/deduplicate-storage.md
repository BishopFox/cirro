# Deduplicate Storage Accounts by Name

Merges duplicate StorageAccount nodes that share the same `name`.

**Priority:** 1000

## Details

When multiple StorageAccount nodes exist with the same name, this step merges them into a single node, combining properties and relationships. The `name` is normalized to lowercase.

## Cypher

```cypher
MATCH (n:StorageAccount)
WHERE n.name IS NOT NULL
WITH toLower(n.name) AS lid, collect(n) AS ns
WHERE size(ns) > 1
UNWIND tail(ns) AS d
CALL apoc.refactor.mergeNodes([head(ns), d], {properties:'combine', mergeRels:true})
YIELD node
SET head(ns).name = lid
```
