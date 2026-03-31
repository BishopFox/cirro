# HybridIPAddress

Represents IP addresses for hybrid machines (not an ARM resource itself).

**Labels:** `:HybridIPAddress`

**Properties:**

- `address` - IP address (primary key)
- `ipAddressVersion` - IP address version (IPv4/IPv6)
- `subnet` - Subnet address prefix

**Relationships:**
- `HAS_IP` ← HybridMachine
