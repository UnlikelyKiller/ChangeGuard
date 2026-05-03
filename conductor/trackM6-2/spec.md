# Specification: Track M6-2 — Contract Matching & Impact Enrichment

## Objective
Wire the indexed API endpoints into impact analysis: match changed files to semantically related endpoints via embedding similarity, flag public API contract risk, surface affected contracts in the impact packet, and display them in human-readable output.

## Components

### 1. Contract Matcher (`src/contracts/matcher.rs`)

```rust
pub fn match_contracts(
    conn: &Connection,
    changed_files: &[&str],
    model_name: &str,
    similarity_threshold: f32,
) -> Result<Vec<AffectedContract>>
```

For each changed file:
1. Retrieve the file's embedding from `embeddings` where `entity_type = "file"` and `entity_id` matches the normalized file path
2. If no embedding exists for the file (not yet indexed): skip contract matching for that file
3. Load all `entity_type = "api_endpoint"` embeddings via `load_candidates`
4. Compute cosine similarity between file embedding and each endpoint embedding
5. Collect endpoints with similarity > `similarity_threshold` (default: 0.5)

Deduplication:
- If the same endpoint is matched by multiple changed files, keep the highest similarity score
- Sort by similarity descending; cap at 10 endpoints

### 2. `AffectedContract` Type (`src/impact/packet.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AffectedContract {
    pub spec_path: PathBuf,
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
    pub similarity: f32,
}

// Manual Ord: sort by similarity descending, then spec_path, method, path ascending
impl Eq for AffectedContract {}
impl PartialOrd for AffectedContract { ... }
impl Ord for AffectedContract { ... }
```

Add to `ImpactPacket`:
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub affected_contracts: Vec<AffectedContract>,
```

**Determinism updates in `ImpactPacket`:**
- Add `self.affected_contracts.sort_unstable()` in `ImpactPacket::finalize()` (after `relevant_decisions.sort_unstable()`)
- Add `self.affected_contracts.clear()` in `ImpactPacket::truncate_for_context()` Phase 3

### 3. Impact Enrichment (`src/commands/impact.rs`)

In `execute_impact()`, after existing enrichment and after document enrichment (M2-2):
1. Call `match_contracts` with the current changed file paths
2. Assign results to `packet.affected_contracts`

Skip when:
- `config.contracts.spec_paths` is empty
- `api_endpoints` table has zero rows
- No file embeddings exist for any changed file

### 4. Risk Elevation

If any `affected_contracts` entry has similarity > 0.75 AND the changed file contains a public symbol change (from existing `analysis_status`):
- Add reason: `"Public contract potentially affected: {method} {path}"` to `risk_reasons`
- This is an informational reason added to the risk score; it does NOT automatically elevate the risk tier by itself (unlike observability, which directly elevates the tier)

### 5. Human Output Table

In `changeguard impact` output, when `affected_contracts` is non-empty:

```
Affected API Contracts
 Method  Path                Spec                  Similarity
 POST    /v1/payments        api/openapi.yaml       0.84
 GET     /v1/payments/{id}   api/openapi.yaml       0.71
```

### 6. Ask Context Injection

If `affected_contracts` is non-empty, include in the ask prompt:

```
## Affected API Contracts
- POST /v1/payments (api/openapi.yaml, similarity=0.84)
- GET /v1/payments/{id} (api/openapi.yaml, similarity=0.71)
```

Enforce budget: contract context is trimmed before decisions/couplings if context overflows.

## Test Specifications

| Test | Assertion |
|---|---|
| `match_contracts` 2 files, 5 endpoints | Top matches >0.5 returned sorted |
| `match_contracts` no file embeddings | Returns empty vec |
| `match_contracts` no endpoint embeddings | Returns empty vec |
| Deduplication same endpoint matched twice | Keeps highest similarity |
| Cap at 10 endpoints | Returns at most 10 |
| `AffectedContract` serialization | Round-trips correctly |
| `ImpactPacket` with `affected_contracts` | Field present in JSON |
| `ImpactPacket` empty `affected_contracts` | Field absent in JSON |
| Impact enrichment seeded fixtures | `affected_contracts` non-empty |
| Risk reason for similarity > 0.75 | Reason added to `risk_reasons` |
| Risk reason for similarity < 0.75 | No reason added |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **No blocking on hot path**: Contract matching runs within the impact pipeline; if it exceeds 200ms, log a `WARN` but do not fail.
- **Config-driven**: When `contracts.spec_paths` is empty, the entire feature is a no-op.
- **No embedding on hot path**: File embeddings must already exist from a prior `changeguard index` run; no embedding generation during impact.
- **Excerpt safety**: `summary` field is sanitized before inclusion (run through existing sanitizer).

## Hardening Additions (not in original plan)

| Addition | Reason |
|---|---|
| `AffectedContract` implements `Eq + Ord` (sort by similarity descending) | Required by `ImpactPacket::finalize()` determinism contract. |
| `affected_contracts` cleared in `truncate_for_context()` Phase 3 | Must be stripped under context budget pressure to honor the 38k token limit. |
| `affected_contracts` sorted in `finalize()` | Deterministic JSON output; byte-identical impact reports across runs. |
