# NetworkRoute

Represents individual routes within route tables.

**Labels:** `:NetworkRoute`

**Properties:**

- `id` - Route ID (primary key)
- `name` - Route name
- `type` - Route type
- `addressPrefix` - Address prefix
- `nextHopType` - Next hop type
- `nextHopIpAddress` - Next hop IP address
- `hasBgpOverride` - Whether BGP override is enabled

**Relationships:**
- `HAS_ROUTE` ← RouteTable
