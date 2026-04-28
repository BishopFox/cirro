# IS_ADMIN

Represents an administrative relationship where a principal administers a target resource.

## Usage

This relationship is used to connect principals to resources they administer:

- **GraphServicePrincipal** → `IS_ADMIN` → **AnalysisServiceServer** - Service principal administrator
- **GraphObject** → `IS_ADMIN` → **AnalysisServiceServer** - Object-based administrator reference
- **GraphUser** → `IS_ADMIN` → **AnalysisServiceServer** - User administrator
- **GraphObject** → `IS_ADMIN` → **ManagedHSM** - Initial Managed HSM administrator

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find principals that administer Analysis Services servers
MATCH (p)-[:IS_ADMIN]->(s:AnalysisServiceServer)
RETURN labels(p), p, s.serverFullName

// Find GraphObjects that administer Managed HSM resources
MATCH (go:GraphObject)-[:IS_ADMIN]->(hsm:ManagedHSM)
RETURN go.id, hsm.id, hsm.hsmUri
```