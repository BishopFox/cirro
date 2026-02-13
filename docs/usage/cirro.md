# Data Collection

Cirro's collection functionality gathers data from various cloud platforms and services.

## Command Structure

```bash
cirro collect <platform> <auth-method> [options]
```

## Azure Collection

### Authentication Methods

#### Azure CLI

Use your existing Azure CLI authentication:

```bash
cirro collect az azcli [options]
```

**Options:**
- `--tenant-id`: Specific tenant ID to use
- `--subscription-id`: Specific subscription ID to target
- `--mode`: Enumeration mode (`both`, `graph`, `arm`)
- `--cloud`: Azure cloud environment (`public`, `china`, `germany`, `usgov`)
- `--output-path`: Output database file (default: `cirro_output.db`)
- `--debug`: Enable debug logging

#### Client Secret

Authenticate using a service principal with client secret:

```bash
cirro collect az client-secret \
  --client-id <client-id> \
  --client-secret <secret> \
  --tenant-id <tenant-id> \
  [options]
```

#### Client Certificate

Authenticate using a service principal with certificate:

```bash
cirro collect az client-cert \
  --client-id <client-id> \
  --certificate <path-to-cert.pem> \
  --tenant-id <tenant-id> \
  [options]
```

#### Access Token

Use a pre-obtained access token:

```bash
cirro collect az access-token \
  --token <access-token> \
  [options]
```

#### Username/Password

Authenticate with username and password:

```bash
cirro collect az user-pass \
  --upn <user@domain.com> \
  --password <password> \
  [options]
```

### Enumeration Modes

- `both` (default): Collects both Microsoft Graph and ARM data
- `graph`: Only collects Microsoft Graph data (users, groups, applications, etc.)
- `arm`: Only collects Azure Resource Manager data (VMs, networks, storage, etc.)

### Examples

```bash
# Collect everything using Azure CLI
cirro collect az azcli --output-path azure-data.db

# Collect only Graph data for a specific tenant
cirro collect az azcli --tenant-id <tenant-id> --mode graph

# Collect ARM data for US Government cloud
cirro collect az client-secret \
  --client-id <id> \
  --client-secret <secret> \
  --tenant-id <tenant> \
  --cloud usgov \
  --mode arm

# Debug mode
cirro collect az azcli --debug
```

## Tailscale Collection

Collect Tailscale network topology data:

```bash
cirro collect ts <auth-method> [options]
```

**Options:**
- `--output-path`: Output database file (default: `cirro_output.db`)
- `--debug`: Enable debug logging

## Output

Collection commands output platform-specific files (for example SQLite or JSON) that can be ingested into a graph database using `cirro graph ingest`.
 CLI Reference

The `cirro` command-line tool is the primary data collection component designed to gather information from multiple cloud platforms. Currently, it supports Azure environments by collecting data from Azure Resource Manager (ARM) APIs and Microsoft Graph APIs.

## Synopsis

```bash
cirro <function> <platform> <auth-method> [options]
```

## Collection Mode

```bash
cirro collect <platform> <auth-method> [options]
```

## Authentication Modes

### `access-token`

Authenticate using a pre-obtained access token.

```bash
cirro collect az access-token --token <TOKEN> [OPTIONS]
```

**Arguments:**

- `-t, --token <TOKEN>` - Access token for authentication

**Options:**

- `-o, --output-path <FILE>` - Output database file path (default: `cirro_output.db`)
- `--cloud <CLOUD>` - Cloud to enumerate (default: `public`)
- `--debug` - Enable debug output

### `azcli`

Authenticate using Azure CLI credentials.

```bash
cirro collect az azcli [OPTIONS]
```

**Options:**

- `-t, --tenant-id <TENANT_ID>` - Tenant ID (optional)
- `-s, --subscription-id <SUBSCRIPTION_ID>` - Subscription ID (optional)
- `-o, --output-path <FILE>` - Output database file path (default: `cirro_output.db`)
- `-m, --mode <MODE>` - Enumeration mode (default: `both`)
- `--cloud <CLOUD>` - Cloud to enumerate (default: `public`)
- `--debug` - Enable debug output

### `client-secret`

Authenticate using a client secret.

```bash
cirro collect az client-secret --client-id <ID> --client-secret <SECRET> --tenant-id <TENANT> [OPTIONS]
```

**Arguments:**

- `-c, --client-id <CLIENT_ID>` - Client ID
- `-p, --client-secret <CLIENT_SECRET>` - Client secret
- `-t, --tenant-id <TENANT_ID>` - Tenant ID

**Options:**

- `-o, --output-path <FILE>` - Output database file path (default: `cirro_output.db`)
- `-m, --mode <MODE>` - Enumeration mode (default: `both`)
- `--cloud <CLOUD>` - Cloud to enumerate (default: `public`)
- `--debug` - Enable debug output

### `client-cert`

Authenticate using a client certificate.

```bash
cirro collect az client-cert --client-id <ID> --certificate <CERT_PATH> --tenant-id <TENANT> [OPTIONS]
```

**Arguments:**

- `-c, --client-id <CLIENT_ID>` - Client ID
- `-p, --certificate <FILE>` - Path to the client certificate file (PEM format)
- `-t, --tenant-id <TENANT_ID>` - Tenant ID

**Options:**

- `-o, --output-path <FILE>` - Output database file path (default: `cirro_output.db`)
- `-m, --mode <MODE>` - Enumeration mode (default: `both`)
- `--cloud <CLOUD>` - Cloud to enumerate (default: `public`)
- `--debug` - Enable debug output

### `user-pass`

Authenticate using username and password.

```bash
cirro collect az user-pass --upn <UPN> --password <PASSWORD> [OPTIONS]
```

**Arguments:**

- `-u, --upn <UPN>` - The username of the account (UPN format)
- `-p, --password <PASSWORD>` - The password of the account

**Options:**

- `-o, --output-path <FILE>` - Output database file path (default: `cirro_output.db`)
- `-m, --mode <MODE>` - Enumeration mode (default: `both`)
- `--cloud <CLOUD>` - Cloud to enumerate (default: `public`)
- `--debug` - Enable debug output

## Global Options

### Enumeration Mode (`--mode`)

Specifies which APIs to query during Azure data collection.

- `both` (default) - Collect from both ARM and Graph APIs
- `arm` - Collect only from Azure Resource Manager APIs
- `graph` - Collect only from Microsoft Graph APIs

### Cloud Environment (`--cloud`)

Specifies the Azure cloud environment to target.

- `public` (default) - Azure Public Cloud
- `china` - Azure China Cloud
- `germany` - Azure Germany Cloud
- `usgov` - Azure US Government Cloud

### Output Path (`--output-path`)

Specifies the path for the output SQLite database file.

- Default: `cirro_output.db`
- Format: SQLite database file

### Debug Mode (`--debug`)

Enables detailed debug logging for troubleshooting.

- Default: `false`
- Provides verbose output for debugging authentication and API issues

## Enrichment Options

When using the `enrich` command, additional flags are available:

!!! warning "Enrichment Flags"

    Enrichment flags must be placed directly after the enrich command but before the authentication mode, as shown below.

### `--storage-keys`

Gather storage account keys during enrichment.

```bash
cirro enrich --storage-keys azcli 
```

## Examples

### Basic Collection

```bash
# Collect from Azure using CLI authentication
cirro collect az azcli

# Collect with specific Azure tenant and subscription
cirro collect az azcli --tenant-id "12345678-1234-1234-1234-123456789012" --subscription-id "87654321-4321-4321-4321-210987654321"

# Collect only Azure ARM resources
cirro collect az azcli --mode arm

# Collect only Azure Graph data
cirro collect az azcli --mode graph
```

### Authentication Examples

```bash
# Using client secret
cirro collect az client-secret \
  --client-id "app-id-here" \
  --client-secret "secret-here" \
  --tenant-id "tenant-id-here"

# Using client certificate
cirro collect az client-cert \
  --client-id "app-id-here" \
  --certificate "/path/to/cert.pem" \
  --tenant-id "tenant-id-here"

# Using access token
cirro collect az access-token --token "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIs..."
```

### Cloud Environment Examples

```bash
# Collect from Azure US Government
cirro collect az azcli --cloud usgov

# Collect from Azure China
cirro collect az azcli --cloud china
```

### Data Enrichment

```bash
# Enrichment with storage keys
cirro enrich --storage-keys azcli 

# Enrichment with client secret
cirro enrich --storage-keys client-secret \
  --client-id "app-id-here" \
  --client-secret "secret-here" \
  --tenant-id "tenant-id-here" \
```

### Output and Debugging

```bash
# Custom output file
cirro collect az azcli --output-path "/path/to/my-data.db"

# Enable debug logging
cirro collect az azcli --debug

# Combined options
cirro collect az azcli \
  --output-path "/path/to/my-data.db" \
  --mode both \
  --cloud public \
  --debug
```

## Notes

- Ensure you have appropriate permissions for the target cloud environment (currently Azure)
- The access token authentication mode has limited options compared to other methods
- Debug mode provides detailed logging but may expose sensitive information
- The output database file will be created if it doesn't exist
- Some authentication modes require specific cloud provider application configurations (Entra ID for Azure)