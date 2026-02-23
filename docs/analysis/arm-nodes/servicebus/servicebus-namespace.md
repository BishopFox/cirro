# ServiceBusNamespace

Represents Azure Service Bus Namespace resources.

**Labels:** `:ArmResource:ServiceBusNamespace`

**Properties:**

- `id` - Resource ID (primary key)
- `createdAt` - Creation timestamp
- `disableLocalAuth` - Whether local authentication is disabled
- `serviceBusEndpoint` - Service Bus endpoint
- `publicNetworkAccess` - Public network access setting

## Examples

```cypher
// Find Service Bus namespaces with local auth enabled
MATCH (sb:ServiceBusNamespace)
WHERE sb.disableLocalAuth = false OR sb.disableLocalAuth IS NULL
RETURN sb.name, sb.serviceBusEndpoint
```
