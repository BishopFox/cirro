<p align='center'><img src='docs/logo.png' alt='logo' height="400"/><br>
</p>

Cirro is a tool that enables security researchers and penetration testers to collect, analyze, and visualize Azure cloud environments and Entra ID relationships through graph databases.

You can check out the [Documentation](https://crispy-adventure-qre5p1k.pages.github.io/) for more info.

## Features

- **Multi-source Data Collection**: Gather data from both Azure Resource Manager (ARM) APIs and Microsoft Graph APIs
- **Flexible Authentication**: Support for multiple authentication methods including Azure CLI, access tokens, client secrets, and client certificates
- **Graph Database Support**: Compatible with both **Neo4j** and **Memgraph** graph databases
- **Data Enrichment**: Additional data collection capabilities for storage accounts, containers, and blob analysis
- **Multi-cloud Support**: Works with Azure Public, China, Germany, and US Government clouds
- **Cross-platform**: Available for Windows, macOS, and Linux

## Architecture

Cirro consists of two main components:

- **`cirro`**: The primary data collection tool that gathers information from Azure and Microsoft Graph APIs
- **`cirro-ingest`**: The ingestion tool that loads collected data into graph databases (Neo4j or Memgraph)

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [releases page](https://github.com/bishopfox/cirro/releases).

### Building from Source

Requires Rust 1.70 or later:

```bash
git clone https://github.com/bishopfox/cirro.git
cd cirro
cargo build --release
```

Binaries will be available in `target/release/`.

## Quick Start

### 1. Data Collection

Collect Azure and Entra ID data using one of the supported authentication methods:

#### Azure CLI Authentication
Uses existing Azure CLI authentication. Ensure you're logged in with `az login` before running.

```bash
# Collect from current Azure CLI context
cirro collect azcli
```

#### Client Secret Authentication
Authenticate using an Azure AD application's client ID and secret.
```bash
cirro collect client-secret \
  --client-id <CLIENT_ID> \
  --client-secret <CLIENT_SECRET> \
  --tenant-id <TENANT_ID>
```

#### Client Certificate Authentication
Authenticate using an Azure AD application's client ID and certificate (PEM format).
```bash
cirro collect client-cert \
  --client-id <CLIENT_ID> \
  --certificate <CERT_PATH> \
  --tenant-id <TENANT_ID>
```

#### Access Token Authentication
Use a pre-obtained access token.
```bash
cirro collect access-token --token <ACCESS_TOKEN>
```

### 2. Data Ingestion

Both Neo4j and Memgraph are supported as graph database backends. Set up your preferred database before ingesting data. There are two docker-compose files in the [tools](/tools/) directory to assist with containerized databases. `cirro-ingest` defaults to Neo4j configurations but you might consider using Memgraph for faster ingestion and performance.

After collecting data, ingest it into your graph database:

#### Neo4j
```bash
cirro-ingest --file cirro_output.db
```

#### Memgraph
```bash
cirro-ingest --file cirro_output.db \
  --graph-type memgraph \
  --user cirro \
  --password cirro
```

## Collection Modes

- **`both`** (default): Collect from both ARM and Graph APIs
- **`arm`**: Collect only from Azure Resource Manager APIs
- **`graph`**: Collect only from Microsoft Graph APIs

## Data Enrichment

Cirro supports additional data enrichment capabilities that will add additional nodes to the graph:

```bash
# Collect storage account keys
cirro enrich azcli --storage-keys
```

## Cloud Support

Cirro supports multiple Azure cloud environments:

- **`public`** (default): Azure Public Cloud
- **`china`**: Azure China Cloud
- **`germany`**: Azure Germany Cloud
- **`usgov`**: Azure US Government Cloud

```bash
cirro collect azcli --cloud usgov
```

## Configuration Options

### Global Options

- `--output-path`: Specify the output SQLite database file (default: `cirro_output.db`)
- `--debug`: Enable debug logging for troubleshooting
- `--mode`: Set collection mode (`both`, `arm`, `graph`)
- `--cloud`: Specify Azure cloud environment

### Ingestion Options

- `--file`: Path to the Cirro SQLite database
- `--graph-type`: Database type (`neo4j` or `memgraph`)
- `--server`: Database server URL (bolt://, neo4j://, etc.)
- `--user`: Database username
- `--password`: Database password
- `--db-name`: Database name (optional, defaults based on graph type)


## Troubleshooting

### Common Issues

1. **Authentication Failures**: Ensure your credentials have appropriate permissions
2. **HTTP 429 Errors**: This can occur in large Azure environments. Retries are implemented in code but please report if you are still getting these errors.
3. **Access Token Authentication**: Ensure your access token has the appropriate audience for Microsoft Graph or Azure Resource Manager.
   
### Debug Mode

Enable debug logging for detailed information:

```bash
cirro collect azcli --debug
cirro-ingest --file cirro_output.db --debug
```

---

**Note**: Cirro is designed for authorized security testing and research. Ensure you have proper permissions before running against any Azure environment.
