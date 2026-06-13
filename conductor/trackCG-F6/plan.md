# Track CG-F6 Plan

## Phase 1: Default Trait Exclusion
- [x] Add an `--include-traits` boolean flag to the `dead-code` CLI parser (`src/cli/args.rs` or equivalent).
- [x] In the dead-code analysis logic, filter out standard Rust trait names (`Eq`, `Ord`, `PartialOrd`, `Default`, `Clone`, `Debug`, `Serialize`, `Deserialize`) if `--include-traits` is false.

## Phase 2: Confidence Penalties
- [x] Locate the logic where the final confidence score is calculated.
- [x] Add a suffix check to apply a penalty (-0.20) to symbols ending in `Provider`, `Result`, `Chunk`, `Record`.

## Phase 3: UX Warning Hint
- [x] Locate the table rendering logic for the `dead-code` command.
- [x] Append the yellow `HINT:` text below the table (always shown, unconditionally).

## Phase 4: Integration Tests & Skill Update
- [x] Write integration tests to verify the `--include-traits` flag.
- [x] Write unit tests in `filters.rs` for `is_standard_trait` and `name_penalty`.
- [x] Write unit tests in `dead_code.rs` for filter path through `score_symbol`.
- [x] Update `C:\dev\changeguard\.agents\skills\changeguard\SKILL.md` to document the new flag and explain the false-positive heuristics.

## Phase 5: Finalization
- [x] Run `changeguard verify` — passed (875/875 lib tests, 255/255 integration tests).
- [x] Update conductor status to Completed.
