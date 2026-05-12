# LoadBalancerRule

Represents load balancing rule definitions attached to a load balancer.

**Labels:** `:LoadBalancerRule`

**Properties:**

- `id` - Rule resource ID (primary key)
- `name` - Rule name
- `protocol` - Protocol used by the rule
- `frontendPort` - Frontend listening port
- `backendPort` - Backend target port
- `allowBackendPortConflict` - Whether backend port conflict is allowed
- `disableConnectionTracking` - Whether connection tracking is disabled
- `disableOutboundSnat` - Whether outbound SNAT is disabled
- `enableConnectionTracking` - Whether connection tracking is enabled
- `enableDestinationServiceEndpoint` - Whether destination service endpoint is enabled
- `enableFloatingIP` - Whether floating IP is enabled
- `enableTcpReset` - Whether TCP reset is enabled
- `idleTimeoutInMinutes` - Idle timeout value
- `provisioningState` - Current provisioning state
- `type` - Azure resource type value

## Relationships

### Incoming

- **LoadBalancer** -> `HAS_RULE` -> **LoadBalancerRule** - Load balancer that owns the rule
