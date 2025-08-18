# HAS_VMAPP

Represents the relationship between virtual machines and their VM applications.

## Usage

This relationship connects virtual machines to their installed VM applications:

- **VirtualMachine** → `HAS_VMAPP` → **VMApplication** - Virtual machines to their VM applications

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all virtual machines with VM applications
MATCH (vm:VirtualMachine)-[:HAS_VMAPP]->(vmapp:VMApplication)
RETURN vm.name, vmapp.name, vmapp.packageReferenceId
```
