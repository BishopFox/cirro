# EventGridTopic

Represents Azure Event Grid topics and system topics.

**Labels:** `:ArmResource:EventGridTopic`

**Properties:**

- `id` - Event Grid topic resource ID (primary key)
- `topicType` - Topic type (for system topics)
- `dataResidencyBoundary` - Data residency boundary setting
- `disableLocalAuth` - Whether local authentication is disabled
- `endpoint` - Topic endpoint URL
- `inputSchema` - Input schema type
- `minimumTlsVersionAllowed` - Minimum TLS version allowed
- `publicNetworkAccess` - Public network access setting

**Relationships:**
- `HAS_SOURCE` → ArmResource (for system topics)
