# Plan: Track 9 - Change Risk Analysis Engine

### Phase 10: Change Risk Analysis
- [ ] Task 9.1: Define internal scoring models in `src/impact/analysis.rs`.
- [ ] Task 9.2: Implement `ProtectedPath` heuristic.
  - [ ] Use `ProtectedPathChecker` to identify hits.
  - [ ] Assign weight and record reasons.
- [ ] Task 9.3: Implement `ChangeVolume` heuristic.
  - [ ] Count files and symbols in the `ImpactPacket`.
- [ ] Task 9.4: Implement `SymbolVisibility` heuristic.
  - [ ] Iterate through extracted symbols and check `is_public`.
- [ ] Task 9.5: Implement `analyze_risk` main loop.
  - [ ] Aggregate weights.
  - [ ] Map to `RiskLevel`.
  - [ ] Finalize `risk_reasons`.
- [ ] Task 9.6: Update `impact` command to call `analyze_risk`.
- [ ] Task 9.7: Add unit tests in `src/impact/analysis.rs` for each scoring factor.
- [ ] Task 9.8: Add integration tests in `tests/risk_analysis.rs`.
- [ ] Task 9.9: Final verification with `cargo test -j 1 -- --test-threads=1`.
