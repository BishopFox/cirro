# HAS_ENTITY

Represents hierarchical relationships in Azure management structures.

## Usage

This relationship connects Azure management entities in hierarchical structures:

- **Tenant** → `HAS_ENTITY` → **ManagementGroup** - Root management groups under tenants
- **ManagementGroup** → `HAS_ENTITY` → **ManagementGroup** - Parent-child management group relationships

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find root management groups (directly under tenant)
MATCH (t:Tenant)-[:HAS_ENTITY]->(mg:ManagementGroup)
RETURN t.displayName, mg.displayName

// Find management group hierarchy
MATCH path = (root:ManagementGroup)-[:HAS_ENTITY*]->(child:ManagementGroup)
WHERE NOT (root)<-[:HAS_ENTITY]-(:ManagementGroup)
RETURN path

// Find all entities under a specific management group
MATCH (mg:ManagementGroup {displayName: 'Production'})-[:HAS_ENTITY*]->(entity)
RETURN mg.displayName, entity
```
