# Subnet

Represents subnets within virtual networks (not an ARM resource itself).

**Labels:** `:Subnet`

**Properties:**

- `id` - Subnet ID (primary key)
- `name` - Subnet name
- `type` - Subnet type
- `addressPrefix` - Address prefix
- `privateEndpointNetworkPolicies` - Private endpoint network policies
- `privateLinkServiceNetworkPolicies` - Private link service network policies

**Relationships:**
- `HAS_SUBNET` ← VirtualNetwork
- `CONTAINS` → IPConfig
