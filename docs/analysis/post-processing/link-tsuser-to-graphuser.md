# Link TSUser to GraphUser

Creates `SAME_USER` relationships between TSUser and GraphUser nodes matched by login name.

**Priority:** 1000

## Details

Finds TSUser nodes that are not yet linked to a GraphUser via `SAME_USER`, then matches them by comparing the TSUser's `loginName` to the GraphUser's `userPrincipalName` (case-insensitive).

## Cypher

```cypher
MATCH (tsu:TSUser)
WHERE NOT (tsu)-[:SAME_USER]->(:GraphUser)
OPTIONAL MATCH (gu:GraphUser)
WHERE toLower(gu.userPrincipalName) = toLower(tsu.loginName)
FOREACH (_ IN CASE WHEN gu IS NULL THEN [] ELSE [1] END |
  MERGE (tsu)-[:SAME_USER]->(gu)
)
```
