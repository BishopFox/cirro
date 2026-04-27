# Merge VMs by vmId

Merges duplicate `VirtualMachine` nodes that share the same `vmId`.

**Priority:** 1000

## Details

Restore point collections can introduce VM placeholders keyed by `vmId`. This step merges those placeholders with full VM nodes so relationships resolve to a single VM identity.

## Cypher

```cypher
CALL apoc.periodic.iterate(
  "
  MATCH (n:VirtualMachine)
  WHERE n.vmId IS NOT NULL
  WITH toLower(n.vmId) AS lid, collect(n) AS ns
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
  SET keep.vmId = lid
  RETURN 1
",
{batchSize: 1000, parallel: false}
)
```
