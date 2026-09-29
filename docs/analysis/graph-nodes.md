# GraphObject

<h2>GraphObject (Base Label)</h2>

Microsoft Graph directory entities use the base `GraphObject` label. Fully collected entities include an `id`; creator placeholders introduced by [`CREATED`](edges/created.md) can initially have only an `appId`.

**Common Properties:**

- `id` - Unique object identifier (primary key)
- `odataType` - Lowercase `@odata.type`, when present, on users, groups, applications, service principals, devices, and administrative units

Permission scopes extracted from service principals use the standalone [`GraphApplicationScope`](graph-nodes/graph-application-scope.md) label.
