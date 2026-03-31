# FederatedIdentityCredential

Represents federated identity credentials for applications.

**Labels:** `:FederatedIdentityCredential`

**Properties:**

- `id` - Credential ID (primary key)
- `subject` - Subject identifier
- `audiences` - Array of audiences
- `issuer` - Token issuer
- `name` - Credential name
- `description` - Credential description

**Relationships:**
- `AUTHENTICATES` → GraphApplication
