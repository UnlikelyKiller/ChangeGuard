# Specification: Track M5-1 — Prometheus Client & Log Scanner

## Objective
Build the observability data-fetching infrastructure: a Prometheus PromQL query client and a local log file scanner. Establish the `ObservabilitySignal` type and snapshot storage. This is the input side; M5-2 wires it into impact enrichment.

## Components

### 1. Prometheus Client (`src/observability/prometheus.rs`)

```rust
pub struct PrometheusSignal {
    pub service_name: String,
    pub error_rate: Option<f32>,
    pub latency_p99_ms: Option<f32>,
}

pub fn query_service(
    prometheus_url: &str,
    service_name: &str,
    timeout_secs: u64,
) -> Result<Option<PrometheusSignal>>
```

For each changed file, look up matching service name(s) from `config.observability.service_map`. For each matched service:

1. Execute two PromQL instant queries:
   - Error rate: `rate(http_requests_total{job="{service}", status=~"5.."}[5m]) / rate(http_requests_total{job="{service}"}[5m])`
   - Latency: `histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{job="{service}"}[5m]))`
2. Use Prometheus HTTP API: `GET {prometheus_url}/api/v1/query?query={encoded_promql}`
3. Timeout: 5 seconds per batch (all service queries in one batch)

On any error (unreachable, non-200, malformed JSON): return `Ok(None)` per service — never fail the caller.

### 2. `query_service_batch` for parallelism

```rust
pub fn query_service_batch(
    prometheus_url: &str,
    service_names: &[&str],
    timeout_secs: u64,
) -> Result<Vec<Option<PrometheusSignal>>>
```

Parallelize up to 8 concurrent queries. Total wall-clock timeout remains 5 seconds.

### 3. Log Scanner (`src/observability/log_scanner.rs`)

```rust
pub struct LogAnomaly {
    pub chunk_excerpt: String,  // first 100 chars
    pub similarity: f32,
}

pub fn scan_logs(
    config: &Config,
    conn: &Connection,
) -> Result<Vec<LogAnomaly>>
```

For files matching `config.observability.log_paths` globs:
1. Read newest bytes first, bounded by `config.observability.log_lookback_secs` seconds back
2. Hard cap: 10 MB total across all log files per run
3. Chunk log lines into groups of 20 lines (preserving temporal proximity)
4. Embed each chunk using the local embedding model (from M1 infrastructure)
5. Compute cosine similarity between each log chunk embedding and the current diff embedding
6. Flag chunks with similarity > 0.6 as anomalies; collect up to 20
7. Raw log content is never included in the anomaly struct — only a 100-char excerpt

Fallback when embedding model unavailable:
- Grep log chunks for keywords: `ERROR`, `FATAL`, `panic`, `exception`
- Include matching chunks as anomalies with `similarity = 1.0`

### 4. `ObservabilitySignal` Type (`src/observability/signal.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObservabilitySignal {
    pub services: Vec<ServiceSignal>,
    pub log_anomaly_count: usize,
    pub risk_elevation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceSignal {
    pub service_name: String,
    pub error_rate: Option<f32>,
    pub latency_p99_ms: Option<f32>,
    pub above_threshold: bool,
}
```

### 5. Risk Elevation Logic (`src/observability/signal.rs`)

```rust
pub fn compute_observability_signal(
    prometheus_signals: &[PrometheusSignal],
    log_anomalies: &[LogAnomaly],
    error_rate_threshold: f32,
) -> ObservabilitySignal
```

- For each service where `error_rate > error_rate_threshold`: set `above_threshold = true`
- If any service is above threshold: `risk_elevation = Some("Service {name} error rate {rate:.1%} exceeds threshold")`
- `log_anomaly_count = log_anomalies.len()`
- Latency is informational only; it does not affect risk tier

### 6. Snapshot Storage

INSERT a row into `observability_snapshots` for each service with the current signal values. Used to detect trends in M5-2 (current vs. 1-hour-ago snapshot).

### 7. Module declaration

Create `src/observability/mod.rs` exporting `prometheus`, `log_scanner`, `signal` submodules. Add `pub mod observability;` to `src/lib.rs`.

## Test Specifications

| Test | Assertion |
|---|---|
| `query_service` mock Prometheus returns valid error rate | Parsed correctly into `PrometheusSignal` |
| `query_service` mock Prometheus unreachable | Returns `Ok(None)`, not error |
| `query_service_batch` 3 services | All 3 results returned |
| `scan_logs` log file with ERROR lines | Fallback keyword match finds anomalies |
| `scan_logs` no log files configured | Returns empty vec |
| `compute_observability_signal` service above threshold | `risk_elevation = Some(...)`, `above_threshold = true` |
| `compute_observability_signal` all below threshold | `risk_elevation = None` |
| Snapshot INSERT + SELECT | Round-trips correctly |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **No panics**: Prometheus unreachable is `Ok(None)`, never an error.
- **Non-blocking**: Observability fetch failure never aborts `impact`.
- **No raw logs in output**: Only 100-char excerpts and counts.
- **CI safety**: All tests pass with `observability.prometheus_url = ""` and `observability.log_paths = []`.
