# CustomDnsConfig

Represents custom DNS configurations for private endpoints.

**Labels:** `:CustomDnsConfig`

**Properties:**

- `fqdn` - Fully qualified domain name (primary key)
- `ipAddresses` - Array of IP addresses

**Relationships:**
- `HAS_DNS_CONFIG` ← PrivateEndpoint
