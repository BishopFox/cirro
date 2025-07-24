# UserAssignedIdentity

Represents user-assigned managed identities.

**Labels:** `:ArmResource:UserAssignedIdentity`

**Properties:**

- `id` - Identity resource ID (primary key)
- `clientId` - Client ID of the identity
- `tenantId` - Tenant ID where the identity exists

**Relationships:**
- `HAS_IDENTITY` → GraphObject (links to the service principal)
- `HAS_IDENTITY` ← ArmResource (resources using this identity)
