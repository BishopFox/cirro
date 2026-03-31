# HybridExtension

Represents extensions installed on hybrid machines (not an ARM resource itself).

**Labels:** `:HybridExtension`

**Properties:**

- `id` - Extension ID (primary key)
- `name` - Extension name
- `type` - Extension type
- `location` - Extension location
- `typeHandlerVersion` - Type handler version
- `autoUpgradeMinorVersion` - Auto upgrade minor version setting
- `enableAutomaticUpgrade` - Enable automatic upgrade setting
- `statusMessage` - Extension status message
- `provisioningState` - Provisioning state

## Relationships

### Incoming

- **HybridMachine** → `HAS_EXTENSION` → HybridExtension - Parent hybrid machine

## Examples

```cypher
// Find all hybrid machine extensions
MATCH (hm:HybridMachine)-[:HAS_EXTENSION]->(ext:HybridExtension)
RETURN hm.displayName, ext.name, ext.type, ext.provisioningState
```

```cypher
// Find extensions with auto upgrade enabled
MATCH (ext:HybridExtension)
WHERE ext.enableAutomaticUpgrade = true
RETURN ext.name, ext.typeHandlerVersion
```