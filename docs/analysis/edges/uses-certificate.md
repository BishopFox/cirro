# USES_CERTIFICATE

Represents Web App usage of a certificate resource.

**Direction:** `(webSite)-[:USES_CERTIFICATE]->(webCertificate)`

**Description:** Created from `hostNameSslStates` on `WebSite` resources for SNI-enabled bindings with certificate references.

**Properties:** None

## Query Examples

```cypher
// Find sites and certificates they use
MATCH (site:WebSite)-[:USES_CERTIFICATE]->(cert:WebCertificate)
RETURN site.name, cert.name, cert.expirationDate
```
