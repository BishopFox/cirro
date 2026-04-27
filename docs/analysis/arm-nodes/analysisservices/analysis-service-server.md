# AnalysisServiceServer

Represents Azure Analysis Services server resources.

**Labels:** `:ArmResource:AnalysisServiceServer`

**Properties:**

- `id` - Resource ID (primary key)
- `managedMode` - Indicates whether the server is managed
- `serverFullName` - Full server name
- `serverMonitorMode` - Monitoring mode for the server
- `state` - Current state of the server
- `administrators` - Raw administrator member entries from the resource
- `enablePowerBIService` - Whether Power BI service access is enabled

## Relationships

### Incoming

- **GraphServicePrincipal** → `IS_ADMIN` → **AnalysisServiceServer** - Service principal listed as administrator
- **GraphObject** → `IS_ADMIN` → **AnalysisServiceServer** - Directory object listed as administrator
- **GraphUser** → `IS_ADMIN` → **AnalysisServiceServer** - User listed as administrator

### Outgoing

- **AnalysisServiceServer** → `HAS_RULE` → **AnalysisServiceFirewallRule** - Firewall rules defined on the server

## Examples

```cypher
// Find Analysis Services servers and administrator principals
MATCH (admin)-[:IS_ADMIN]->(s:AnalysisServiceServer)
RETURN s.serverFullName, labels(admin), admin
```

```cypher
// Find Analysis Services servers and their firewall rules
MATCH (s:AnalysisServiceServer)-[:HAS_RULE]->(r:AnalysisServiceFirewallRule)
RETURN s.serverFullName, r.name, r.rangeStart, r.rangeEnd
```