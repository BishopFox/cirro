# HybridMachine

Represents Azure Arc-enabled machines (hybrid/on-premises machines managed by Azure).

**Labels:** `:ArmResource:HybridMachine`

**Properties:**

- `id` - Hybrid machine resource ID (primary key)
- `adFqdn` - Active Directory FQDN
- `agentVersion` - Azure Arc agent version
- `clientPublicKey` - Client public key
- `cloud` - Cloud provider metadata
- `displayName` - Display name
- `dnsFqdn` - DNS FQDN
- `domainName` - Domain name
- `lastStatusChange` - Last status change timestamp
- `machineFqdn` - Machine FQDN
- `osName` - Operating system name
- `computerName` - Computer name
- `osSku` - Operating system SKU
- `osVersion` - Operating system version
- `status` - Machine status
- `vmId` - Virtual machine ID
- `vmUuid` - Virtual machine UUID

**Relationships:**
- `HAS_IP` → HybridIPAddress
- `HAS_EXTENSION` → HybridExtension
