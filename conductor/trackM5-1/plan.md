## Plan: Track M5-1 — Prometheus Client & Log Scanner

### Phase 1: Prometheus Client
- [ ] Task 1.1: Create `src/observability/prometheus.rs` with `query_service(prometheus_url, service_name, timeout_secs) -> Result<Option<PrometheusSignal>>`.
- [ ] Task 1.2: Construct PromQL queries for error rate and latency as documented in the spec.
- [ ] Task 1.3: Use `ureq` GET to `{prometheus_url}/api/v1/query?query={encoded}`. Parse response `data.result[0].value[1]` as f32.
- [ ] Task 1.4: On any error (unreachable, non-200, malformed JSON): return `Ok(None)` — never fail.
- [ ] Task 1.5: Add `query_service_batch(prometheus_url, service_names, timeout_secs) -> Result<Vec<Option<PrometheusSignal>>>`: up to 8 concurrent queries.
- [ ] Task 1.6: Write unit test: mock Prometheus server returns valid `{"data":{"result":[{"value":[1234567890,"0.032"]}]}}` → parsed correctly.
- [ ] Task 1.7: Write unit test: mock Prometheus server unreachable → returns `Ok(None)`.
- [ ] Task 1.8: Write unit test: `query_service_batch` with 3 services returns 3 results.

### Phase 2: Log Scanner
- [ ] Task 2.1: Create `src/observability/log_scanner.rs` with `scan_logs(config, conn) -> Result<Vec<LogAnomaly>>`.
- [ ] Task 2.2: Read files matching `config.observability.log_paths` globs; 10 MB total cap; bounded by `log_lookback_secs`.
- [ ] Task 2.3: Chunk log lines into groups of 20 lines. Embed each chunk via `embed_and_store`.
- [ ] Task 2.4: Compute cosine similarity between each log chunk embedding and current diff embedding (from `embeddings` table with `entity_type = "diff"`).
- [ ] Task 2.5: Flag chunks with similarity > 0.6 as anomalies; include 100-char excerpt.
- [ ] Task 2.6: Implement keyword fallback when embedding unavailable: grep for `ERROR`, `FATAL`, `panic`, `exception`.
- [ ] Task 2.7: Write unit test: log file with ERROR lines → fallback keyword match returns anomalies.
- [ ] Task 2.8: Write unit test: no log files configured → returns empty vec, no error.
- [ ] Task 2.9: Write unit test: log file exceeds 10 MB cap → only first 10 MB read.

### Phase 3: ObservabilitySignal & Risk Elevation
- [ ] Task 3.1: Create `src/observability/signal.rs` with `ObservabilitySignal`, `ServiceSignal` structs.
- [ ] Task 3.2: Implement `compute_observability_signal(prometheus_signals, log_anomalies, error_rate_threshold) -> ObservabilitySignal`.
- [ ] Task 3.3: Set `above_threshold` per service; set `risk_elevation` if any service exceeds threshold.
- [ ] Task 3.4: INSERT snapshot rows into `observability_snapshots` table for each service.
- [ ] Task 3.5: Write unit test: service above threshold → `risk_elevation = Some(...)` and `above_threshold = true`.
- [ ] Task 3.6: Write unit test: all services below threshold → `risk_elevation = None`.
- [ ] Task 3.7: Write unit test: snapshot INSERT + SELECT round-trips correctly.

### Phase 4: Module Setup
- [ ] Task 4.1: Create `src/observability/mod.rs` exporting `prometheus`, `log_scanner`, `signal`.
- [ ] Task 4.2: Add `pub mod observability;` to `src/lib.rs`.

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`.
- [ ] Task 5.2: Run `cargo test --lib observability` — all tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions.
