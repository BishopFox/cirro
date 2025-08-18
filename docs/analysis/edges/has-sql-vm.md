# HAS_SQL_VM

Represents the relationship between virtual machines and their SQL virtual machine configurations.

## Usage

This relationship connects virtual machines to their SQL Server configurations:

- **VirtualMachine** → `HAS_SQL_VM` → **SqlVirtualMachine** - Virtual machines to their SQL configurations

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all virtual machines with SQL Server configurations
MATCH (vm:VirtualMachine)-[:HAS_SQL_VM]->(sqlvm:SqlVirtualMachine)
RETURN vm.name, sqlvm.sqlServerLicenseType, sqlvm.sqlManagement
```
