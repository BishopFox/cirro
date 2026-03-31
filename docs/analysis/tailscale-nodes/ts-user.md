# TSUser

Represents a Tailscale user who can own devices within a Tailscale network.

**Labels:** `:TSUser`

**Properties:**

- `id` - Unique user identifier (primary key)
- `loginName` - User's login name
- `displayName` - User's display name

## Relationships

- **OWNS** → **TSDevice** - Users own the devices they register to the Tailscale network