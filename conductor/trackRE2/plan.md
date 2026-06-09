# Plan: Track RE2 (Modularize `src/impact/analysis/mod.rs`)

- [ ] 1. Define the `ImpactProvider` trait in `src/impact/analysis/mod.rs`.
- [ ] 2. Create `src/impact/analysis/git.rs`, `src/impact/analysis/dependencies.rs`, `src/impact/analysis/semantic.rs`, and `src/impact/analysis/temporal.rs`.
- [ ] 3. Incrementally move analysis logic from the monolith to these providers.
- [ ] 4. Refactor `ImpactOrchestrator` to loop through providers and aggregate results into `ImpactPacket`.
- [ ] 5. Ensure error handling and tracing context is maintained during provider execution.
- [ ] 6. Run comprehensive impact tests to ensure parity.
