# Subscription

Represents Azure subscriptions - billing and management units within a tenant.

**Labels:** `:Subscription:ArmResource`

**Properties:**

- `id` - Subscription ID (primary key)
- `displayName` - Subscription display name
- `authorizationSource` - Authorization source
- `state` - Subscription state (Enabled, Disabled, etc.)
- `subscriptionId` - Subscription identifier
- `tenantId` - Associated tenant ID

**Relationships:**
- `CONTAINS` → ResourceGroup
