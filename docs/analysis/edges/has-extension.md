# HAS_EXTENSION

Represents the relationship between compute resources and their extensions.

## Usage

This relationship connects compute resources to their installed extensions:

- **VirtualMachine** → `HAS_EXTENSION` → **VMExtension** - Virtual machines to their extensions
- **HybridMachine** → `HAS_EXTENSION` → **HybridExtension** - Hybrid machines to their extensions

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all virtual machines and their extensions
MATCH (vm:VirtualMachine)-[:HAS_EXTENSION]->(ext)
RETURN vm.name, ext.name, ext.publisher, ext.type
```
