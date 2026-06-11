# Track Z4: Cargo.lock Dependency Ingestion
 
**Status:** In Progress
**Milestone:** Z — Command Audit Remediation & Ollama Cloud Hardening
**Priority:** High

## Objective

Populate the `dependencies list` table from `Cargo.lock` during the Knowledge Graph loading phase so that it is populated on standard indexing without requiring external OSV data files.

## Problem Statement

Currently, the `dependencies list` command queries CozoDB for `category: 'package'` nodes but returns an empty table. `NodeKind::Package` is defined, but nothing in the graph loader actually reads `Cargo.lock` and inserts these package nodes/edges during `changeguard index --analyze-graph`.

## Acceptance Criteria

1. During `changeguard index --analyze-graph`, the graph loader scans for `Cargo.lock` at the repository root.
2. If `Cargo.lock` is present, it is parsed to extract all packages (name, version, and dependencies).
3. The package names/versions are inserted into CozoDB as `NodeKind::Package` nodes with metadata `{version, ecosystem: "rust/cargo", manifest: "Cargo.lock"}` and URN `urn:changeguard:package:{name}:{version}`.
4. outgoing `DependsOn` edges are created to link dependent packages.
5. If no `Cargo.lock` is found, a `warn!` is printed and execution continues cleanly.

## Key Files

* `src/index/graph_loader.rs` — Graph loading pipeline.

## Definition of Done

* `cargo nextest run --lib --bins --workspace` passes.
* Running `changeguard index --analyze-graph && changeguard dependencies list` shows packages.
