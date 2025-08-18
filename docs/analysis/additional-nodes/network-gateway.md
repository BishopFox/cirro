# NetworkGateway

Represents virtual network gateways.

**Labels:** `:NetworkGateway`

**Properties:**

- `id` - Gateway ID (primary key)

**Relationships:**
- `HAS_GATEWAY` ← NetworkPeering
- `CONNECTS` → VirtualNetwork
