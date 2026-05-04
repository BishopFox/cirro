# SyntexDocumentProcessor

Represents Microsoft Syntex document processor resources, which provide AI-based document processing capabilities integrated with SharePoint Online.

**Labels:** `:ArmResource:SyntexDocumentProcessor`

**Properties:**

- `id` - Resource ID (primary key)
- `spoTenantId` - SharePoint Online tenant ID
- `spoTenantUrl` - SharePoint Online tenant URL

## Examples

```cypher
// Find all Syntex document processors
MATCH (sdp:SyntexDocumentProcessor)
RETURN sdp.id, sdp.spoTenantId, sdp.spoTenantUrl
```
