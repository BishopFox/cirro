# HAS_CONFIG

Represents configuration relationships between resources and their configuration objects.

## Usage

This relationship connects resources to their configuration components:

- **NetworkInterface** → `HAS_CONFIG` → **IPConfig** - Network interfaces to their IP configurations
- **BastionHost** → `HAS_CONFIG` → **BastionIPConfig** - Bastion hosts to their IP configurations

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all network interfaces and their IP configurations
MATCH (ni:NetworkInterface)-[:HAS_CONFIG]->(config:IPConfig)
RETURN ni.name, config.privateIPAddress
```
