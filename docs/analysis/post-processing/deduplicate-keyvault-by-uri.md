# Deduplicate Key Vault by URI

Merges duplicate KeyVault nodes that share the same `vaultUri`.

**Priority:** 1000

## Details

When multiple KeyVault nodes have the same `vaultUri`, this step merges them into a single node, combining properties and relationships. The surviving `vaultUri` is normalized to lowercase so later lookups stay consistent.

## Cypher

```cypher
CALL apoc.periodic.iterate(
  "
  MATCH (n:KeyVault)
  WHERE n.vaultUri IS NOT NULL
  WITH toLower(n.vaultUri) AS lid, collect(n) AS ns
  WHERE size(ns) > 1
  RETURN lid, ns
  ",
  "
  WITH lid, ns
  WITH
      head(ns) AS keep,
      tail(ns) AS dups,
      lid
  UNWIND dups AS d
  CALL apoc.refactor.mergeNodes(
      [keep, d],
      {properties:'combine', mergeRels:true}
  )
  YIELD node
  SET keep.vaultUri = lid
  RETURN 1
  ",
  {batchSize: 1000, parallel: false}
)
```
