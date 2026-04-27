# HAS_INSTANCE

Represents the relationship between container resources and their concrete instances.

## Usage

This relationship connects restore point collections to their restore point instances:

- **RestorePointCollection** -> `HAS_INSTANCE` -> **RestorePoint** - Collection to restore point instance

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find restore point collections and their instances
MATCH (rpc:RestorePointCollection)-[:HAS_INSTANCE]->(rp:RestorePoint)
RETURN rpc.id, rp.id, rp.timeCreated
```
