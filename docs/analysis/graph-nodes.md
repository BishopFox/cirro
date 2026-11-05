# GraphObject

<h2>GraphObject (Base Label)</h2>

Microsoft Graph directory entities use the base `GraphObject` label. Fully collected entities include an `id`; creator placeholders introduced by [`CREATED`](edges/created.md) can initially have only an `appId`.

**Common Properties:**

- `id` - Unique object identifier (primary key)
- `odataType` - Lowercase `@odata.type`, when present, on users, groups, applications, service principals, devices, and administrative units

Permission scopes extracted from service principals use the standalone [`GraphApplicationScope`](graph-nodes/graph-application-scope.md) label.

## User enrichment

Normal Graph collection retrieves `lastPasswordChangeDateTime` for users and managers for enabled member users. Ingestion stores the timestamp on [`GraphUser`](graph-nodes/graph-user.md) and creates a [`MANAGES`](edges/manages.md) edge from the manager to the user. See [user query examples](graph-nodes/graph-user.md#examples).

## Application relationships

[`GraphApplication`](graph-nodes/graph-application.md) includes creation and template metadata, descriptions, notes, authentication and management flags, and redirect URI properties when the source fields are available.

`createdByAppId` creates a [`CREATED`](edges/created.md) relationship to an application or service principal. Service principal `publishedPermissionScopes` create [`GraphApplicationScope`](graph-nodes/graph-application-scope.md) nodes connected by [`HAS_PUBLISHED_SCOPE`](edges/has-published-scope.md). Published scopes describe the permissions an application exposes; [OAuth consent grants](edges/has-oauth-grant.md) describe permissions granted to a client.

## Agent identity metadata

| Node | Properties |
| --- | --- |
| [`GraphUser`](graph-nodes/graph-user.md) | `agentIdentityBlueprintId`, `identityParentId`, `identityParentType` |
| [`GraphServicePrincipal`](graph-nodes/graph-service-principal.md) | `agentAppId`, `agentIdentityBlueprintId` |

These values remain properties on the existing nodes. Ingestion does not create additional agent identity edges from these IDs. Use `odataType` and the available identity properties to inspect the collected objects.

## Partial collection

[Restricted guest collection](../usage/cirro.md#restricted-guest-collection) follows accessible application relationships without fetching a full directory inventory. Referenced principals can remain `GraphObject` placeholders with only an ID. Queries that require `GraphUser` or `GraphServicePrincipal` labels can omit those placeholders; use `GraphObject` when inspecting all collected references.
