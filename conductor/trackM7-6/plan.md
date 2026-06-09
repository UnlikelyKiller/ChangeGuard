## Plan: Track M7-6 — ADR Staleness Detection

### Phase 1: Staleness Computation
- [ ] Task 1.1: Implement `compute_staleness()` in `src/retrieval/query.rs`.
- [ ] Task 1.2: Implement multi-source age detection: mtime → frontmatter `date:` → `created:` metadata → git fallback.
- [ ] Task 1.3: Implement recently-updated exemption (mtime within 30 days).
- [ ] Task 1.4: Implement severity tiers: Warning (365-730 days), Critical (>730 days).
- [ ] Task 1.5: Write test: ADR mtime > threshold → staleness populated.
- [ ] Task 1.6: Write test: ADR mtime < threshold → staleness is None.
- [ ] Task 1.7: Write test: ADR with frontmatter `date:` uses frontmatter date.
- [ ] Task 1.8: Write test: ADR edited 5 days ago → exempt.
- [ ] Task 1.9: Write test: ADR > 730 days → Critical tier.

### Phase 2: Type Extension
- [ ] Task 2.1: Add `staleness_days: Option<u32>` to `RelevantDecision`.
- [ ] Task 2.2: Add `staleness_tier: Option<StalenessTier>` to `RelevantDecision`.
- [ ] Task 2.3: Define `StalenessTier` enum with `Ord` derive.
- [ ] Task 2.4: Write test: serialization roundtrip with staleness fields populated.
- [ ] Task 2.5: Write test: serialization roundtrip with staleness fields as None.

### Phase 3: Retrieval Integration
- [ ] Task 3.1: Call `compute_staleness()` for each matched ADR in retrieval pipeline.
- [ ] Task 3.2: Add staleness annotation to ask context block.
- [ ] Task 3.3: Write test: ask context includes staleness warning when present.
- [ ] Task 3.4: Write test: ask context omits staleness when None.

### Phase 4: Human Output
- [ ] Task 4.1: Add staleness tier display in human output "Relevant Documentation" section.
- [ ] Task 4.2: Write test: human output shows staleness tier when Critical.

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Task 5.2: Run `cargo test retrieval` — all tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions.
- [ ] Task 5.4: Verify `[coverage.adr_staleness].enabled = false` → no staleness computed.
