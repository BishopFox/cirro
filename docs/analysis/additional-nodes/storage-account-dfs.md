# StorageAccountDFS

Represents Data Lake Storage Gen2 filesystems within Azure Storage Accounts.

**Labels:** `:StorageAccountDFS`

**Properties:**

- `id` - DFS filesystem ID (primary key)  
- `name` - Filesystem name

**Relationships:**
- Connected from Storage Account via `HAS_DFS_FILESYSTEM` relationship
- `USED_BY` → Various services that use DFS storage (e.g., Synapse Workspaces)