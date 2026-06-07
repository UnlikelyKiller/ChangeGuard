# Track W8 Plan: Dependency, SDK, and Advisory Graph

- [ ] Task W8.1: Create fixtures for Cargo, npm, Python, Go, and scanner output formats.
- [ ] Task W8.2: Write tests for direct/transitive graph construction and deterministic ordering.
- [ ] Task W8.3: Add package graph node and edge types using W1 helpers.
- [ ] Task W8.4: Extend SDK/provider usage extraction and service/config links.
- [ ] Task W8.5: Add local advisory ingestion adapters for scanner JSON outputs.
- [ ] Task W8.6: Add impact rules for vulnerable paths, major upgrades, removed SDKs, and auth/config provider changes.
- [ ] Task W8.7: Add dependency graph review command output with human and JSON modes.
- [ ] Task W8.8: Run package graph, SDK, advisory, and full verification gates; reinstall.

## Definition of Done Checklist

- [ ] Dependency paths distinguish direct and transitive edges.
- [ ] Advisory matches include evidence and affected services where known.
- [ ] No network dependency is required for baseline functionality.
- [ ] Full verification gate passes.
