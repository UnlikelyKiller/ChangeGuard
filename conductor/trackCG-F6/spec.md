# Track CG-F6: Dead-Code False Positive Reduction

## Overview
The `changeguard dead-code` command currently flags standard derived traits (`Eq`, `Ord`), dynamic trait providers (`CIPredictorProvider`), and generic data structs (`RetrievedChunk`) as dead code because they are used implicitly, via reflection, or dynamically, avoiding static reachability paths. This creates too much noise for developers.

## Requirements

1. **Default Trait Exclusion & Flag**:
   - By default, the `changeguard dead-code` command must completely filter out standard trait implementations (e.g., `Eq`, `Ord`, `PartialOrd`, `Default`, `Clone`, `Debug`, `Serialize`, `Deserialize`) from the results table.
   - Add a new `--include-traits` CLI flag to disable this filter and show all findings.

2. **Suspicious Name Penalties**:
   - Apply a dynamic penalty (e.g., -0.20 to -0.30) to the final `Confidence` score of a symbol if it matches common patterns for serialized data or dynamically dispatched services (e.g., ends in `Provider`, `Result`, `Chunk`, or `Record`).
   - This ensures these highly-likely false positives fall below the standard 0.75 threshold without fully hiding them if they sit idle for years.

3. **UX Hint**:
   - At the bottom of the dead-code table output, print a yellow proactive hint:
     `HINT: Derived traits, serialization structs, and dynamically dispatched trait objects are often falsely flagged as dead code due to implicit usage.`

4. **Skill Update**:
   - Update `C:\dev\changeguard\.agents\skills\changeguard\SKILL.md` to document the new `--include-traits` flag and advise subagents on the heuristic nature of dead code analysis.

## Definition of Done
- `changeguard dead-code` omits standard traits like `Eq` and `Ord` by default.
- `changeguard dead-code --include-traits` successfully displays them.
- `*Provider` and `*Chunk` types receive a confidence score penalty.
- The `HINT:` is visibly printed below the table.
- `tests/integration/cli_dead_code.rs` (or equivalent) verifies the new `--include-traits` flag and the hint.
- `SKILL.md` for ChangeGuard is updated.
- `changeguard verify` passes and the ledger transaction is committed.
