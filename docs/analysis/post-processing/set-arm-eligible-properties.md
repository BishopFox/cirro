# Set ARM Eligible Relation Properties

Enriches `IS_RBAC_ELIGIBLE` relationships with permission details from existing role assignments.

**Priority:** 0

## Details

Finds role definition permissions from existing role assignment relationships and copies them (actions, notActions, dataActions, notDataActions) onto matching `IS_RBAC_ELIGIBLE` relationships.

## Cypher

```cypher
MATCH (:GraphObject)-[rel]->(o:ArmResource)
WHERE rel.type = 'microsoft.authorization/roleassignments'
WITH rel.roleDefinitionId AS roleDefinitionId, head(collect(rel)) AS reldef
WITH roleDefinitionId, {
    actions: reldef.actions,
    notActions: reldef.notActions,
    dataActions: reldef.dataActions,
    notDataActions: reldef.notDataActions
} AS permissions

MATCH (:GraphObject)-[r:IS_RBAC_ELIGIBLE]->(o:ArmResource)
WHERE r.roleDefinitionId = roleDefinitionId
SET r += {
    actions: permissions.actions,
    notActions: permissions.notActions,
    dataActions: permissions.dataActions,
    notDataActions: permissions.notDataActions
}
```
