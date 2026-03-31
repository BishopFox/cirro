# VerifiedDomain

Represents verified domains associated with Entra ID organizations. These are domains that have been verified as belonging to the organization.

**Labels:** `:VerifiedDomain`

**Properties:**

- `name` - Domain name (primary key)
- `isDefault` - Whether this is the default domain for the organization
- `isInitial` - Whether this is the initial domain created with the tenant
- `type` - Type of domain verification

**Notes:**
- The default domain is typically used for new user creation