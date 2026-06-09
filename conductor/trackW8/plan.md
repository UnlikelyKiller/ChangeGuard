# Track W8 Plan: Dependency, SDK, and Advisory Graph

- [x] Task W8.1: Create fixtures for Cargo, npm, Python, Go, OSV-Scanner JSON, and optional scanner compatibility output formats.
- [x] Task W8.2: Write tests for direct/transitive graph construction and deterministic ordering.
- [x] Task W8.3: Add package graph node and edge types using W1 helpers.
- [x] Task W8.4: Extend SDK/provider usage extraction and service/config links.
- [x] Task W8.5: Add the primary OSV-Scanner JSON importer, including offline-run metadata and schema-version handling.
- [x] Task W8.6: Add optional compatibility importers for cargo-deny, cargo-audit, npm audit, and pip-audit only after OSV ingestion is green.
- [x] Task W8.7: Add impact rules for OSV vulnerable paths, major upgrades, removed SDKs, and auth/config provider changes.
- [x] Task W8.8: Add dependency graph review command output with human and JSON modes.
- [x] Task W8.9: Document the local-first OSV workflow, including `--offline`, local DB cache expectations, and failure behavior when the cache is missing.
- [x] Task W8.10: Run package graph, SDK, advisory, and full verification gates; reinstall.

## Definition of Done Checklist

- [x] Dependency paths distinguish direct and transitive edges.
- [x] OSV advisory matches include evidence and affected services where known.
- [x] Optional scanner compatibility imports cannot bypass the normalized OSV-style advisory graph.
- [x] No network dependency is required for baseline functionality.
- [x] Full verification gate passes.
