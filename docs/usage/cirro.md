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

## Notes

- Ensure you have appropriate permissions for your target environment.
- Debug logging can include sensitive values; handle output carefully.