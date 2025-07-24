# Additional Resource Types

The following additional resource types are also created with their specific properties:

- **AutomationAccount** - Azure Automation accounts
- **AvailabilitySet** - VM availability sets
- **ArcSqlServer** - Azure Arc-enabled SQL servers
- **ClassicStorageAccount** - Classic storage accounts
- **ContainerRegistry** - Azure container registries
- **CommunicationServiceAccount** - Communication service accounts
- **Disk** - Azure managed disks
- **DnsZone** - DNS zones
- **HybridMachine** - Azure Arc-enabled machines
- **PublicIPAddress** - Public IP addresses
- **RestorePointCollection** - VM restore point collections
- **Snapshot** - Disk snapshots
- **SSHPublicKey** - SSH public keys
- **UserAssignedIdentity** - User-assigned managed identities

Each resource type includes the standard ARM resource properties plus type-specific properties relevant to that service.

## Notes

- All node IDs are automatically converted to lowercase during ingestion
- Nodes with the same ID are automatically merged during post-processing
- Properties may be null/empty if not provided by the Azure APIs
- Additional properties may be present depending on the specific Azure service configuration
