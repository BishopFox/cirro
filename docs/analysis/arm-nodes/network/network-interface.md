# NetworkInterface

Represents network interfaces attached to VMs.

**Labels:** `:ArmResource:NetworkInterface`

**Properties:**

- `id` - NIC resource ID (primary key)
- `macAddress` - MAC address
- `allowPort25Out` - Whether port 25 outbound is allowed
- `appliedDnsServers` - Applied DNS servers
- `dnsServers` - Configured DNS servers

**Relationships:**
- `HAS_CONFIG` → IPConfig
- `HAS_NSG` → NSG (Network Security Group)
- `HAS_NIC` ← VirtualMachine
