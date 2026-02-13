# Graph Operations

Cirro's graph functionality manages data ingestion into graph databases and data export operations.

## Command Structure

```bash
cirro graph <command> [options]
```

## Data Ingestion

Ingest collected data into Neo4j or Memgraph:

```bash
cirro graph ingest --type <platform> --file <data-file> [options]
```

### Options

- `--type`, `-t`: Platform type (`az` for Azure, `ts` for Tailscale)
- `--file`, `-f`: Path to the collected data file for the selected platform
- `--server`, `-s`: Database server URL (default: `bolt://localhost:7687`)
- `--user`, `-u`: Database username (default: `neo4j`)
- `--password`, `-p`: Database password (default: `password`)
- `--db-name`, `-d`: Database name (default: `neo4j`)
- `--debug`: Enable debug logging

### Supported Database Schemes

- `bolt://` - Unencrypted connection
- `bolt+s://` - Encrypted connection with full certificate validation
- `bolt+ssc://` - Encrypted connection with self-signed certificates
- `neo4j://` - Neo4j routing protocol
- `neo4j+s://` - Encrypted Neo4j routing
- `neo4j+ssc://` - Neo4j routing with self-signed certificates

### Examples

```bash
# Ingest Azure data with defaults
cirro graph ingest --type az --file cirro_output.db

# Ingest to custom database
cirro graph ingest \
  --type az \
  --file cirro_output.db \
  --server bolt://graph-server:7687 \
  --user admin \
  --password secretpass \
  --db-name azure-prod

# Ingest Tailscale data with debug logging
cirro graph ingest --type ts --file cirro_ts_socket.json --debug

# Use encrypted connection
cirro graph ingest \
  --type az \
  --file cirro_output.db \
  --server bolt+s://secure-server:7687
```

## Data Export

Export graph data to various formats:

```bash
cirro graph export --format <format> [options]
```

### Options

- `--format`, `-f`: Export format (`opengraph`)
- `--output`, `-o`: Output file path (default: `cirro_export`)
- `--server`, `-s`: Database server URL (default: `bolt://localhost:7687`)
- `--user`, `-u`: Database username (default: `neo4j`)
- `--password`, `-p`: Database password (default: `password`)
- `--db-name`, `-d`: Database name (default: `neo4j`)
- `--debug`: Enable debug logging

### Examples

```bash
# Export to OpenGraph format
cirro graph export --format opengraph --output my-export.json

# Export from custom database
cirro graph export \
  --format opengraph \
  --output export.json \
  --server bolt://graph-server:7687 \
  --user admin \
  --password secretpass
```

## Database Setup

### Using Docker Compose

Cirro includes a docker-compose file for easy database setup:

```bash
cd tools
docker-compose up -d
```

This starts Neo4j with:
- Web UI at `http://localhost:7474`
- Bolt protocol at `bolt://localhost:7687`
- Default credentials: `neo4j/password`

## Data Model

See the [Analysis](../analysis/cirro-graph-analysis.md) section for details on the graph schema, nodes, and relationships created during ingestion.
## CLI Reference

The `cirro graph ingest` command loads data collected by `cirro collect` into a Neo4j graph database. It currently supports platform data collected from Azure and Tailscale.

!!! warning "Security Notice"

    - **Passwords**: Avoid using default passwords in production
    - **TLS**: Use encrypted connections (`bolt+s://`, `neo4j+s://`) for remote servers
    - **Credentials**: Store credentials securely (environment variables, credential managers)
    - **Network**: Ensure database servers are properly secured and firewalled

## Synopsis

The `cirro graph ingest` command performs the following operations:

1. **Database Connection** - Establishes connection to the target graph database
2. **Schema Setup** - Creates necessary constraints and indexes
3. **Data Transformation** - Converts collected platform data to graph format
4. **Node Creation** - Creates nodes for all collected entities
5. **Relationship Creation** - Establishes relationships between entities
6. **Index Optimization** - Optimizes database performance

```bash
cirro graph ingest --type <platform> --file <data-file> [OPTIONS]
```

## Required Arguments

`--type` / `-t`

Platform type for the input data.

```bash
cirro graph ingest --type az --file cirro_output.db
```

**Values:** `az`, `ts`

`--file` / `-f`

Path to the collected data file created by `cirro collect` for the selected platform.

```bash
cirro graph ingest --type az --file cirro_output.db
```

**Format:** platform-specific data file path  
**Example:** `cirro_output.db`, `cirro_ts_socket.json`

## Options

`--server` / `-s`

Database server connection string with supported URI schemes.

```bash
cirro graph ingest --type az --file cirro_output.db --server bolt://localhost:7687
```

**Supported Schemes:**

- `bolt://` - Plain Bolt protocol
- `bolt+s://` - Bolt with TLS encryption
- `bolt+ssc://` - Bolt with self-signed certificate
- `neo4j://` - Neo4j routing protocol
- `neo4j+s://` - Neo4j routing with TLS encryption
- `neo4j+ssc://` - Neo4j routing with self-signed certificate

**Default:** `bolt://localhost:7687`

`--user` / `-u`

Database username for authentication.

```bash
cirro graph ingest --type az --file cirro_output.db --user myuser
```

**Default:** `neo4j`

`--password` / `-p`

Database password for authentication.

```bash
cirro graph ingest --type az --file cirro_output.db --password mypassword
```

**Default:** `password`

`--db-name` / `-d`

Database name to connect to (optional).

```bash
cirro graph ingest --type az --file cirro_output.db --db-name mydatabase
```

**Default Behavior:**

- For Neo4j: defaults to `neo4j`

`--debug`

Enable debug logging for troubleshooting.

```bash
cirro graph ingest --type az --file cirro_output.db --debug
```

**Default:** `false`

## Examples

### Basic Usage

```bash
# Basic ingestion to Neo4j with defaults
cirro graph ingest --type az --file cirro_output.db
```

### Neo4j Examples

```bash
# Neo4j with custom server
cirro graph ingest \
  --type az \
  --file cirro_output.db \
  --server bolt://neo4j.example.com:7687 \
  --user neo4j \
  --password mypassword

# Neo4j with TLS encryption
cirro graph ingest \
  --type az \
  --file cirro_output.db \
  --server bolt+s://neo4j.example.com:7687 \
  --user neo4j \
  --password mypassword

# Neo4j cluster with routing
cirro graph ingest \
  --type az \
  --file cirro_output.db \
  --server neo4j://neo4j-cluster.example.com:7687 \
  --user neo4j \
  --password mypassword

# Neo4j with custom database name
cirro graph ingest \
  --type az \
  --file cirro_output.db \
  --db-name analytics \
  --user neo4j \
  --password mypassword
```



### Debug and Troubleshooting

```bash
# Enable debug logging
cirro graph ingest --type az --file cirro_output.db --debug

# Full example with all options
cirro graph ingest \
  --type az \
  --file /path/to/cirro_output.db \
  --server bolt+s://neo4j.example.com:7687 \
  --user myuser \
  --password mypassword \
  --db-name mydb \
  --debug
```

## Connection Examples

### Local Development

```bash
# Local Neo4j (default)
cirro graph ingest --type az --file cirro_output.db
```

### Docker Containers

```bash
# Neo4j in Docker
cirro graph ingest \
  --type az \
  --file cirro_output.db \
  --server bolt://localhost:7687 \
  --user neo4j \
  --password password
```

### Cloud Instances

```bash
# Neo4j AuraDB (cloud)
cirro graph ingest \
  --type az \
  --file cirro_output.db \
  --server neo4j+s://xxxxxxxx.databases.neo4j.io:7687 \
  --user neo4j \
  --password your-aura-password

# Self-hosted with TLS
cirro graph ingest \
  --type az \
  --file cirro_output.db \
  --server bolt+s://your-server.com:7687 \
  --user your-username \
  --password your-password
```

