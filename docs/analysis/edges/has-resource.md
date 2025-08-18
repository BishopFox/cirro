# HAS_RESOURCE

Represents the relationship between resource groups and their contained resources.

## Usage

This relationship connects resource groups to the Azure resources they contain:

- **ResourceGroup** → `HAS_RESOURCE` → **ArmResource** - Resource groups to their contained resources

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all resource groups and their resources
MATCH (rg:ResourceGroup)-[:HAS_RESOURCE]->(resource:ArmResource)
RETURN rg.name, resource.name, resource.type
```
