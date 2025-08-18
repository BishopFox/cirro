# HAS_IDENTITY

Represents the relationship between Azure resources and their managed identities.

## Usage

This relationship connects Azure resources to their assigned managed identities:

- **ArmResource** → `HAS_IDENTITY` → **GraphObject** - ARM resources to their managed identities
- **UserAssignedIdentity** → `HAS_IDENTITY` → **GraphObject** - User-assigned identities to their Graph objects

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all resources with managed identities
MATCH (resource:ArmResource)-[:HAS_IDENTITY]->(identity:GraphObject)
RETURN resource.name, resource.type, identity.displayName
```
