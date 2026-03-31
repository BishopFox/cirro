# Set AppRole Relation Properties

Enriches `ASSIGNED_APPROLE` relationships with properties from the corresponding GraphAppRole node.

**Priority:** 0

## Details

Matches `ASSIGNED_APPROLE` relationships to their corresponding `GraphAppRole` node via the `appRoleId` property, then copies display name, description, and other metadata onto the relationship.

## Cypher

```cypher
MATCH (:GraphObject)-[r:ASSIGNED_APPROLE]->(o:GraphServicePrincipal)-[:HAS_APPROLE]->(ar:GraphAppRole)
WHERE r.appRoleId = ar.id
SET r += {
    displayName: ar.displayName,
    description: ar.description,
    isEnabled: ar.isEnabled,
    isPreAuthorizationRequired: ar.isPreAuthorizationRequired,
    isPrivate: ar.isPrivate,
    origin: ar.origin,
    value: ar.value
}
```
