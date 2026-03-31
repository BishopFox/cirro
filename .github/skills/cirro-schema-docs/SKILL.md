---
name: cirro-schema-docs
description: Synchronize Cirro analysis documentation and mkdocs navigation with schema constants and graph config YAML files.
---

# Cirro Schema Docs Skill

## Purpose

Use this skill when Cirro graph config files or schema constants change and documentation must be updated to match.

Primary sources of truth:
- `src/graph/config/constants.yaml`
- `src/graph/config/**/*.tera.yaml`

Primary documentation targets:
- `docs/analysis/**`
- `mkdocs.yml` nav entries

## When To Use

Trigger examples:
- sync cirro docs
- update node and edge documentation
- ensure docs match cirro config
- add missing cirro documentation

## Required Workflow

1. Read constants.
	- Parse `REL.*` and `LABELS.*` from `src/graph/config/constants.yaml`.

2. Inventory existing docs.
	- `docs/analysis/graph-nodes/*.md`
	- `docs/analysis/arm-nodes/**/*.md`
	- `docs/analysis/additional-nodes/*.md`
	- `docs/analysis/tailscale-nodes/*.md`
	- `docs/analysis/edges/*.md`
	- `docs/analysis/post-processing/*.md`
	- Also inspect the `nav:` section in `mkdocs.yml`.

3. Scan config YAML.
	- `src/graph/config/azure/arm/**/*.tera.yaml`
	- `src/graph/config/azure/msgraph/*.tera.yaml`
	- `src/graph/config/post_processing/*.tera.yaml`
	- `src/graph/config/tailscale/**/*.tera.yaml`
	- For each file, extract `name`, `label`, `properties`, and all relationship usage from `cypher`.

4. Cross-reference and find gaps.
	- Missing node docs from `LABELS.*`
	- Missing edge docs from `REL.*` (convert `UPPER_SNAKE` to `lower-kebab`)
	- Missing post-processing docs from files in `src/graph/config/post_processing/`
	- Missing `mkdocs.yml` nav entries for all docs under `docs/analysis/`

5. Create or update docs.
	- Prefer minimal edits and keep style consistent with nearby files.
	- Create missing directories before creating files.

6. Update `mkdocs.yml` nav.
	- Place docs in the correct analysis sections.
	- Keep entries alphabetically sorted within each section.
	- Preserve existing indentation and structure.

7. Ask for confirmation on unused constants.
	- If a `LABELS.*` or `REL.*` constant is not referenced by any `.tera.yaml`, ask whether to document it or treat it as deprecated.

## Mapping Rules

- ARM labels -> `docs/analysis/arm-nodes/<provider>/<node-name>.md`
- Graph labels -> `docs/analysis/graph-nodes/graph-<name>.md`
- Additional labels -> `docs/analysis/additional-nodes/<name>.md`
- Tailscale labels -> `docs/analysis/tailscale-nodes/ts-<name>.md`
- Edge docs -> `docs/analysis/edges/<rel-name>.md`
- Post-processing docs -> `docs/analysis/post-processing/<step-name>.md`

## Content Templates

Use these as baseline structures.

### Node Doc

~~~markdown
# NodeDisplayName

Description of what the node represents.

**Labels:** `:ArmResource:NodeLabel`

**Properties:**
- `id` - Resource ID (primary key)
- `propertyName` - Description

## Relationships

### Outgoing
- **NodeLabel** -> `REL_NAME` -> **TargetLabel** - Description

## Examples

```cypher
MATCH (n:NodeLabel)
RETURN n.name, n.property
```
~~~

For Graph nodes, use `:GraphObject:NodeLabel` in `Labels`.

### Edge Doc

~~~markdown
# REL_NAME

Description of the relationship.

## Usage
- **SourceLabel** -> `REL_NAME` -> **TargetLabel** - Description

## Properties
No additional properties on the relationship.

## Examples

```cypher
MATCH (a)-[r:REL_NAME]->(b)
RETURN a.name, b.name
```
~~~

### Post-Processing Doc

~~~markdown
# Step Display Name

Description of what this post-processing step does.

**Priority:** N

## Details
Explanation of the logic.

## Cypher

```cypher
// Cleaned-up cypher from YAML
```
~~~

## Guardrails

- Treat YAML config and constants as source of truth.
- Do not invent labels or relationships not present in source files.
- Keep edits deterministic, concise, and sorted where applicable.
- Preserve existing docs style and naming conventions.
