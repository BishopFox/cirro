# AzureStackEdgeMachineNic

Represents network interface cards attached to Azure Stack HCI edge machines (not an ARM resource itself).

**Labels:** `:AzureStackEdgeMachineNic`

**Properties:**

- `machineId` - Parent edge machine ID (composite key with `adapterName`)
- `adapterName` - Network adapter name (composite key with `machineId`)
- `dnsServers` - DNS server addresses
- `ip4Address` - IPv4 address
- `macAddress` - MAC address
- `rdmaCapability` - RDMA capability of the adapter
- `subnetMask` - Subnet mask

## Relationships

### Incoming

- **AzureStackEdgeMachine** → `HAS_NIC` → **AzureStackEdgeMachineNic** - Parent edge machine

## Examples

```cypher
// Find edge machines and their NIC details
MATCH (em:AzureStackEdgeMachine)-[:HAS_NIC]->(nic:AzureStackEdgeMachineNic)
RETURN em.id, nic.adapterName, nic.ip4Address, nic.macAddress, nic.rdmaCapability
```
