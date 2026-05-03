# Specification: Track M5-2 — Observability Impact Enrichment

## Objective
Wire the observability signals from M5-1 into the impact analysis pipeline: fetch live system signals during `changeguard impact`, elevate risk when signals exceed thresholds, populate the `observability` field in `ImpactPacket`, and include a compact summary in `ask` context.

## Components

### 1. Fetch Enrichment (`src/observability/mod.rs`)

```rust
pub fn enrich_observability(
    config: &Config,
    conn: &Connection,
    changed_paths: &[&str],
) -> Result<Option<ObservabilitySignal>>
```

Called from `execute_impact()`:
1. Look up service names from `config.observability.service_map` matching each changed file path prefix
2. If no services match AND `log_paths` is empty: return `Ok(None)` immediately (fast path)
3. Call `query_service_batch` for matched services (in parallel with log scanning)
4. Call `scan_logs` (in parallel with Prometheus queries)
5. Call `compute_observability_signal` to produce the final signal
6. Return `Some(signal)` or `None` if no signals were produced

### 2. Risk Elevation in `execute_impact()`

After computing the `ObservabilitySignal`:
- If `signal.risk_elevation` is `Some(reason)`:
  - Elevate the risk tier by one level (Low → Medium → High)
  - High is the ceiling (High stays High)
  - Append the reason to `packet.risk_reasons`
- Assign `packet.observability = Some(signal)`

### 3. New `ImpactPacket` Field (`src/impact/packet.rs`)

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub observability: Option<ObservabilitySignal>,
```

### 4. Timing Constraint

Observability fetching runs in parallel with other impact enrichment. Total wall-clock time added must not exceed 6 seconds (5s Prometheus timeout + 1s buffer). If the parallel fetch exceeds this, log `WARN` and include whatever partial results were obtained.

### 5. Enrichment in `changeguard scan --impact`

The observability fetch is triggered automatically during `execute_impact()`. No new CLI flags are needed — enabled when config is populated, silently skipped when not.

### 6. Ask Context Injection (`src/commands/ask.rs`)

If `packet.observability` is `Some`, include a compact summary in the user prompt:

```
## Live System Signals
Service payments-svc: error_rate=3.2%, latency_p99=450ms (above threshold)
Log anomalies: 4 chunks semantically similar to this change in the last hour
```

Enforce budget: observability block is trimmed before decisions/couplings/hotspots if context overflows.

## Test Specifications

| Test | Assertion |
|---|---|
| `enrich_observability` no services, no logs | Returns `Ok(None)` immediately |
| `enrich_observability` service above threshold | Signal returned with `risk_elevation = Some(...)` |
| Risk elevation Low + obs above threshold | Packet `risk_level = Medium` |
| Risk elevation Medium + obs above threshold | Packet `risk_level = High` |
| Risk elevation High + obs above threshold | Packet `risk_level = High` (ceiling) |
| Risk reason appended | `risk_reasons` contains obs elevation reason |
| `ImpactPacket` with `observability` | Serializes + deserializes correctly |
| `ImpactPacket` with `observability = None` | Field absent in JSON |
| Ask context with observability signal | Summary block present in prompt |
| `prometheus_url = ""` | `observability` is `None`, impact completes normally |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **Parallel by default**: Observability fetch runs in parallel with other enrichment; no blocking.
- **6-second wall-clock cap**: Total observability time budget enforced.
- **No blocking failures**: Prometheus unreachable, log file missing, embedding unavailable — all are graceful degrades.
- **CI safety**: All tests pass with `observability.prometheus_url = ""` and `observability.log_paths = []`.

## Hardening Additions (not in original plan)

| Addition | Reason |
|---|---|
| `observability` field cleared in `ImpactPacket::truncate_for_context()` Phase 3 | `ObservabilitySignal` can contain unbounded log anomaly excerpts. Must be stripped under context budget pressure alongside other large fields. |
