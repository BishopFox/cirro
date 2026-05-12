# LoadBalancerFrontendIPConfiguration

Represents frontend IP configuration objects for a load balancer.

**Labels:** `:LoadBalancerFrontendIPConfiguration`

**Properties:**

- `id` - Frontend configuration resource ID (primary key)
- `name` - Frontend configuration name
- `privateIPAllocationMethod` - Private IP allocation method
- `provisioningState` - Current provisioning state
- `type` - Azure resource type value

## Relationships

### Incoming

- **LoadBalancer** -> `HAS_LB_FRONTEND` -> **LoadBalancerFrontendIPConfiguration** - Parent load balancer

### Outgoing

- **LoadBalancerFrontendIPConfiguration** -> `HAS_IP` -> **PublicIPAddress** - Associated public IP (if present)
