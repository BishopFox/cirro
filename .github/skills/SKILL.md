# Skill: Sync Cirro Config Documentation

## Description

This skill synchronizes documentation in `docs/analysis/` and `mkdocs.yml` with the Cirro configuration YAML files in `src/graph/config/` and the constants defined in `src/graph/config/constants.yaml`. Use this skill when Cirro configs or constants are added or changed and documentation needs to be updated.

## Trigger Phrases

- "sync cirro docs"
- "update node and edge documentation"
- "ensure docs match cirro config"
- "add missing cirro documentation"

## Steps

### 1. Read constants and identify all labels and relations

Read `src/graph/config/constants.yaml` to get the full list of:
- **REL**: All relationship/edge type constants
- **LABELS**: All node label constants (ARM resources, Graph objects, specialized nodes, Tailscale nodes)

### 2. Inventory existing documentation

Collect all existing doc files:
- `docs/analysis/graph-nodes/*.md` — Entra ID / Microsoft Graph node docs
- `docs/analysis/arm-nodes/**/*.md` — ARM resource node docs
- `docs/analysis/additional-nodes/*.md` — Specialized Azure node docs
- `docs/analysis/tailscale-nodes/*.md` — Tailscale node docs
- `docs/analysis/edges/*.md` — Edge/relationship docs
- `docs/analysis/post-processing/*.md` — Post-processing step docs

Also read `mkdocs.yml` nav section to know what's already registered.

### 3. Scan all YAML config files

Read all `.tera.yaml` files in:
- `src/graph/config/azure/arm/**/*.tera.yaml` — ARM resource configs
- `src/graph/config/azure/msgraph/*.tera.yaml` — Microsoft Graph configs
- `src/graph/config/post_processing/*.tera.yaml` — Post-processing step configs
- `src/graph/config/tailscale/**/*.tera.yaml` — Tailscale configs

For each YAML file, extract:
- `name` — Display name
- `label` — Node label (references `LABELS.*` constants)
- `properties` — Property paths collected
- `cypher` — The Cypher query, which defines relationships (`REL.*`) and any inline child nodes

### 4. Cross-reference and find gaps

Compare what exists in docs vs what's defined in config:

#### Missing node docs
For each `LABELS.*` constant, check if a corresponding doc file exists. Map labels to doc paths:
- ARM labels → `docs/analysis/arm-nodes/<provider>/<node-name>.md`
- Graph labels → `docs/analysis/graph-nodes/graph-<name>.md`
- Specialized labels → `docs/analysis/additional-nodes/<name>.md`
- Tailscale labels → `docs/analysis/tailscale-nodes/ts-<name>.md`

#### Missing edge docs
For each `REL.*` constant, check if `docs/analysis/edges/<name>.md` exists (convert `UPPER_SNAKE` to `lower-kebab`).

#### Missing post-processing docs
For each file in `src/graph/config/post_processing/`, check if a corresponding doc exists in `docs/analysis/post-processing/`.

#### Missing mkdocs.yml nav entries
Check that every doc file under `docs/analysis/` is listed in the `nav:` section of `mkdocs.yml`.

### 5. Create missing documentation

Use the following templates based on node/edge type.

#### ARM Node Template
File path: `docs/analysis/arm-nodes/<provider>/<node-name>.md`

```markdown
# NodeDisplayName

Description of what the node represents.

**Labels:** `:ArmResource:NodeLabel`

**Properties:**

- `id` - Resource ID (primary key)
- `propertyName` - Description

## Relationships

### Outgoing

- **NodeLabel** → `REL_NAME` → **TargetLabel** - Description

## Examples

\`\`\`cypher
// Example query
MATCH (n:NodeLabel)
RETURN n.name, n.property
\`\`\`
```

#### Graph Node Template
File path: `docs/analysis/graph-nodes/graph-<name>.md`

Same structure as ARM but use **Labels:** `:GraphObject:NodeLabel`

#### Edge Template
File path: `docs/analysis/edges/<rel-name>.md`

```markdown
# REL_NAME

Description of the relationship.

## Usage

- **SourceLabel** → `REL_NAME` → **TargetLabel** - Description

## Properties

No additional properties on the relationship.
_(Or list properties if the cypher sets them on the relationship.)_

## Examples

\`\`\`cypher
// Example query
MATCH (a)-[r:REL_NAME]->(b)
RETURN a.name, b.name
\`\`\`
```

#### Post-Processing Template
File path: `docs/analysis/post-processing/<step-name>.md`

```markdown
# Step Display Name

Description of what this post-processing step does.

**Priority:** N _(if specified in YAML)_

## Details

Explanation of the logic.

## Cypher

\`\`\`cypher
// Cleaned-up cypher from the YAML (resolve tera template variables to readable names)
\`\`\`
```

### 6. Create directories as needed

Run `mkdir -p` for any new subdirectories before creating files (e.g., `docs/analysis/arm-nodes/paloalto/`).

### 7. Update mkdocs.yml

Add new pages to the `nav:` section of `mkdocs.yml` in the appropriate location:
- **Graph Nodes** go under `Nodes and Edges > Graph Nodes`
- **ARM Nodes** go under `Nodes and Edges > ARM Nodes > <Provider>` (create new provider section if needed)
- **Additional Nodes** go under `Nodes and Edges > Additional Nodes`
- **Edges** go under `Nodes and Edges > Edges` (alphabetical order)
- **Post-Processing** goes under a top-level `Post-Processing` section

Keep entries alphabetically sorted within each section.

### 8. Confirm unknown items with the user

If a `LABELS.*` or `REL.*` constant exists in `constants.yaml` but is **not referenced** in any `.tera.yaml` config file, ask the user whether documentation should be created for it or if it's deprecated.

## Notes

- The `mkdocs.yml` uses 2-space indentation for nav entries.
- Some YAML configs define multiple node types inline in cypher (e.g., virtual networks also create Subnet and NetworkPeering nodes). Check the cypher `MERGE` statements for child nodes.
- Post-processing configs use `apoc.periodic.iterate()` and have a `priority` field (0 = runs first).
