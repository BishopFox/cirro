# Post-Processing Steps

Post-processing steps run after all resources and graph objects have been ingested. They deduplicate nodes, normalize data, and enrich relationships.

## Steps

| Step | Priority | Description |
|------|----------|-------------|
| [Deduplicate Nodes](post-processing/deduplicate-nodes.md) | 0 | Merge duplicate nodes by ID |
| [Deduplicate Relationships](post-processing/deduplicate-relationships.md) | 2 | Merge duplicate relationships |
| [Normalize IDs](post-processing/normalize-ids.md) | 0 | Lowercase all node IDs |
| [Remove MemberOf GraphRole Relations](post-processing/remove-memberof-graphrole.md) | 0 | Remove redundant MEMBER_OF → GraphRole |
| [Set AppRole Properties](post-processing/set-approle-properties.md) | 0 | Enrich ASSIGNED_APPROLE relationships |
| [Set ARM Eligible Properties](post-processing/set-arm-eligible-properties.md) | 0 | Enrich IS_RBAC_ELIGIBLE relationships |
| [Set HasScopedRole Properties](post-processing/set-hasscopedrole-properties.md) | 0 | Enrich HAS_SCOPED_ROLE relationships |
| [Deduplicate Container Registry](post-processing/deduplicate-container-registry.md) | 1000 | Merge container registries by login server |
| [Deduplicate Key Vault by URI](post-processing/deduplicate-keyvault-by-uri.md) | 1000 | Merge Key Vault nodes by vault URI |
| [Deduplicate Graph Applications](post-processing/deduplicate-graph-applications.md) | 1000 | Merge graph applications by app ID |
| [Deduplicate Graph Roles](post-processing/deduplicate-graph-roles.md) | 1000 | Merge graph roles by role template ID |
| [Deduplicate Service Principals](post-processing/deduplicate-service-principals.md) | 1000 | Merge service principals by app ID |
| [Deduplicate Storage](post-processing/deduplicate-storage.md) | 1000 | Merge storage accounts by name |
| [Fix DFS Without ID](post-processing/fix-dfs-without-id.md) | 1001 | Fix DFS filesystem nodes missing IDs |
| [Link TSUser to GraphUser](post-processing/link-tsuser-to-graphuser.md) | 1000 | Link Threat Stack users to Graph users |
