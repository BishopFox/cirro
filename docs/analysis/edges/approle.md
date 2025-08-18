# APPROLE

Represents the assignment of application roles to Graph objects.

## Usage

This relationship connects Graph objects to the application roles assigned to them:

- **GraphObject** → `APPROLE` → **GraphApplication** - Objects with app role assignments
- **GraphObject** → `APPROLE` → **GraphServicePrincipal** - Objects with service principal role assignments

## Properties

- `appRoleId` - The ID of the specific application role assigned

## Examples

```cypher
// Find all users with application role assignments
MATCH (user:GraphUser)-[r:APPROLE]->(app:GraphApplication)
RETURN user.displayName, app.displayName, r.appRoleId
```
