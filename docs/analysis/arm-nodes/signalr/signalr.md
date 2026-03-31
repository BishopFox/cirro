# SignalR

Represents Azure SignalR Service instances.

**Labels:** `:ArmResource:SignalR`

**Properties:**

- `id` - Resource ID (primary key)
- `allowedOrigins` - Allowed CORS origins
- `disableAadAuth` - Whether AAD authentication is disabled
- `disableLocalAuth` - Whether local authentication is disabled
- `externalIp` - External IP address
- `hostName` - Host name
- `hostNamePrefix` - Host name prefix
- `publicNetworkAccess` - Public network access setting
- `publicPort` - Public port
- `serverPort` - Server port
- `clientCertEnabled` - Whether client certificate is enabled
- `version` - Version

## Examples

```cypher
// Find SignalR instances with public access
MATCH (s:SignalR)
WHERE s.publicNetworkAccess = 'Enabled'
RETURN s.name, s.hostName
```
