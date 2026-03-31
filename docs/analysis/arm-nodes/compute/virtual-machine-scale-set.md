# VMScaleSet

Represents Azure Virtual Machine Scale Sets.

**Labels:** `:ArmResource:VMScaleSet`

**Properties:**

- `id` - Resource ID (primary key)
- `constrainedMaximumCapacity` - Max capacity when constrained
- `orchestrationMode` - Orchestration mode (Flexible/Uniform)
- `uniqueId` - Unique identifier
- `timeCreated` - Creation timestamp
- `adminUsername` - Admin username on the VM profile
- `computerNamePrefix` - Computer name prefix for instances

## Examples

```cypher
// List VMSS with orchestration mode
MATCH (s:VMScaleSet)
RETURN s.id, s.orchestrationMode, s.constrainedMaximumCapacity
```
