# IPConfig

Represents IP configurations for network interfaces (not an ARM resource itself).

**Labels:** `:IPConfig`

**Properties:**

- `id` - IP config ID (primary key)
- `name` - Configuration name
- `type` - Configuration type
- `privateIPAddress` - Private IP address
- `privateIPAddressVersion` - Private IP version
- `privateIPAllocationMethod` - Private IP allocation method

**Relationships:**
- `HAS_CONFIG` ← NetworkInterface
- `CONTAINS` ← Subnet
- `HAS_IP` → PublicIPAddress
