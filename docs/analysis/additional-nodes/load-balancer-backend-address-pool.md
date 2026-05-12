# LoadBalancerBackendAddressPool

Represents backend address pool objects attached to a load balancer.

**Labels:** `:LoadBalancerBackendAddressPool`

**Properties:**

- `id` - Backend address pool resource ID (primary key)
- `name` - Backend pool name
- `provisioningState` - Current provisioning state
- `type` - Azure resource type value

## Relationships

### Incoming

- **LoadBalancer** -> `HAS_BACKEND_POOL` -> **LoadBalancerBackendAddressPool** - Parent load balancer

### Outgoing

- **LoadBalancerBackendAddressPool** -> `ASSOCIATED_WITH` -> **NetworkInterface** - Backend NIC configuration reference
