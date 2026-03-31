# Deduplicate Graph Applications by App ID

Merges duplicate GraphApplication nodes that share the same `appId`.

**Priority:** 1000

## Details

When multiple GraphApplication nodes exist with the same app ID, this step merges them into a single node, combining properties and relationships. The `appId` is normalized to lowercase.

## Cypher

```cypher
MATCH (n:GraphApplication)
WHERE n.appId IS NOT NULL
WITH toLower(n.appId) AS lid, collect(n) AS ns
WHERE size(ns) > 1
UNWIND tail(ns) AS d
CALL apoc.refactor.mergeNodes([head(ns), d], {properties:'combine', mergeRels:true})
YIELD node
SET head(ns).appId = lid
```
