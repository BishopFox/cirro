# PrivateEndpoint

Represents Azure private endpoints.

**Labels:** `:ArmResource:PrivateEndpoint`

**Properties:**

- `id` - Private endpoint resource ID (primary key)
- `customNetworkInterfaceName` - Custom network interface name
- `ipVersionType` - IP version type

**Relationships:**
- `HAS_DNS_CONFIG` → CustomDnsConfig
- `CONTAINS` ← Subnet
- `HAS_PRIVATE_ENDPOINT` ← ArmResource (services)
