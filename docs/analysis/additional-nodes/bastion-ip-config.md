# BastionIPConfig

Represents IP configurations for Bastion hosts.

**Labels:** `:BastionIPConfig`

**Properties:**

- `id` - Bastion IP config ID (primary key)
- `privateIPAllocationMethod` - Private IP allocation method

**Relationships:**
- `HAS_CONFIG` ← BastionHost
- `HAS_IP` → PublicIPAddress
- `CONTAINS` ← Subnet
