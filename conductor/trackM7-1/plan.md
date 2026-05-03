## Plan: Track M7-1 — Trace Config & SDK Dependency Detection

### Phase 1: Module Setup
- [ ] Task 1.1: Create `src/coverage/mod.rs` with `pub mod traces; pub mod sdk;` declarations.
- [ ] Task 1.2: Register `pub mod coverage;` in `src/lib.rs`.
- [ ] Task 1.3: Add `CoverageConfig` (with `traces` and `sdk` sub-sections) to `src/config/model.rs`, defaults all `enabled = false`.

### Phase 2: Trace Config Detection
- [ ] Task 2.1: Implement `detect_trace_config_changes()` in `src/coverage/traces.rs`.
- [ ] Task 2.2: Implement glob-safe pattern matching with fallback (invalid glob → WARN + skip).
- [ ] Task 2.3: Implement stale trace config detection (files deleted since last scan).
- [ ] Task 2.4: Write test: otel-collector yaml detected as `OpenTelemetryCollector`.
- [ ] Task 2.5: Write test: non-trace yaml skipped.
- [ ] Task 2.6: Write test: invalid glob pattern does not abort.
- [ ] Task 2.7: Write test: `.yaml.tmpl` double-extension matched.

### Phase 3: Trace Env-Var Detection
- [ ] Task 3.1: Implement `detect_trace_env_vars()` in `src/coverage/traces.rs`.
- [ ] Task 3.2: Implement `exclude_env_patterns` filtering.
- [ ] Task 3.3: Write test: `OTEL_EXPORTER_OTLP_ENDPOINT` flagged.
- [ ] Task 3.4: Write test: `OTEL_SDK_DISABLED` excluded by pattern.
- [ ] Task 3.5: Write test: non-trace env var `DATABASE_URL` not flagged.

### Phase 4: SDK Dependency Detection
- [ ] Task 4.1: Implement `detect_sdk_changes()` in `src/coverage/sdk.rs`.
- [ ] Task 4.2: Implement language-aware import extraction (Rust `use`, Python `from`/`import`, JS/TS `import`/`require`, Go `import`).
- [ ] Task 4.3: Implement case-insensitive matching.
- [ ] Task 4.4: Compute SdkDependencyDelta (added, removed, modified) by comparing current scan vs previous.
- [ ] Task 4.5: Write test: Rust `use stripe::Charge` → Stripe detected.
- [ ] Task 4.6: Write test: Python `from stripe import Charge` → Stripe detected.
- [ ] Task 4.7: Write test: JS `import { Stripe } from "stripe"` → Stripe detected.
- [ ] Task 4.8: Write test: Go `import "github.com/stripe/stripe-go"` → Stripe detected.
- [ ] Task 4.9: Write test: case-insensitive `STRIPE`, `Stripe`, `stripe` all match.

### Phase 5: Types
- [ ] Task 5.1: Define `TraceConfigType` enum with `Ord` derive.
- [ ] Task 5.2: Define `TraceConfigChange`, `TraceEnvVarChange`, `SdkDependencyDelta`, `SdkDependency` types.
- [ ] Task 5.3: Write test: `TraceConfigChange` serialization roundtrip.
- [ ] Task 5.4: Write test: `SdkDependencyDelta` default is empty.

### Phase 6: Final Validation
- [ ] Task 6.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Task 6.2: Run `cargo test coverage` — all tests pass.
- [ ] Task 6.3: Run full `cargo test` — no regressions.
- [ ] Task 6.4: Verify `[coverage.traces].enabled = false` → both functions return empty.
