# Track W5 Spec: Data Model and Migration Compatibility Graph

## Background

Data model and migration tracking currently scores 7/10. ChangeGuard extracts data models and has modular migrations, but compatibility semantics and links to endpoints, services, tests, and historical incidents are limited.

## Objective

Raise data tracking to 9/10 by modeling tables, entities, migrations, schema changes, compatibility classes, ownership, and affected runtime surfaces.

## Proposed Design

1. Add durable relations for data_model, table, column, migration, schema_change, compatibility_class, owner, and backfill requirement.
2. Parse common migration formats and classify add, drop, rename, type change, index, constraint, and backfill operations.
3. Link ORM models and SQL table usage to endpoints, services, tests, ADRs, and migrations.
4. Add risk rules for destructive migrations, missing backfill tests, incompatible changes, and unowned data models.
5. Add `changeguard data-models impact --changed`.

## Critical Files

| File | Expected work |
|---|---|
| `src/index/data_models.rs` | Extend data model extraction and table/entity links |
| `src/state/migrations/` | Add compatibility metadata migrations if needed |
| `src/coverage/dataflow.rs` | Link compatibility risks to call chains |
| `src/impact/enrichment/` | Add migration compatibility provider |
| `src/commands/` and `src/cli.rs` | Add focused data model impact command |

## Definition of Done

- Migration changes are classified by compatibility risk with evidence.
- Data models link to owners, services, endpoints, tests, ADRs, and migrations where known.
- Destructive or backward-incompatible changes produce targeted verification guidance.
- Target score after completion: 9/10.
