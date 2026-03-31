# PublicIPAddress

Represents public IP addresses.

**Labels:** `:ArmResource:PublicIPAddress`

**Properties:**

- `id` - Public IP resource ID (primary key)
- `idleTimeoutInMinutes` - Idle timeout setting
- `ipAddress` - The actual IP address
- `publicIPAddressVersion` - IP version (IPv4/IPv6)
- `publicIPAllocationMethod` - Allocation method (Static/Dynamic)

**Relationships:**
- `HAS_IP` ← IPConfig
