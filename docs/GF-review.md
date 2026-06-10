# GF Milestone Review

**Reviewed:** 2026-06-10  
**Tracks:** GF1–GF8 (God-File Decomposition and Boundary Hardening)  
**Reviewer:** Claude Sonnet 4.6

---

## Verdict

**Functionally complete, procedurally incomplete.** All eight decompositions compiled cleanly, all 830 unit tests and 230 integration tests pass, and clippy is clean. However, none of the standard finalization steps were completed — conductor registry, plan checklists, and ledger provenance are all stale — and several tracked temp files were not removed. A cleanup pass is needed before the milestone can be marked done.

---

## Test Gate

| Gate | Result |
|---|---|
| `cargo clippy --all-targets --all-features -- -D warnings` | **PASS** |
| `cargo nextest run --lib --bins --workspace` | **PASS** (830 tests) |
| `cargo nextest run --test integration --test-threads=1` | **PASS** (230 tests, 3 skipped) |
| Ledger status | Clean (0 pending, 0 unaudited drift) |

---

## Findings

### F1 — Critical: conductor.md status not updated (all 8 tracks)

`conductor/conductor.md` still shows `Status: Planning` for every GF track, and the milestone heading still reads `(Planning)`. This is the authoritative status registry for the project — it needs to reflect completed work.

**Fix:** Update `conductor.md` to set `Status: Completed` for GF1–GF8 and change the milestone heading to `(Completed)`.

---

### F2 — Critical: plan.md checklists not checked off (all 8 tracks)

Every task in every GF plan file is still `- [ ]` (0 of ~28–31 tasks checked per track). The standard finalization step ("Mark tasks as `- [x]`") was skipped.

Files affected:
- `conductor/trackGF1/plan.md` — 31 unchecked tasks
- `conductor/trackGF2/plan.md` — 28 unchecked tasks
- `conductor/trackGF3/plan.md` — 30 unchecked tasks
- `conductor/trackGF4/plan.md` — 29 unchecked tasks
- `conductor/trackGF5/plan.md` — 31 unchecked tasks
- `conductor/trackGF6/plan.md` — 30 unchecked tasks
- `conductor/trackGF7/plan.md` — 30 unchecked tasks
- `conductor/trackGF8/plan.md` — 27 unchecked tasks

**Fix:** Mark all completed task rows as `- [x]` in each plan file.

---

### F3 — High: 24 temp review artifacts still tracked in git

The GF5 commit accidentally included `.agents/review_*.md`, `.codex/review_*.md`, and `output/` files. A partial cleanup was done in `bccb7d2` but left 24 files tracked:

```
.agents/review_gf1_round5.md   through   .agents/review_gf4_round1.md   (11 files)
.codex/review_gf1_round5.md    through   .codex/review_gf4_round1.md    (11 files)
output/final_graph.html
output/review_gf5.md
```

These are intermediate review transcripts and generated output — they belong in `.gitignore` or should be removed from the index.

**Fix:**
```powershell
git rm --cached .agents/review_gf*.md .codex/review_gf*.md output/final_graph.html output/review_gf5.md
```
Then add `output/` and `.agents/review_*.md` / `.codex/review_*.md` to `.gitignore`.

---

### F4 — Medium: `output/` directory not in `.gitignore`

The `.gitignore` does not exclude the `output/` directory. This directory is used for generated graphs, codex prompts, and batch scripts. Any future `changeguard viz` or review tooling run from the repo root will produce tracked files unless it is excluded.

**Fix:** Add `output/` to `.gitignore`.

---

### F5 — Medium: GF3 phases not extracted into separate files

The GF3 spec goal was to break `build_native_graph` into "explicit graph loading phases with testable helpers." The implementation added a `GraphLoadContext` struct and nine `phase_*` private functions, which is a real improvement. However:

- `src/index/graph_loader.rs` is still **1,501 lines** (up from ~1,300 before the track).
- All phase functions (`phase_files`, `phase_symbols`, `phase_routes`, etc.) are private and cannot be unit-tested directly.
- The spec listed "phase-level tests protect deleted-node pruning and incremental behavior" as a definition-of-done requirement. No per-phase unit tests were added.

**Current state:** The refactoring improved readability via named phases and the `GraphLoadContext` carrier, which satisfies the spirit of the work. The letter of the spec (separately testable phases) was not met.

**Recommendation:** Either add `#[cfg(test)]` unit tests that directly invoke the `phase_*` helpers (requires making them `pub(crate)` or `pub(super)`) or document in the plan why the current organization meets the DoD.

---

### F6 — Medium: `new_for_worker` uses bare `.unwrap()` on `Connection::open_in_memory()`

`src/index/orchestrator.rs:94`:

```rust
storage: StorageManager::init_from_conn(
    rusqlite::Connection::open_in_memory().unwrap(),
),
```

`open_in_memory()` is infallible in practice (SQLite in-memory never fails to open), but the codebase rule is **no `unwrap()` in production code**. This call was introduced by GF6.

**Fix:**
```rust
storage: StorageManager::init_from_conn(
    rusqlite::Connection::open_in_memory()
        .expect("SQLite in-memory open is infallible"),
),
```
Or propagate with `into_diagnostic()?` if the surrounding context allows it.

---

### F7 — Low: `#[allow(clippy::too_many_arguments)]` on two output helpers in `commands/index.rs`

GF7 extracted index mode handlers but left two output formatting functions — `print_json_output` and `print_human_output` — with 15 parameters each, both suppressed with `#[allow(clippy::too_many_arguments)]`.

```
src/commands/index.rs:428  #[allow(clippy::too_many_arguments)]
src/commands/index.rs:536  #[allow(clippy::too_many_arguments)]
```

A simple `IndexOutputStats` struct collecting all the `*Stats` fields would eliminate both suppressions and make call sites more readable.

**Severity:** Low — functional, clippy is silenced, not a correctness issue. Worth cleaning up in a follow-on pass.

---

### F8 — Low: `rows` module re-exported through `orchestrator` facade

`src/index/orchestrator.rs:65–68` re-exports row helpers from the sibling `crate::index::rows` module:

```rust
pub use crate::index::rows::{
    delete_file_index_dependents, delete_file_symbols, get_file_id_by_path, insert_file_row,
    insert_symbol_row, upsert_file_row,
};
```

Both modules are `pub` in `src/index/mod.rs`. Callers can reach these functions through two paths (`crate::index::rows::*` or `crate::index::orchestrator::*`). No behavior issue, but it adds surface area and can confuse future readers about where the canonical definition lives.

**Recommendation:** Remove the re-export from `orchestrator.rs` and let callers import directly from `crate::index::rows`. Audit call sites to update any that currently use the `orchestrator` path.

---

## Per-Track Summary

| Track | Files | Tests | Clippy | conductor.md | plan.md | Notes |
|---|---|---|---|---|---|---|
| GF1 | `src/impact/packet/` (8 submodules + facade) | Pass | Clean | **Planning** | **0/31** | `serialization` correctly not re-exported |
| GF2 | `src/config/model/` (8 submodules + facade) | Pass | Clean | **Planning** | **0/28** | Clean split |
| GF3 | `src/index/graph_loader.rs` (phases in-file) | Pass | Clean | **Planning** | **0/30** | **F5**: phases private, not separately testable |
| GF4 | `src/ledger/db/` (8 submodules + facade) | Pass | Clean | **Planning** | **0/29** | `LedgerDb` facade preserved correctly |
| GF5 | `src/cli/{args,dispatch,mod}.rs` | Pass | Clean | **Planning** | **0/31** | **F3**: 24 temp files in commit; CLI contract intact |
| GF6 | `src/index/orchestrator/` (8 submodules + rows) | Pass | Clean | **Planning** | **0/30** | **F6/F8**: bare unwrap, double re-export |
| GF7 | `src/commands/index.rs` (mode functions) | Pass | Clean | **Planning** | **0/30** | **F7**: 15-arg helpers with `#[allow]` |
| GF8 | `src/impact/analysis/dead_code/` (3 submodules) | Pass | Clean | **Planning** | **0/27** | Clean implementation |

---

## Required Before Marking GF Complete

1. **F1** — Update `conductor/conductor.md`: set all 8 GF tracks to `Status: Completed` and change milestone to `(Completed)`.
2. **F2** — Check off all tasks in all 8 `plan.md` files.
3. **F3** — Remove the 24 temp review artifacts from git tracking.
4. **F4** — Add `output/` to `.gitignore`.

## Recommended Follow-On (Non-Blocking)

5. **F5** — Add per-phase tests for `graph_loader.rs` phase functions, or document why the current coverage is sufficient.
6. **F6** — Replace `open_in_memory().unwrap()` in `new_for_worker` with a safe alternative.
7. **F7** — Introduce `IndexOutputStats` struct to eliminate the two `#[allow]` suppressions.
8. **F8** — Remove the `rows` re-export from `orchestrator.rs`.
