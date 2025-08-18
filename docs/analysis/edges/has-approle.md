# HAS_APPROLE

Represents the relationship between Graph objects and their application roles.

## Usage

This relationship connects Graph objects to their defined application roles:

- **GraphApplication** → `HAS_APPROLE` → **GraphAppRole** - Applications to their defined roles
- **GraphServicePrincipal** → `HAS_APPROLE` → **GraphAppRole** - Service principals to their app roles

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all applications and their defined roles
MATCH (app:GraphApplication)-[:HAS_APPROLE]->(role:GraphAppRole)
RETURN app.displayName, role.displayName, role.value
```
