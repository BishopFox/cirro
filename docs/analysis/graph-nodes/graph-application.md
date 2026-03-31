# GraphApplication

Represents Entra ID applications collected from Microsoft Graph.

**Labels:** `:GraphObject:GraphApplication`

**Properties:**

- `id` - Application object ID (primary key)
- `displayName` - Application's display name
- `appId` - Application ID (client ID)
- `publisherDomain` - Publisher domain
- `signInAudience` - Sign-in audience configuration
- `identifierUris` - Array of identifier URIs
- `redirectUris` - Combined array of all redirect URIs (web + SPA + public client)
- `publicClientRedirectUris` - Array of public client redirect URIs
- `spaRedirectUris` - Array of single-page application redirect URIs
- `webRedirectUris` - Array of web application redirect URIs
- `implicitAccessToken` - Whether implicit grant flow access token issuance is enabled
- `implicitIdToken` - Whether implicit grant flow ID token issuance is enabled

## Relationships

### Incoming

- **GraphObject** → `OWNS` → **GraphApplication** - Owners of the application
- **GraphObject** → `APPROLE` → **GraphApplication** - Objects with app role assignments
- **ClientSecret** → `AUTHENTICATES` → **GraphApplication** - Client secrets for authentication
- **Certificate** → `AUTHENTICATES` → **GraphApplication** - Certificates for authentication

### Outgoing

- **GraphApplication** → `HAS_APPROLE` → **GraphAppRole** - App roles defined by the application
- **GraphApplication** → `FEDERATED_CREDENTIAL` → **FederatedIdentityCredential** - Federated identity credentials

## Examples

```cypher
// Find all multi-tenant applications
MATCH (app:GraphApplication)
WHERE app.signInAudience = "AzureADMultipleOrgs"
RETURN app.displayName, app.appId, app.publisherDomain
```

```cypher
// Find applications and their owners
MATCH (owner:GraphObject)-[:OWNS]->(app:GraphApplication)
RETURN app.displayName, collect(owner.displayName) AS owners
```

```cypher
// Find applications with federated credentials
MATCH (app:GraphApplication)-[:FEDERATED_CREDENTIAL]->(cred:FederatedIdentityCredential)
RETURN app.displayName, cred.issuer, cred.subject
```
