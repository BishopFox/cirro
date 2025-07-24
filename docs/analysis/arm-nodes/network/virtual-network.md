# VirtualNetwork

Represents Azure virtual networks.

**Labels:** `:ArmResource:VirtualNetwork`

**Properties:**

- `id` - VNet resource ID (primary key)
- `addressPrefixes` - Array of address prefixes for the VNet

**Relationships:**
- `HAS_SUBNET` → Subnet
