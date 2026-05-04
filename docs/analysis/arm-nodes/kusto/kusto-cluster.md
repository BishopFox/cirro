# KustoCluster

Represents Azure Data Explorer (Kusto) cluster resources.

**Labels:** `:ArmResource:KustoCluster`

**Properties:**

- `id` - Cluster resource ID (primary key)
- `acceptedAudiences` - Audiences accepted by the cluster
- `allowedFqdnList` - List of allowed fully qualified domain names
- `allowedIpRangeList` - List of allowed IP ranges
- `dataIngestionUri` - URI for data ingestion
- `enableAutoStop` - Whether auto-stop is enabled
- `enableDiskEncryption` - Whether disk encryption is enabled
- `enableDoubleEncryption` - Whether double encryption is enabled
- `enablePurge` - Whether purge is enabled
- `enableStreamingIngest` - Whether streaming ingestion is enabled
- `engineType` - Type of engine (e.g., V3)
- `publicIpType` - Type of public IP configuration
- `publicNetworkAccess` - Public network access setting
- `restrictOutboundNetworkAccess` - Whether outbound network access is restricted
- `state` - Cluster state (e.g., Running, Unavailable)
- `stateReason` - Reason for the cluster state
- `trustedExternalTenants` - List of trusted external tenant IDs
- `uri` - Cluster URI/endpoint

## Examples

```cypher
// Find all Kusto clusters and their states
MATCH (kc:KustoCluster)
RETURN kc.id, kc.state, kc.uri, kc.enableAutoStop
```

```cypher
// Find Kusto clusters with specific security settings
MATCH (kc:KustoCluster)
WHERE kc.enableDoubleEncryption = true AND kc.publicNetworkAccess = 'Disabled'
RETURN kc.id, kc.uri
```
