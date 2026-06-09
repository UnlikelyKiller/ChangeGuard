# Track X2: `dependencies list` Populates from Cargo.lock During Index

**Status:** Completed  
**Milestone:** X — Command Surface Correctness  
**Priority:** High

## Objective

`changeguard dependencies list` queries CozoDB for `category: 'package'` nodes but always shows an empty table unless `dependencies audit --input osv.json` has been run first. For Rust projects, Cargo.lock contains the full resolved dependency tree and should be parsed during `changeguard index --analyze-graph` to create `Package` nodes automatically.

## Problem Statement

`NodeKind::Package` exists in `graph_kinds.rs` and `dependencies list` queries it correctly, but `graph_loader.rs` never populates these nodes from Cargo.lock. The only writer is `OsvImporter::populate_kg` in `src/index/advisories.rs`, which requires an external OSV JSON file. A fresh `changeguard index` leaves the package table empty.

## Acceptance Criteria

1. After `changeguard index --analyze-graph` on any Rust project with a `Cargo.lock`, `dependencies list` shows all direct and transitive dependencies with name, version, and `rust/cargo` ecosystem.
2. Packages are represented as `NodeKind::Package` nodes in CozoDB with `metadata: {version: "...", ecosystem: "rust/cargo", manifest: "Cargo.lock", direct: true/false}`.
3. `DependsOn` edges link packages to each other based on the Cargo.lock dependency tree (package → dependency).
4. Re-indexing is idempotent: existing package nodes are upserted, not duplicated.
5. The `Cargo.lock` parser handles workspaces (multiple `[[package]]` sections).
6. When no `Cargo.lock` is found, a `warn!` is emitted and the command continues cleanly.

## API Contracts

`dependencies list` output (unchanged):
```
Package         Version    Ecosystem
---             ---        ---
serde           1.0.228    rust/cargo
miette          7.6.0      rust/cargo
...
```

`dependencies list --json` emits the same data as structured JSON.

CozoDB node shape:
```
node{
  id: "urn:changeguard:package:serde:1.0.228",
  label: "serde",
  category: "package",
  metadata: {"version":"1.0.228","ecosystem":"rust/cargo","manifest":"Cargo.lock","direct":true}
}
```

## Key Files

- `src/index/graph_loader.rs` — add Section 10: Cargo.lock ingestion
- `src/index/advisories.rs` — reference for package node/edge shape
- `src/state/graph_kinds.rs` — `NodeKind::Package`, `EdgeKind::DependsOn`
- `src/platform/urn.rs` — `build_urn` helper

## Dependencies

None — standard library `std::fs::read_to_string` + `toml` crate (already in Cargo.toml at `1.1.2`) for Cargo.lock parsing.

## Definition of Done

- `changeguard index --analyze-graph && changeguard dependencies list` shows 200+ packages on this repo.
- `changeguard dependencies list --json | jq length` > 0.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
