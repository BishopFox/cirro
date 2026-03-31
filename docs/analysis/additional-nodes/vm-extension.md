# VMExtension

Represents an extension attached to an Azure virtual machine.

**Labels:** `:VMExtension`

**Properties:**

- `id` - Extension resource ID (primary key)
- `name` - Extension name
- `extType` - Extension type
- `location` - Resource location
- `provisioningState` - Provisioning state
- `publisher` - Extension publisher
- `autoUpgradeMinorVersion` - Auto-upgrade behavior
- `triggerForceUpgrade` - Force-upgrade trigger
- `vmType` - VM type metadata from extension settings
- `settings` - Serialized extension settings

## Relationships

### Incoming

- **VirtualMachine** → `HAS_EXTENSION` → **VMExtension** - Extension bound to a VM
- **HybridMachine** → `HAS_EXTENSION` → **VMExtension** - Extension attached to an Arc/hybrid machine

## Examples

```cypher
// Find extensions by publisher across VMs
MATCH (vm:VirtualMachine)-[:HAS_EXTENSION]->(ext:VMExtension)
RETURN vm.name, ext.name, ext.publisher, ext.provisioningState
```
