# Remove MemberOf GraphRole Relations

Removes all `MEMBER_OF` relationships pointing to GraphRole nodes.

**Priority:** 0

## Details

Since the graph uses `HAS_ROLE` relationships for role membership, this step removes redundant `MEMBER_OF` relationships between GraphObject and GraphRole nodes.

## Cypher

```cypher
MATCH (:GraphObject)-[r:MEMBER_OF]->(:GraphRole)
DELETE r
```
