# HybridExtension

Represents extensions installed on Azure Arc-enabled hybrid machines.

**Labels:** `:ArmResource:HybridExtension`

**Properties:**

- `id` - Extension resource ID (primary key)
- `name` - Extension name
- `type` - Resource type (`microsoft.hybridcompute/machines/extensions`)
- `location` - Resource location
- `extType` - Extension type (e.g. `CustomScriptExtension`)
- `typeHandlerVersion` - Type handler version
- `autoUpgradeMinorVersion` - Auto upgrade minor version
- `enableAutomaticUpgrade` - Enable automatic upgrade
- `statusMessage` - Extension status message
- `provisioningState` - Provisioning state
- `publisher` - Extension publisher
- `settings` - Serialized extension settings (JSON)

## Relationships

### Incoming

- **HybridMachine** → `HAS_EXTENSION` → **HybridExtension** - Parent hybrid machine

## Examples

```cypher
// Find all hybrid machine extensions
MATCH (hm:HybridMachine)-[:HAS_EXTENSION]->(ext:HybridExtension)
RETURN hm.displayName, ext.name, ext.extType, ext.provisioningState
```

```cypher
// Find extensions with auto upgrade enabled
MATCH (ext:HybridExtension)
WHERE ext.enableAutomaticUpgrade = true
RETURN ext.name, ext.extType, ext.typeHandlerVersion
```
