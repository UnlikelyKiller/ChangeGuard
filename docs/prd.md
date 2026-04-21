# ChangeGuard PRD

ChangeGuard is a local-first CLI that turns repository changes into explicit risk, verification, and Gemini-ready context.

## Primary user outcome

Given a repo with pending edits, a developer should be able to:

1. initialize local state under `.changeguard/`
2. inspect environment readiness with `doctor`
3. summarize current changes with `scan`
4. generate a structured impact packet with `impact`
5. inspect complexity/temporal hotspots with `hotspots`
6. run predictive targeted verification with `verify`
7. ask Gemini for bounded assistance with `ask`
8. optionally use sibling repository schemas through `federate`
9. optionally use LSP diagnostics through the daemon feature

## Product constraints

- local-first by default
- deterministic outputs where practical
- Windows 11 + PowerShell first
- safe rebuildability of generated state
- explicit recovery path through `changeguard reset`
- visible degradation instead of silent fallback for Phase 2 intelligence
- secret redaction before persisted impact state or Gemini prompt submission

## Current feature set

- Git scan and watch batching
- Symbol, import/export, runtime, and complexity extraction for Rust, TypeScript, and Python
- Temporal coupling from git history, first-parent by default
- Hotspot ranking from normalized complexity and change frequency
- Predictive verification from current imports, packet history, and temporal coupling
- Federated sibling schema export, scan, dependency discovery, and cross-repo impact checks
- Gemini analyze/suggest/review/narrative prompting with sanitization and fallback artifacts
- Optional LSP daemon with diagnostics, Hover, CodeLens, read-only SQLite access, and lifecycle tests
- Reset/recovery for derived `.changeguard/` state
