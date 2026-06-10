# Track GF1: Impact Packet Domain Type Split

## Objective

Decompose `src/impact/packet.rs` without changing the public impact-packet contract. This file is the highest-coupled type surface in the repo: the user-supplied analysis reports 2064 total lines, 1094 code lines, 72 functions, 32 structs, 51 impl blocks, and imports from 65 files. Any edit here has broad compile and schema risk, so the first refactor must prioritize compatibility and behavior preservation over aggressive cleanup.

## Evidence

- User analysis ranks `src/impact/packet.rs` as refactor need 10/10 due to type density and 65-file fan-in.
- `changeguard scan --impact` on the clean tree reported no active diff and risk `low`.
- `.changeguard/reports/latest-impact.json` identified current top hotspots, which means packet work must be verified beyond hotspot-only signals because packet risk is mostly fan-in/schema coupling rather than recent churn.
- Existing conductor history includes R1 impact refactors and W-surface packet extensions; this track should preserve those surfaces instead of reopening feature semantics.

## Scope

Required module shape:

- Keep `src/impact/packet.rs` as the public compatibility facade during this track.
- Create focused modules under `src/impact/packet/` or an equivalent local module tree. The list below was corrected against the actual type inventory on 2026-06-09 — packet.rs defines **no** security types, and observability signal types (`ObservabilitySignal`) are imported from `src/observability/signal.rs` and must stay there. Do not create empty `security.rs`/`observability.rs` modules.
  - `metadata.rs`: `ImpactPacket`, schema version constants, packet construction defaults, and the `finalize`/truncate/`escalate_risk` helpers. (Avoid the module name `core` — it invites path confusion with the `core` crate in `use` statements.)
  - `changed_file.rs`: `ChangedFile`, `AnalysisStatus`, `FileAnalysisStatus`, and file classification helpers.
  - `risk.rs`: `RiskLevel`, `RiskImpact`, `TemporalCoupling`, `StructuralCoupling`, `CentralityRisk`.
  - `verification.rs`: `VerificationResult`, `CIGate`, `CIPrediction`, `CiConfigChange`.
  - `coverage.rs`: `CoverageDelta`, `TestCoverage`, `CoveringTest`, `CallChain`/`CallChainNode`, `RuntimeUsageDelta`, `DataFlowMatch`, `TraceConfigChange`/`TraceEnvVarChange`/`TraceConfigType`, `SdkDependencyDelta`/`SdkDependency`, `DeployManifestChange`/`ManifestType`.
  - `surfaces.rs`: `DataModel`, `ApiRoute`, `ServiceMapDelta`, `Service`.
  - `intelligence.rs`: `RelevantDecision`, `StalenessTier`, `Hotspot`, `AiInsight`, `KGImpact`, `DeadCodeFinding`, `ConfidenceFactor`.
  - `serialization.rs`: serde defaults, legacy aliases, schema-stability helpers.
- Run a full `pub struct`/`pub enum` inventory of packet.rs before starting and assign every type a home; the list above is the verified starting point, not a cap.
- Re-export existing public names from `src/impact/packet.rs` so current imports keep compiling.
- Move tests into focused module tests where practical, but keep existing fixture assertions intact.
- Do not rename JSON fields, remove serde defaults, or alter `ImpactPacket` field order unless tests prove the output contract is unchanged.

## Non-Goals

- Do not change risk scoring behavior.
- Do not redesign the impact packet schema.
- Do not migrate call sites away from compatibility re-exports in the same track unless a local edit is needed for compilation.
- Do not touch `.changeguard` state files.

## Implementation Notes

- Start with pure moves and re-exports, then run `cargo check --all-targets --all-features` before any cleanup.
- If constructors or defaults are entangled, extract helpers after the move rather than before it.
- Treat serde defaults as part of the API. Add tests before changing any default function.
- Preserve deterministic sorting of packet collections.
- Write the schema-stability golden test **before** any type moves, not after — it is the characterization safety net for the whole track. Dev-dependencies are limited to `tempfile` and `httpmock` (no `insta`); implement it as a plain `serde_json` golden assertion over a fully-populated packet.
- Coordinate with GF8: `DeadCodeFinding` and `ConfidenceFactor` are consumed by `src/impact/analysis/dead_code.rs`. If GF8 runs first or concurrently, agree on the destination module to avoid a double move.

## Verification Strategy

Targeted:

- `cargo test impact::packet`
- `cargo check --all-targets --all-features`
- Existing integration tests that parse `scan --impact --json` or `impact --json`.

Final:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `src/impact/packet.rs` is a small facade or near-facade with domain modules holding the moved types.
- All existing public type paths continue to compile.
- Packet JSON emitted by representative `scan --impact --json` and `impact --json` tests remains schema-compatible.
- New or relocated tests make each domain module discoverable.
- Final verification and reinstall pass.

## Risks

- High fan-in can create broad compile failures from missed re-exports.
- Serde defaults and aliases can silently change JSON behavior if moved carelessly.
- Tests inside the original file may rely on private helper visibility; move them incrementally.
