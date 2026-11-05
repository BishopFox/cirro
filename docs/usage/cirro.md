# Data Collection

Cirro's collection functionality gathers data from various cloud platforms and services.

## Command Structure

```bash
cirro collect <platform> <auth-method> [options]
```

## Platforms and Auth Methods

### Azure

```bash
cirro collect az azcli [options]
cirro collect az client-secret --client-id <ID> --client-secret <SECRET> --tenant-id <TENANT> [options]
cirro collect az client-cert --client-id <ID> --certificate <CERT_PATH> --tenant-id <TENANT> [options]
cirro collect az access-token --token <TOKEN> [options]
cirro collect az user-pass --upn <UPN> --password <PASSWORD> [options]
```

### Tailscale

```bash
cirro collect ts <auth-method> [options]
```

## Examples

```bash
# Azure via Azure CLI
cirro collect az azcli --tenant-id <tenant-id> --mode both

# Azure via client secret
cirro collect az client-secret \
  --client-id <id> \
  --client-secret <secret> \
  --tenant-id <tenant> \
  --cloud usgov

# Tailscale collection
cirro collect ts <auth-method> --output-path cirro_ts_socket.json

# Debug mode
cirro collect az azcli --debug
```

## Microsoft Graph User Enrichment

User collection fetches `lastPasswordChangeDateTime` with a per-user `$select` request and merges it into the collected user record when returned. Manager lookups run only for enabled member users (`accountEnabled: true`, `userType: Member`). The collected `manager` value is the manager's object ID; graph ingestion creates a `MANAGES` edge from the manager to the user.

An unassigned manager (HTTP 404) is skipped. Other failed enrichment requests can leave properties or relationships absent; use debug logs to investigate incomplete data.

Graph enrichment uses batches of up to 20 requests. Individual requests returning HTTP 429 or 5xx are retried up to five times, honoring `Retry-After` when supplied. Throttled responses without that header use a 25-second fallback delay.

## Restricted Guest Collection

Use `--restricted-guest` when collecting Microsoft Graph data as a restricted Entra ID guest. This replaces normal Graph enumeration with traversal from well-known Microsoft application IDs. It looks up service principals by application ID, follows app-role assignments, OAuth permission grants and service principal owners, and probes accessible backing applications.

```bash
# Start from the built-in well-known application IDs
cirro collect az azcli --tenant-id <tenant-id> --mode graph --restricted-guest

# Replace the default seeds with specific application IDs
cirro collect az azcli --tenant-id <tenant-id> --mode graph --restricted-guest \
  --restricted-appids 00000003-0000-0000-c000-000000000000,00000002-0000-0000-c000-000000000000

# Start from the full embedded Microsoft application catalog
cirro collect az azcli --tenant-id <tenant-id> --mode graph \
  --restricted-guest --restricted-guest-msapps
```

`--restricted-appids` accepts comma-separated, space-separated or repeated values. `--restricted-guest-msapps` uses a catalog of 1,753 Microsoft applications and prompts for confirmation before collection. Both options require `--restricted-guest` and cannot be combined with each other.

Restricted collection writes to the existing `servicePrincipals`, `applications` and `oauth2PermissionGrants` tables for normal graph ingestion. Unreadable or missing objects and relationships (HTTP 403/404) are skipped while retaining data already collected. Results may therefore be incomplete; a forbidden lookup does not establish that an application is absent.

Normal Graph selection flags and saved Graph pagination state do not control this traversal. Each restricted run starts from its seeds. ARM collection continues normally in `--mode both`; restricted guest options have no effect in `--mode arm`.

## Notes

- Ensure you have appropriate permissions for your target environment.
- Debug logging can include sensitive values; handle output carefully.
