# AnalysisServiceFirewallRule

Represents firewall rule entries attached to an Analysis Services server.

**Labels:** `:AnalysisServiceFirewallRule`

**Properties:**

- `name` - Firewall rule name
- `serverId` - Parent Analysis Services server resource ID (used with `name` as a composite key)
- `rangeStart` - Start of allowed IPv4 range
- `rangeEnd` - End of allowed IPv4 range

## Relationships

### Incoming

- **AnalysisServiceServer** → `HAS_RULE` → **AnalysisServiceFirewallRule** - Server that owns the firewall rule

## Examples

```cypher
// List firewall rules per Analysis Services server
MATCH (s:AnalysisServiceServer)-[:HAS_RULE]->(r:AnalysisServiceFirewallRule)
RETURN s.serverFullName, r.name, r.rangeStart, r.rangeEnd
ORDER BY s.serverFullName, r.name
```