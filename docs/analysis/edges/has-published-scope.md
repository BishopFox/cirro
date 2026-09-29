# HAS_PUBLISHED_SCOPE

Links a service principal to a permission scope in its `publishedPermissionScopes` collection.

## Usage

- **GraphServicePrincipal** → `HAS_PUBLISHED_SCOPE` → **GraphApplicationScope** - Published permission scope

This relationship describes a published scope, not a consent grant or role assignment.

## Properties

No additional properties on the relationship.

## Examples

```cypher
MATCH (sp:GraphServicePrincipal)-[:HAS_PUBLISHED_SCOPE]->(scope:GraphApplicationScope)
RETURN sp.displayName, scope.value, scope.isEnabled
```
