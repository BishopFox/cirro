# Deduplicate Container Registries by Server

Merges duplicate Container Registry nodes that share the same `loginServer` value.

**Priority:** 1000

## Details

When multiple ContainerRegistry nodes exist with the same login server address, this step merges them into a single node using `apoc.refactor.mergeNodes`, combining properties and relationships. The `loginServer` is normalized to lowercase.

## Cypher

```cypher
MATCH (n:ContainerRegistry)
WHERE n.loginServer IS NOT NULL
WITH toLower(n.loginServer) AS lid, collect(n) AS ns
WHERE size(ns) > 1
// Merge duplicates into the first node
UNWIND tail(ns) AS d
CALL apoc.refactor.mergeNodes([head(ns), d], {properties:'combine', mergeRels:true})
YIELD node
SET head(ns).loginServer = lid
```
