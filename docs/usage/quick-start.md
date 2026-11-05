# Setup

!!! tip "Prerequisites"

    - Valid cloud platform credentials (currently Azure)
    - Running Neo4j instance
    - Appropriate permissions for target cloud environment

!!! warning

    - Neo4j requires [APOC plugin](https://neo4j.com/docs/apoc/current/)
  
## Install Cirro

=== "Shell Script (Unix/Linux/macOS)"

    ```bash
    curl -sSL https://github.com/bishopfox/cirro/releases/latest/download/cirro-installer.sh | sh
    ```

=== "PowerShell (Windows)"

    ```powershell
    irm https://github.com/bishopfox/cirro/releases/latest/download/cirro-installer.ps1 | iex
    ```

=== "Manual Download"

    Download pre-built binaries from the [releases page](https://github.com/bishopfox/cirro/releases).

## Collect Data (Azure)

```bash
# Using Azure CLI authentication
cirro collect az azcli

# Using client secret
cirro collect az client-secret \
  --client-id <CLIENT_ID> \
  --client-secret <CLIENT_SECRET> \
  --tenant-id <TENANT_ID>
```

### Collect as a restricted guest

```bash
cirro collect az azcli --tenant-id <TENANT_ID> --mode graph \
  --restricted-guest --output-path cirro_restricted.db
```

See [restricted guest options](cirro.md#restricted-guest-collection) to specify application IDs or use the Microsoft application catalog. Ingest `cirro_restricted.db` with the command below, replacing the input filename.

## Ingest into Graph Database

Neo4j is supported as the graph database backend. Set up your database before ingesting cloud data. There is a docker-compose file in the [tools](https://github.com/bishopfox/cirro/tools) directory to assist with containerized database setup.

```bash
# For Neo4j
docker-compose up
```

```bash
# For Neo4j
cirro graph ingest --type az --file cirro_output.db \
    --server bolt://localhost:7687 \
  --user neo4j \
  --password password
```

## Next Steps

After completing the quick start:

1. **Explore Your Data**: Use your graph database's query interface to explore the collected data
2. **Learn Query Patterns**: Explore [user and manager queries](../analysis/graph-nodes/graph-user.md#examples) and [service principal relationships](../analysis/graph-nodes/graph-service-principal.md#examples)
3. **Set Up Visualization**: Configure dashboards and visualizations for your specific use case
4. **Advanced Features**: Explore [collection options](cirro.md) and [application metadata](../analysis/graph-nodes/graph-application.md)
