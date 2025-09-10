# GraphApplication

Represents Azure AD applications collected from Microsoft Graph.

**Labels:** `:GraphObject:GraphApplication`

**Properties:**

- `displayName` - Application's display name
- `appId` - Application ID (client ID)
- `publisherDomain` - Publisher domain
- `signInAudience` - Sign-in audience configuration
- `identifierUris` - Array of identifier URIs
- `redirectUris` - Combined array of all redirect URIs (web + SPA + public client)
- `publicClientRedirectUris` - Array of public client redirect URIs
- `spaRedirectUris` - Array of single-page application redirect URIs
- `webRedirectUris` - Array of web application redirect URIs
