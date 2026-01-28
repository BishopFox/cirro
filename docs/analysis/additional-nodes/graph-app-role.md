# GraphAppRole

Represents application roles within Entra ID applications.

**Labels:** `:GraphAppRole`

**Properties:**

- `id` - App role ID (primary key)
- `displayName` - Role display name
- `description` - Role description
- `allowedMemberTypes` - Array of allowed member types
- `isEnabled` - Whether the role is enabled
- `isPreAuthorizationRequired` - Whether pre-authorization is required
- `isPrivate` - Whether the role is private
- `origin` - Role origin
- `value` - Role value

**Relationships:**
- `HAS_APPROLE` ← GraphApplication
- `HAS_APPROLE` ← GraphServicePrincipal
