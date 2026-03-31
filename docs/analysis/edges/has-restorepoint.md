# HAS_RESTOREPOINT

Represents the relationship between virtual machines and their restore points.

## Usage

This relationship connects virtual machines to their restore point collections:

- **VirtualMachine** → `HAS_RESTOREPOINT` → **RestorePointCollection** - Virtual machines to their restore points

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all virtual machines with restore points
MATCH (vm:VirtualMachine)-[:HAS_RESTOREPOINT]->(rp:RestorePointCollection)
RETURN vm.name, rp.name, rp.restorePointCollectionSourceId
```
