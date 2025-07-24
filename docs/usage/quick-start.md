# Setup

!!! tip "Prerequisites"

    - Valid Azure credentials
    - Running Neo4j or Memgraph instance
    - Appropriate permissions for target Azure environment

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

## Collect Data

```bash
# Using Azure CLI authentication
cirro collect azcli

# Using client secret
cirro collect client-secret \
  --client-id <CLIENT_ID> \
  --client-secret <CLIENT_SECRET> \
  --tenant-id <TENANT_ID>
```

## Ingest into Graph Database

Both Neo4j and Memgraph are supported as graph database backends. Set up your preferred database before ingesting data. There are two docker-compose files in the [tools](/tools/) directory to assist with containerized databases. `cirro-ingest` defaults to Neo4j configurations but you might consider using Memgraph for faster ingestion and performance.

```bash
# For Neo4j
docker-compose up

# For Memgraph
docker-compose -f docker-compose.mg.yml up
```

```bash
# For Neo4j
cirro-ingest --file cirro_output.db \
  --graph-type neo4j \
  --user neo4j \
  --password password

# For Memgraph  
cirro-ingest --file cirro_output.db \
  --graph-type memgraph \
  --user cirro \
  --password cirro
```

## Next Steps

After completing the quick start:

1. **Explore Your Data**: Use your graph database's query interface to explore the collected data
2. **Learn Query Patterns**: Check out our [query examples](examples.md) for common analysis patterns
3. **Set Up Visualization**: Configure dashboards and visualizations for your specific use case
4. **Advanced Features**: Explore data enrichment and custom collection options

For detailed installation and configuration instructions, see the [Installation Guide](installation.md).
