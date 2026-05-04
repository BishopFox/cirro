# AzureStackEdgeMachine

Represents Azure Stack HCI edge machine resources.

**Labels:** `:ArmResource:AzureStackEdgeMachine`

**Properties:**

- `id` - Edge machine resource ID (primary key)
- `arcMachineResourceGroupId` - Resource group ID of the associated Arc machine
- `arcMachineResourceId` - Resource ID of the associated Arc machine
- `cloudId` - Cloud identifier this edge machine is registered with
- `connectivityStatus` - Connectivity status of the edge machine
- `devicePoolResourceId` - Device pool resource ID
- `edgeMachineKind` - Kind/type of edge machine
- `lastUpdated` - Last update timestamp
- `lastSyncTimestamp` - Last synchronization timestamp
- `machineState` - Current machine state

## Relationships

### Outgoing

- **AzureStackEdgeMachine** → `REGISTERED_BY` → **GraphObject** - Identity that registered this edge machine
- **AzureStackEdgeMachine** → `HAS_NIC` → **AzureStackEdgeMachineNic** - Network interfaces attached to this edge machine

## Examples

```cypher
// Find all edge machines and their connectivity status
MATCH (em:AzureStackEdgeMachine)
RETURN em.id, em.edgeMachineKind, em.connectivityStatus, em.machineState
```

```cypher
// Find edge machines and their network interfaces
MATCH (em:AzureStackEdgeMachine)-[:HAS_NIC]->(nic:AzureStackEdgeMachineNic)
RETURN em.id, nic.adapterName, nic.ip4Address, nic.macAddress
```
