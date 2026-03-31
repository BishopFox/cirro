# IS_RBAC_ELIGIBLE

Represents eligible (PIM) Azure RBAC role assignments where a principal can activate a role on a scope.

## Usage

- **GraphObject** → `IS_RBAC_ELIGIBLE` → **ArmResource** - A principal is eligible to activate an RBAC role on a resource scope

## Properties

- `id` - Assignment ID
- `name` - Assignment name
- `type` - Resource type
- `memberType` - Member type (e.g., Direct, Group)
- `createdOn` - When the assignment was created
- `startDateTime` - Eligibility start time
- `endDateTime` - Eligibility end time
- `scope` - Target resource scope
- `roleName` - Display name of the role definition
- `roleDefinitionId` - Role definition resource ID

## Examples

```cypher
// Find all eligible RBAC assignments
MATCH (p:GraphObject)-[r:IS_RBAC_ELIGIBLE]->(scope:ArmResource)
RETURN p.displayName, r.roleName, scope.name
```

```cypher
// Find users eligible for Owner role
MATCH (u:GraphUser)-[r:IS_RBAC_ELIGIBLE]->(scope:ArmResource)
WHERE r.roleName = "Owner"
RETURN u.userPrincipalName, scope.name, r.startDateTime, r.endDateTime
```
