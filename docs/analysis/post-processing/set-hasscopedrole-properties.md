# Set HasScopedRole Relation Properties

Enriches `HAS_SCOPED_ROLE` relationships with role details from the corresponding GraphRole node.

**Priority:** 0

## Details

Matches `HAS_SCOPED_ROLE` relationships on GraphAdministrativeUnit nodes to their GraphRole by `roleId`, then copies display name, description, and role template ID onto the relationship.

## Cypher

```cypher
MATCH (:GraphObject)-[r:HAS_SCOPED_ROLE]->(o:GraphAdministrativeUnit)
MATCH (gr:GraphRole {id: toLower(r.roleId)})
SET r += {
    displayName: gr.displayName,
    description: gr.description,
    roleTemplateId: gr.roleTemplateId
}
```
