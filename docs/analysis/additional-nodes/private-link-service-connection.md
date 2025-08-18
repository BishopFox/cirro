# PrivateLinkServiceConnection

Represents private link service connections.

**Labels:** `:PrivateLinkServiceConnection`

**Properties:**

- `id` - Connection ID (primary key)
- `name` - Connection name
- `type` - Connection type
- `groupIds` - Array of group IDs

**Relationships:**
- `HAS_PRIVATE_ENDPOINT` → PrivateEndpoint
