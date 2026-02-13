# Developer Notes

## Adding a New Datasource (Example: DataSaaS)

Graph ingestion can support any datasource as long as the input schema is consistent with what the ingest specs expect. The storage/transport format (for example JSON file, SQLite rows, or another source) is an implementation detail.

Use `DataSaaS` as a generic example datasource. The same approach works for any new source.

## Schema-First Principle

Design your datasource around a stable schema contract:

- predictable IDs for merge keys
- consistent field names/types
- repeatable relationship identifiers

When these are stable, the same spec-driven ingestion model works regardless of where the data came from.

## DataSaaS Integration Checklist

### 1) Add constants

Update `src/graph/config/constants.yaml`:

- Add any new labels under `LABELS` (for example `DataSaaSTenant`, `DataSaaSUser`, `DataSaaSAsset`)
- Add any new relationships under `REL` (for example `HAS_ACCOUNT`, `HAS_ASSET`)

These values are rendered into templates through Tera context at load time.

### 2) Add datasource spec files

Create a new folder under `src/graph/config`, for example:

- `src/graph/config/datasaas/`

Add one or more `.tera.yaml` files.

Example spec (`src/graph/config/datasaas/tenants.tera.yaml`):

```yaml
name: "DataSaaS Tenants"
label: "{{ LABELS.DataSaaSTenant }}"
table_name: "datasaas_entities"
properties:
  - "/id"
  - "/name"
  - "/status"
cypher: |
  UNWIND $batch AS row
  MERGE (obj:{{ LABELS.DataSaaSTenant }} {id: toLower(row.id)})
  SET obj += {
    name: row.name,
    status: row.status
  }
  RETURN count(obj) AS count
```

### 3) Add spec Rust types

Add a new module under `src/graph/specs/datasaas/` similar to Azure/Tailscale:

- `mod.rs`
- `types.rs`

Your `types.rs` should derive `Deserialize` and implement `SpecTrait`.

### 4) Register spec config

Update `src/graph/specs/configs.rs`:

- Add a `SpecConfig` for DataSaaS path prefix (for example `datasaas/`)
- Include it in `ALL_SPEC_CONFIGS` in desired ingestion order

### 5) Extend registry and loader

Update `src/graph/specs/factory.rs`:

- Add a `Vec<YourDataSaaSSpecType>` field to `SpecRegistry`
- Load that spec set in `load_all_specs()`
- Add error aggregation consistent with existing loaders

### 6) Add ingest type and dispatcher

Update `src/graph/ingest/ingestor.rs`:

- Add a new `IngestType` enum value (for example `DataSaas`)
- Add a `match` arm in `run()` to call your processing function

### 7) Implement ingestion logic

Add `src/graph/ingest/datasaas.rs` and wire it in `src/graph/ingest/mod.rs`.

- Choose any input adapter that produces the expected schema shape consumed by specs
- Keep parsing/normalization in the adapter layer and keep spec Cypher focused on graph mapping
- Reuse `create_constraints_and_indexes_by_spec` for any non-empty labels

### 8) Expose CLI support

Because `IngestType` is a `clap::ValueEnum`, adding a new enum variant automatically surfaces a new `--type` value in `cirro graph ingest`.

Example:

```bash
cirro graph ingest --type datasaas --file datasaas_export.json
```

### 9) Keep docs aligned

When adding a datasource, update docs in parallel:

- `docs/analysis` node and edge docs for new labels/relationships
- `docs/usage/cirro-graph.md` for new ingest `--type` values
- `docs/architecture.md` if ingestion flow assumptions change

## Practical Guidance

- Start with 1–2 minimal specs and validate graph shape first.
- Keep relationship names specific instead of generic.
- Prefer stable IDs and lowercase normalization for merge keys.
- Use post-processing specs in `src/graph/config/post_processing/` only for cross-cutting cleanup/enrichment.
