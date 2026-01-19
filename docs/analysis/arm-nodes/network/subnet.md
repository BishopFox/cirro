# Subnet

Represents subnets within virtual networks (not an ARM resource itself).

**Labels:** `:Subnet`

**Properties:**

- `id` - Subnet ID (primary key)
- `name` - Subnet name
- `addressPrefix` - Address prefix for the subnet
- `privateEndpointNetworkPolicies` - Private endpoint network policies setting
- `privateLinkServiceNetworkPolicies` - Private link service network policies setting

## Relationships

### Incoming

- **VirtualNetwork** → `CONTAINS` → **Subnet** - Parent virtual network

### Outgoing

- **Subnet** → `CONTAINS` → **IPConfiguration** - IP configurations in this subnet
- **Subnet** → `HAS_CONFIG` → **RouteTable** - Associated route table
- **Subnet** → `HAS_NSG` → **NetworkSecurityGroup** - Associated network security group

## Examples

```cypher
// Find subnets and their address prefixes
MATCH (vnet:VirtualNetwork)-[:CONTAINS]->(subnet:Subnet)
RETURN vnet.name, subnet.name, subnet.addressPrefix
```

```cypher
// Find subnets with route tables
MATCH (subnet:Subnet)-[:HAS_CONFIG]->(rt:RouteTable)
RETURN subnet.name, rt.name
```
