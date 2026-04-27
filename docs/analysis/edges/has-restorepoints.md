# HAS_RESTOREPOINTS

Represents the relationship between virtual machines and restore point collections.

## Usage

This relationship connects virtual machines to their restore point collections:

- **VirtualMachine** -> `HAS_RESTOREPOINTS` -> **RestorePointCollection** - VM to collection ownership

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all virtual machines with restore point collections
MATCH (vm:VirtualMachine)-[:HAS_RESTOREPOINTS]->(rpc:RestorePointCollection)
RETURN vm.name, rpc.id, rpc.restorePointCollectionId
```
