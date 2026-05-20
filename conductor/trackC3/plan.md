## Plan: Track C3 - Predictive Verification IPC & Watcher Intervention

### Phase 1: Predictive Verification IPC Endpoint
- [x] Task 1.1: Created `src/verify/ipc_verify.rs` with `predictive_verify(scope, layout)` function returning structured `IpcPredictiveResult`.
- [x] Task 1.2: Wired existing predictive CI logic — builds scoped `ImpactPacket`, computes failure probability from rule scores (clamped [0.05, 0.95]), checks ledger for drift.
- [x] Task 1.3: IPC verify mode is fast — rule-based only (no embedding), deterministic results for same input.
- [x] Task 1.4: Returns `failure_probability`, `drift_detected`, `risk_level`, `predicted_files`, `warnings` as structured fields.
- [x] Task 1.5: Write 10 unit tests: empty scores, bounded probability, risk derivation, scoped packet, dedup, determinism, drift check, serialization.

### Phase 2: Watcher Risk Alerts
- [x] Task 2.1: Added `push_risk_alert()` to `src/bridge/notify.rs` with `BridgePayload::RiskAlert` containing coupled_file_a, coupled_file_b, coupling_score, affected_symbols, suggested_remediation, risk_level.
- [x] Task 2.2: Added `RiskAlert` variant to `BridgePayload` in `src/bridge/model.rs` and match arm in `src/bridge/import.rs`.
- [x] Task 2.3: Integrated `check_temporal_coupling_alerts()` into `src/commands/watch.rs` — throttled every 10th batch, uses `gix::open` + `TemporalEngine` + `GixHistoryProvider`.
- [x] Task 2.4: Per-session deduplication via `LazyLock<Mutex<HashSet<(String,String)>>>` with canonicalized file pairs.
- [x] Task 2.5: Fire-and-forget: spawns thread with 100ms connect timeout; all failures logged at `tracing::debug!`.
- [x] Task 2.6: Write 4 unit tests: dedup, below-threshold rejection, different pairs, constant verification.

### Phase 3: Verification
- [x] Task 3.1: `cargo fmt --all -- --check ; cargo clippy --all-targets --all-features -- -D warnings ; cargo test --workspace` — 864 passed, 0 failed
- [x] Task 3.2: Watcher integration test: risk alerts fire on high coupling in watched repo.
- [x] Task 3.3: IPC verify endpoint returns results well within 500ms target.
