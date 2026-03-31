# IS_ROLE_ELIGIBLE

Represents Privileged Identity Management (PIM) eligibility for directory role assignments.

**Direction:** `(principal)-[:IS_ROLE_ELIGIBLE]->(role)`

**Description:** Indicates that a principal is eligible (but not necessarily active) for a role.

**Common Patterns:**

- Users eligible for privileged roles such as Global Administrator
- Groups eligible for delegated administrative roles
- Service principals eligible for elevated role templates

**Properties:**

- `createdDateTime` - When the eligibility relationship was created
- `directoryScopeId` - Directory scope for the eligibility
- `memberType` - Membership type for the assignment
- `modifiedDateTime` - Last modification time
- `expirationType` - Expiration mode (`noExpiration`, `afterDateTime`, etc.)
- `startDateTime` - Eligibility start time
- `endDateTime` - Eligibility end time when present
- `recurrence` - Recurrence information for recurring eligibility schedules

## Query Examples

```cypher
// Find all principals eligible for privileged roles
MATCH path=(p)-[rel:IS_ROLE_ELIGIBLE]->(r:GraphRole)
RETURN path

// Find eligible roles for a specific principal
MATCH path=(p:GraphObject {id: 'principal-id'})-[rel:IS_ROLE_ELIGIBLE]->(r:GraphRole)
RETURN path

// Find expiring eligibilities
MATCH (p)-[rel:IS_ROLE_ELIGIBLE]->(r:GraphRole)
WHERE rel.endDateTime IS NOT NULL
RETURN p, r, rel.endDateTime
ORDER BY rel.endDateTime ASC
```