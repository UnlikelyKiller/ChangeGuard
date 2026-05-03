## Plan: Track M5-2 — Observability Impact Enrichment

### Phase 1: Fetch Enrichment in `src/observability/mod.rs`
- [ ] Task 1.1: Add `enrich_observability(config, conn, changed_paths) -> Result<Option<ObservabilitySignal>>` to `src/observability/mod.rs`.
- [ ] Task 1.2: Look up service names from `config.observability.service_map` for each changed file path prefix. Deduplicate.
- [ ] Task 1.3: Fast-path: if no services matched AND `log_paths` is empty, return `Ok(None)` immediately.
- [ ] Task 1.4: Run `query_service_batch` and `scan_logs` in parallel (tokio or rayon, depending on existing project dep — use `std::thread::scope` if no async framework available).
- [ ] Task 1.5: Call `compute_observability_signal` to produce final signal.
- [ ] Task 1.6: Write unit test: no services, no logs → returns `Ok(None)`.
- [ ] Task 1.7: Write unit test: service above threshold → signal with `risk_elevation = Some(...)`.

### Phase 2: Impact Enrichment & Risk Elevation
- [ ] Task 2.1: In `execute_impact()` in `src/commands/impact.rs`, after existing enrichment, call `enrich_observability`.
- [ ] Task 2.2: If `signal.risk_elevation` is `Some(reason)`: elevate risk tier by one level (capped at High), append reason to `risk_reasons`.
- [ ] Task 2.3: Assign `packet.observability = Some(signal)`.
- [ ] Task 2.4: Enforce 6-second wall-clock cap on observability fetch (timeout wrapper).
- [ ] Task 2.5: Write unit test: Low risk + obs above threshold → `risk_level = Medium`, reason in `risk_reasons`.
- [ ] Task 2.6: Write unit test: Medium risk + obs above threshold → `risk_level = High`.
- [ ] Task 2.7: Write unit test: High risk + obs above threshold → `risk_level = High` (ceiling).
- [ ] Task 2.8: Write unit test: `prometheus_url = ""` → `observability` is `None`, impact completes normally.

### Phase 3: ImpactPacket Field
- [ ] Task 3.1: Add `observability: Option<ObservabilitySignal>` to `ImpactPacket` in `src/impact/packet.rs` with `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- [ ] Task 3.2: Write unit test: `ImpactPacket` with observability signal serializes + deserializes.
- [ ] Task 3.3: Write unit test: `ImpactPacket` with `observability = None` serializes without the field.

### Phase 3.5: Determinism & Truncation
- [ ] Task 3.4: In `ImpactPacket::truncate_for_context()` Phase 3, add `self.observability = None;` after clearing `runtime_usage_delta` to strip observability signals under context budget pressure.
- [ ] Task 3.5: Write unit test: `truncate_for_context()` sets `observability` to `None` when budget exceeded.

### Phase 4: Ask Context Injection
- [ ] Task 4.1: In `execute_ask()` or context assembly, add a function `format_observability_signal(signal: &ObservabilitySignal) -> String` producing the documented markdown block.
- [ ] Task 4.2: Include observability block in context assembly between decisions and couplings.
- [ ] Task 4.3: Enforce budget: observability block is trimmed from context if it exceeds budget (trimmed before decisions).
- [ ] Task 4.4: Write unit test: ask context with observability signal includes summary block.
- [ ] Task 4.5: Write unit test: ask context overflow → observability trimmed before decisions.

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`.
- [ ] Task 5.2: Run `cargo test --lib observability` — all tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions.
- [ ] Task 5.4: Verify `changeguard impact` completes normally with all observability config empty.
