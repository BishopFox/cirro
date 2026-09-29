# GraphApplication

Represents Entra ID applications collected from Microsoft Graph.

**Labels:** `:GraphObject:GraphApplication`

**Properties:**

- `description` - Application description
- `notes` - Application notes
- `createdDateTime` - Creation timestamp
- `createdByAppId` - Application ID of the creator
- `applicationTemplateId` - Application template ID
- `isAuthorizationServiceEnabled` - Authorization service flag
- `isDeviceOnlyAuthSupported` - Device-only authentication support flag
- `isDisabled` - Whether the application is disabled
- `isFallbackPublicClient` - Fallback public client flag
- `isManagementRestricted` - Management restriction flag
- `nativeAuthenticationApisEnabled` - Native authentication API configuration
- `defaultRedirectUri` - First URI in `web.redirectUris`

- `odataType` - Lowercase `@odata.type`, when returned by Microsoft Graph

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

- **GraphObject** → [`CREATED`](../edges/created.md) → **GraphApplication** - Creator identified by `createdByAppId`

- **GraphObject** → `OWNS` → **GraphApplication** - Owners of the application
- **GraphObject** → `APPROLE` → **GraphApplication** - Objects with app role assignments
- **ClientSecret** → `AUTHENTICATES` → **GraphApplication** - Client secrets for authentication
- **Certificate** → `AUTHENTICATES` → **GraphApplication** - Certificates for authentication

### Outgoing

- **GraphApplication** → [`CREATED`](../edges/created.md) → **GraphServicePrincipal** - Service principals created by the application

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
