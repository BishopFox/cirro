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
- `--file`, `-f`: Path to the SQLite database file from collection
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
cirro graph ingest --type az --file azure-data.db

# Ingest to custom database
cirro graph ingest \
  --type az \
  --file azure-data.db \
  --server bolt://graph-server:7687 \
  --user admin \
  --password secretpass \
  --db-name azure-prod

# Ingest Tailscale data with debug logging
cirro graph ingest --type ts --file tailscale.db --debug

# Use encrypted connection
cirro graph ingest \
  --type az \
  --file azure-data.db \
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
 CLI Reference

The `cirro-graph` command-line tool is responsible for loading data collected by `cirro` from various cloud platforms into Neo4j graph database. Currently supports data collected from Azure environments.

!!! warning "Security Notice"

    - **Passwords**: Avoid using default passwords in production
    - **TLS**: Use encrypted connections (`bolt+s://`, `neo4j+s://`) for remote servers
    - **Credentials**: Store credentials securely (environment variables, credential managers)
    - **Network**: Ensure database servers are properly secured and firewalled

## Synopsis

The `cirro-graph` tool performs the following operations:

1. **Database Connection** - Establishes connection to the target graph database
2. **Schema Setup** - Creates necessary constraints and indexes
3. **Data Transformation** - Converts SQLite platform data to graph format
4. **Node Creation** - Creates nodes for all collected entities
5. **Relationship Creation** - Establishes relationships between entities
6. **Index Optimization** - Optimizes database performance

```bash
cirro-graph --file <DATABASE_FILE> [OPTIONS]
```

## Required Arguments

`--file` / `-f`

Path to the Cirro results database file (SQLite) created by the `cirro` tool containing cloud platform data.

```bash
cirro-graph --file cirro_output.db
```

**Format:** SQLite database file path  
**Example:** `cirro_output.db`, `/path/to/results.db`

## Options

`--graph-type` / `-g`

Specifies the target graph database type.

```bash
cirro-graph --file cirro_output.db --graph-type neo4j
```

**Values:**

- `neo4j` (default) - Neo4j graph database

**Default:** `neo4j`

`--server` / `-s`

Database server connection string with supported URI schemes.

```bash
cirro-graph --file cirro_output.db --server bolt://localhost:7687
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
cirro-graph --file cirro_output.db --user myuser
```

**Default:** `neo4j`

`--password` / `-p`

Database password for authentication.

```bash
cirro-graph --file cirro_output.db --password mypassword
```

**Default:** `password`

`--db-name` / `-d`

Database name to connect to (optional).

```bash
cirro-graph --file cirro_output.db --db-name mydatabase
```

**Default Behavior:**

- For Neo4j: defaults to `neo4j`

`--debug`

Enable debug logging for troubleshooting.

```bash
cirro-graph --file cirro_output.db --debug
```

**Default:** `false`

## Examples

### Basic Usage

```bash
# Basic ingestion to Neo4j with defaults
cirro-graph --file cirro_output.db
```

### Neo4j Examples

```bash
# Neo4j with custom server
cirro-graph \
  --file cirro_output.db \
  --graph-type neo4j \
  --server bolt://neo4j.example.com:7687 \
  --user neo4j \
  --password mypassword

# Neo4j with TLS encryption
cirro-graph \
  --file cirro_output.db \
  --server bolt+s://neo4j.example.com:7687 \
  --user neo4j \
  --password mypassword

# Neo4j cluster with routing
cirro-graph \
  --file cirro_output.db \
  --server neo4j://neo4j-cluster.example.com:7687 \
  --user neo4j \
  --password mypassword

# Neo4j with custom database name
cirro-graph \
  --file cirro_output.db \
  --db-name analytics \
  --user neo4j \
  --password mypassword
```



### Debug and Troubleshooting

```bash
# Enable debug logging
cirro-graph --file cirro_output.db --debug

# Full example with all options
cirro-graph \
  --file /path/to/cirro_output.db \
  --graph-type neo4j \
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
cirro-graph --file cirro_output.db
```

### Docker Containers

```bash
# Neo4j in Docker
cirro-graph \
  --file cirro_output.db \
  --server bolt://localhost:7687 \
  --user neo4j \
  --password password
```

### Cloud Instances

```bash
# Neo4j AuraDB (cloud)
cirro-graph \
  --file cirro_output.db \
  --server neo4j+s://xxxxxxxx.databases.neo4j.io:7687 \
  --user neo4j \
  --password your-aura-password

# Self-hosted with TLS
cirro-graph \
  --file cirro_output.db \
  --server bolt+s://your-server.com:7687 \
  --user your-username \
  --password your-password
```

