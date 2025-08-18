# ManagementGroup

Azure Management Groups provide a way to efficiently manage access, policies, and compliance across multiple Azure subscriptions.

**Labels:** `:ManagementGroup`

## Properties

- `id` - Management Group ID (primary key)
- `displayName` - Display name of the management group

## Relationships

- `Tenant` → `HAS_ENTITY` → `ManagementGroup` - Root management groups under tenants
- `ManagementGroup` → `HAS_ENTITY` → `ManagementGroup` - Parent-child management group hierarchy
- `ManagementGroup` → `HAS_SUBSCRIPTION` → `Subscription` - Management groups containing subscriptions
