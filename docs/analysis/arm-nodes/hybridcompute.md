# Hybrid Compute Resources

Azure Arc-enabled machines and hybrid compute resources.

## HybridMachine

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

## HybridIPAddress

Represents IP addresses for hybrid machines (not an ARM resource itself).

**Labels:** `:HybridIPAddress`

**Properties:**

- `address` - IP address (primary key)
- `ipAddressVersion` - IP address version (IPv4/IPv6)
- `subnet` - Subnet address prefix

**Relationships:**
- `HAS_IP` ← HybridMachine

## HybridExtension

Represents extensions installed on hybrid machines (not an ARM resource itself).

**Labels:** `:HybridExtension`

**Properties:**

- `id` - Extension ID (primary key)
- `name` - Extension name
- `type` - Extension type
- `location` - Extension location
- `typeHandlerVersion` - Type handler version
- `autoUpgradeMinorVersion` - Auto upgrade minor version
- `enableAutomaticUpgrade` - Enable automatic upgrade
- `statusMessage` - Status message
- `provisioningState` - Provisioning state

**Relationships:**
- `HAS_EXTENSION` ← HybridMachine
