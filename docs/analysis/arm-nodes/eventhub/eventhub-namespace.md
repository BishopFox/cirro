# EventHubNamespace

Represents an Event Hub namespace resource for streaming and event ingestion.

**Labels:** `:ArmResource`, `:EventHubNamespace`

**Properties:**

- `id` - Resource identifier (primary key)
- `createdAt` - Timestamp when the namespace was created
- `disableLocalAuth` - Whether local authentication is disabled
- `isAutoInflateEnabled` - Whether auto-inflate is enabled for throughput units
- `kafkaEnabled` - Whether Kafka is enabled for the namespace
- `publicNetworkAccess` - Public network access configuration
- `serviceBusEndpoint` - Service Bus endpoint URL
- `status` - Current status of the namespace
- `updatedAt` - Timestamp when the namespace was last updated