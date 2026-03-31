# HAS_PRIVATE_ENDPOINT

Represents the relationship between Azure resources and their private endpoints.

## Usage

This relationship connects Azure resources to their associated private endpoints:

- **ArmResource** → `HAS_PRIVATE_ENDPOINT` → **PrivateEndpoint** - Azure resources to their private endpoints

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all resources with private endpoints
MATCH (resource:ArmResource)-[:HAS_PRIVATE_ENDPOINT]->(pe:PrivateEndpoint)
RETURN resource.name, resource.type, pe.name
```
