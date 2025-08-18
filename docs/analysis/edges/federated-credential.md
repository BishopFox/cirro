# FEDERATED_CREDENTIAL

Represents the relationship between Graph applications and their federated identity credentials.

## Usage

This relationship connects Graph applications to their federated identity credentials:

- **GraphApplication** → `FEDERATED_CREDENTIAL` → **FederatedIdentityCredential** - Applications to their federated credentials

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all applications with federated credentials
MATCH (app:GraphApplication)-[:FEDERATED_CREDENTIAL]->(cred:FederatedIdentityCredential)
RETURN app.displayName, cred.name, cred.issuer, cred.subject
```
