# RouteTable

Represents Azure route tables.

**Labels:** `:ArmResource:RouteTable`

**Properties:**

- `id` - Route table resource ID (primary key)
- `disableBgpRoutePropagation` - Whether BGP route propagation is disabled

**Relationships:**
- `HAS_ROUTE` → NetworkRoute
- `HAS_ROUTE_TABLE` ← Subnet
