# PanCloudNgfwLocalRulestack

Represents Palo Alto Cloud NGFW Local Rulestack resources.

**Labels:** `:ArmResource:PanCloudNgfwLocalRulestack`

**Properties:**

- `id` - Resource ID (primary key)
- `defaultMode` - Default mode
- `description` - Description
- `minAppIdVersion` - Minimum App-ID version
- `provisioningState` - Provisioning state
- `scope` - Scope
- `antiSpywareProfile` - Anti-spyware profile
- `antiVirusProfile` - Anti-virus profile
- `dnsSubscription` - DNS subscription
- `fileBlockingProfile` - File blocking profile
- `urlFilteringProfile` - URL filtering profile
- `vulnerabilityProfile` - Vulnerability profile

## Relationships

### Incoming

- **PanCloudNgfwFirewall** → `HAS_RULESTACK` → **PanCloudNgfwLocalRulestack**

## Examples

```cypher
// List all local rulestacks with their security profiles
MATCH (rs:PanCloudNgfwLocalRulestack)
RETURN rs.name, rs.antiSpywareProfile, rs.antiVirusProfile, rs.vulnerabilityProfile
```
