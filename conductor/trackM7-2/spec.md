# Specification: Track M7-2 — Service-Map Derivation

## Objective
Infer service boundaries from route/handler/data-model topology, derive cross-service dependency edges, and surface multi-service change risk in the impact packet.

## Components

### 1. Service Inference (`src/coverage/services.rs`)

```rust
pub fn infer_services(
    routes: &[Route],
    call_graph: &CallGraph,
    topology: &DirectoryTopology,
) -> Vec<Service>
```

Multi-strategy service naming (in priority order):
1. Directory name: `src/api/users/` → service "users"
2. Package name: nearest `Cargo.toml`/`package.json`/`__init__.py` parent directory
3. Fallback: "unnamed-service-N"

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub routes: Vec<String>,
    pub data_models: Vec<String>,
    pub directory: PathBuf,
}
```

### 2. Cross-Service Edge Computation

```rust
pub fn compute_cross_service_edges(
    services: &[Service],
    call_graph: &CallGraph,
) -> Vec<(String, String, usize)>
```

From call-graph edges, identify caller→callee pairs where caller and callee belong to different services. Collapse multiple edges between the same service pair into one edge with a count.

### 3. Service Map Delta (`src/impact/packet.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMapDelta {
    pub affected_services: Vec<String>,
    pub cross_service_edges: Vec<(String, String, usize)>,
    pub total_services: usize,
}
```

### 4. Risk Impact

| Changed services | Elevation | Reason |
|---|---|---|
| 2 | Low | Cross-service change: {svc_a} → {svc_b} |
| 3-4 | Medium | Multi-service change spanning {n} services |
| 5+ | High | Large blast radius: {n} services affected |

### 5. Impact Enrichment

In `execute_impact()`, compute `ServiceMapDelta` from pre-indexed service assignments and populate `packet.service_map_delta`. Service assignments are computed during `changeguard index`, not during `impact`.

## Test Specifications

| Test | Assertion |
|---|---|
| Routes in `src/api/users/` → service "users" | `Service.name == "users"` |
| Routes in flat repo `src/handler.rs` → service "src" | Single service, not exploded |
| No routes detected → `None` | `ServiceMapDelta` returns `None` |
| Cross-service edges computed | Edges contain (svc_a, svc_b, count) |
| Multiple edges collapsed | 5 A→B edges become one with count=5 |
| Monorepo depth cap | Directory nesting >2 levels caps at 2 |
| Package-name fallback | `package.json` parent dir used when no directory name |
| Unnamed-service fallback | No directory, no package → "unnamed-service-1" |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **Index-time computation**: Service map is derived during `changeguard index`, queried during `impact`.
- **Config-driven**: `[coverage.services].enabled = false` → service map not computed.
- **No hot-path embedding**: Service inference uses existing AST-level data only.
- **Deterministic output**: Service ordering is deterministic (alphabetical by name).

## Hardening Additions (in plan)

| Addition | Reason |
|---|---|
| Multi-strategy service naming | Directory→package→fallback ensures no empty service names |
| Service dedup by route ownership | Closest handler file wins tiebreaks |
| Monorepo-aware depth cap | Flat repos don't explode into N services |
| Empty route set returns None | CLI tools/libraries don't produce meaningless maps |
| Cross-service edge dedup with counts | Multiple edges between same pair collapsed |
