# RedisCache

Represents Azure Cache for Redis instances.

**Labels:** `:ArmResource:RedisCache`

**Properties:**

- `id` - Resource ID (primary key)
- `hostname` - Host name
- `disableAccessKeyAuthentication` - Whether access key authentication is disabled
- `enableNonSslPort` - Whether non-SSL port is enabled
- `minimumTlsVersion` - Minimum TLS version
- `port` - Redis port
- `sslPort` - Redis SSL port
- `redisVersion` - Redis version
- `updateChannel` - Update channel
- `instancePorts` - List of SSL ports for instances

## Examples

```cypher
// Find Redis caches with non-SSL port enabled
MATCH (r:RedisCache)
WHERE r.enableNonSslPort = true
RETURN r.name, r.hostname, r.minimumTlsVersion
```
