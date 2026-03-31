# Deduplicate Graph Roles by Role Template ID

Merges duplicate GraphRole nodes that share the same `roleTemplateId`.

**Priority:** 1000

## Details

When multiple GraphRole nodes exist with the same role template ID, this step merges them into a single node, combining properties and relationships.

## Cypher

```cypher
MATCH (n:GraphRole)
WHERE n.roleTemplateId IS NOT NULL
WITH toLower(n.roleTemplateId) AS lid, collect(n) AS ns
WHERE size(ns) > 1
UNWIND tail(ns) AS d
CALL apoc.refactor.mergeNodes([head(ns), d], {properties:'combine', mergeRels:true})
YIELD node
RETURN 1
```
