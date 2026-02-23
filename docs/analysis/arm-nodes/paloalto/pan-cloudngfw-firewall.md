# PanCloudNgfwFirewall

Represents Palo Alto Cloud NGFW Firewall resources.

**Labels:** `:ArmResource:PanCloudNgfwFirewall`

**Properties:**

- `id` - Resource ID (primary key)
- `enableDnsProxy` - Whether DNS proxy is enabled
- `enabledDnsType` - DNS type enabled
- `isPanoramaManaged` - Whether managed by Panorama
- `isStrataCloudManaged` - Whether managed by Strata Cloud
- `enableEgressNat` - Whether egress NAT is enabled
- `networkType` - Network type from network profile

## Relationships

### Outgoing

- **PanCloudNgfwFirewall** → `HAS_RULESTACK` → **PanCloudNgfwLocalRulestack** - Associated local rulestack
- **PanCloudNgfwFirewall** → `HAS_IP` → **PublicIPAddress** - Public IP addresses assigned to the firewall

## Examples

```cypher
// Find firewalls and their rulestacks
MATCH (fw:PanCloudNgfwFirewall)-[:HAS_RULESTACK]->(rs:PanCloudNgfwLocalRulestack)
RETURN fw.name, rs.name
```
