# HAS_SCOPED_ROLE

Represents a scoped role assignment on a Graph Administrative Unit.

## Usage

- **GraphObject** → `HAS_SCOPED_ROLE` → **GraphAdministrativeUnit** - Principal has a scoped role on the administrative unit

## Properties

- `displayName` - Role display name (set by post-processing)
- `description` - Role description (set by post-processing)
- `roleTemplateId` - Role template ID (set by post-processing)
- `roleId` - Original role ID

## Examples

```cypher
// Find users with scoped roles on administrative units
MATCH (u:GraphUser)-[r:HAS_SCOPED_ROLE]->(au:GraphAdministrativeUnit)
RETURN u.displayName, r.displayName AS role, au.displayName AS adminUnit
```
