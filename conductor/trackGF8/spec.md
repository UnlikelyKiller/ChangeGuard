# Track GF8: Dead-Code Analysis Provider Boundary Tightening

## Objective

Continue the provider-pattern direction established in Track R1-2 (the `ImpactProvider` registry) in `src/impact/analysis/dead_code.rs` by splitting dead-code evidence collection, confidence scoring, filtering, and report construction into focused modules. The file already implements `DeadCodeImpactProvider` (an `ImpactProvider`) and a `ConfidenceScorer`, plus a tests module — so the goal is boundary clarity inside an existing pattern, not introducing one. The user-supplied analysis ranks this lower than the other files because it already has some decomposition and tests, but it remains a complex ~900-line analysis domain.

## Evidence

- User analysis ranks `src/impact/analysis/dead_code.rs` as refactor need 5/10.
- The file has meaningful test coverage already, so the goal is boundary clarity and false-positive protection rather than wholesale redesign.
- The R1-2 decomposition established the `ImpactProvider` registry pattern that should be extended instead of replaced.
- Boundary notes (verified 2026-06-09): the result types `DeadCodeFinding` and `ConfidenceFactor` live in `src/impact/packet.rs` (GF1's domain — coordinate the destination module rather than double-moving), and a separate enrichment provider exists at `src/impact/enrichment/dead_code.rs`; this track must not blur the line between the analysis provider and that enrichment provider.

## Scope

Required module boundaries:

- `evidence`: graph reachability, symbol visibility, git activity, test references, and ignore/config evidence.
- `scoring`: confidence score calculation and reason weighting.
- `filters`: exclusions for public API, generated code, tests, examples, fixtures, migrations, and configured ignore paths.
- `report`: human and JSON output assembly if currently mixed into analysis.
- `types`: dead-code candidate, evidence, reason, and confidence types if they are local to this domain.
- Preserve current command behavior and confidence thresholds.

## Non-Goals

- Do not change the default confidence threshold.
- Do not introduce automatic deletion or rewrite suggestions.
- Do not change graph schema.
- Do not remove existing tests.

## Implementation Notes

- Add characterization tests before moving scoring logic.
- Keep deterministic ordering of candidates and reasons.
- Prefer explicit confidence reasons over opaque score changes.
- Treat new false positives as blockers.

## Verification Strategy

Targeted:

- `cargo test impact::analysis::dead_code`
- CLI smoke for `changeguard dead-code --threshold 0.75`.
- Integration tests for dead-code empty and non-empty fixtures if present.

Final:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- Evidence, scoring, filtering, and report logic are separately owned.
- Existing tests remain green and focused tests protect scoring/filtering behavior.
- Candidate ordering and confidence values remain stable for fixtures.
- No new false-positive deletion recommendations are introduced.
- Final verification and reinstall pass.

## Risks

- Small scoring changes can alter recommendations even when compile/tests pass.
- Filtering generated/test code is easy to regress.
- Graph reachability data may be absent in temp fixtures, so tests must cover both populated and empty graph states.
