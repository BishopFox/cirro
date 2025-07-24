# VirtualMachine

Represents Azure virtual machines.

**Labels:** `:ArmResource:VirtualMachine`

**Properties:**

- `id` - VM resource ID (primary key)
- `vmSize` - VM size/SKU
- `adminUsername` - Administrator username
- `allowExtensionOperations` - Whether extensions are allowed
- `computerName` - Computer name
- `vmId` - Virtual machine ID
- `os` - Operating system type
- `image` - VM image offer
- `imageVersion` - VM image version

**Relationships:**
- `HAS_NIC` → NetworkInterface
- `HAS_EXTENSION` → VMExtension
- `HAS_DISK` ← Disk (via managedBy)
