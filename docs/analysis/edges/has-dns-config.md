# HAS_DNS_CONFIG

Represents the relationship between private endpoints and their DNS configurations.

## Usage

This relationship connects private endpoints to their custom DNS configurations:

- **PrivateEndpoint** → `HAS_DNS_CONFIG` → **CustomDnsConfig** - Private endpoints to their DNS configurations

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all private endpoints and their DNS configurations
MATCH (pe:PrivateEndpoint)-[:HAS_DNS_CONFIG]->(dns:CustomDnsConfig)
RETURN pe.name, dns.fqdn, dns.ipAddresses
```
