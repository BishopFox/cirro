# ManagementGroup

Azure Management Groups provide a way to efficiently manage access, policies, and compliance across multiple Azure subscriptions.

**Labels:** `:ManagementGroup`

## Properties

- `id` - Management Group ID (primary key)
- `displayName` - Display name of the management group

## Relationships

- `Tenant` → `CONTAINS` → `ManagementGroup` - Root management groups under tenants
- `ManagementGroup` → `CONTAINS` → `ManagementGroup` - Parent-child management group hierarchy
- `ManagementGroup` → `CONTAINS` → `Subscription` - Management groups containing subscriptions
