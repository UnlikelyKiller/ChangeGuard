# Changeguard V1 Engineering Principles Review

## Scope

This review evaluates **Changeguard Implementation Plan v1** against the following constraints:

* **Follow SRP**: Keep modules focused on one task.
* **Be Idiomatic Rust**: Use `Result` and `match`, avoid `unwrap`.
* **Stay KISS/YAGNI**: Don't build the abstraction until it's needed.
* **Prefer Determinism**: Ensure outputs are predictable and testable.
* **Favor Error Visibility**: Use `anyhow`/`miette` to provide actionable user errors.

---

## Overall Verdict

**Status: Mostly aligned, but needs tightening in a few places before implementation.**

The v2 plan is directionally strong. It already leans toward deterministic behavior, conservative defaults, and good module boundaries. The main risk areas are:

1. a few modules are still broad enough that implementers could violate SRP
2. the plan still leaves room for premature abstraction in platform/state/index layers
3. the plan should state Rust error-handling expectations more explicitly
4. the plan should more aggressively discourage “smart” fallbacks that reduce determinism

---

## 1. SRP Review

### Current Strengths

The v2 plan already separates major concerns better than most implementation plans. In particular, it separates:

* CLI routing
* platform handling
* git scanning
* watching/debounce
* indexing
* impact scoring
* policy evaluation
* verification
* Gemini wrapping

That is a good foundation.

### Remaining Risks

A few modules are still broad enough to invite kitchen-sink implementations:

#### A. `platform/`

Risk:

* `detect`, `shell`, `paths`, `env`, and `process_policy` are fine as a split, but implementers may still start putting general subprocess behavior and validation logic there.

Recommendation:

* treat `platform/` strictly as environment-specific normalization and detection only
* keep generic process spawning in `util/process.rs` or `verify/runner.rs`
* keep business decisions out of `platform/`

#### B. `index/`

Risk:

* `symbols`, `references`, `runtime_usage`, `normalize`, and `storage` could become a catch-all semantic engine.

Recommendation:

* keep `symbols` limited to declaration extraction
* keep `references` limited to lightweight file-local or parser-derived relationships
* keep `runtime_usage` limited to env/config/runtime access detection
* do not let `index/` become “global program intelligence” in v1

#### C. `state/`

Risk:

* `db`, `migrations`, `reports`, and `locks` are good, but implementers may mix layout, persistence, recovery, and report formatting.

Recommendation:

* `layout` should only know paths and directory structure
* `db` should only know persistence API
* `migrations` should only know schema upgrades
* `reports` should only know report read/write
* `reset` logic should remain in the command layer or a narrow recovery helper

#### D. `impact/`

Risk:

* `relationships`, `reasoning`, and `score` could blur together.

Recommendation:

* `relationships` computes input facts only
* `score` assigns tier/weights only
* `reasoning` formats human-readable explanations only

### SRP Verdict

**Pass with conditions.** The plan is structurally good, but should add stricter module role statements to reduce implementation drift.

---

## 2. Idiomatic Rust Review

### Current Strengths

The plan already prefers:

* Rust-first implementation
* explicit diagnostics
* bounded subprocess handling
* deterministic local state

Those are compatible with idiomatic Rust.

### Missing Explicitness

The plan should say this plainly:

* public fallible functions should return `Result<T, E>`
* user-facing command handlers should return `miette::Result<()>` or an equivalent top-level diagnostic result
* internal libraries may use `anyhow::Result<T>` for app-level composition where typed errors are not worth the complexity
* `unwrap`, `expect`, and unchecked assumptions should be forbidden in production code except in tests or impossible-by-construction cases that are documented

### Recommended Addition

Add a short implementation rule section:

* Prefer `Result` propagation with `?`
* Use `match` for branch clarity when multiple failure modes need user-visible handling
* Use `Option` only when absence is expected and non-exceptional
* Convert lower-level errors into actionable command-level diagnostics with context
* Avoid panics in normal runtime paths

### Idiomatic Rust Verdict

**Needs explicit reinforcement.** The plan is compatible with idiomatic Rust, but it should say so directly to prevent sloppy agent-generated code.

---

## 3. KISS / YAGNI Review

### Current Strengths

The v2 plan does a lot right here:

* rejects Python as required runtime for v1
* rejects MCP/server/cloud architecture for v1
* rejects whole-program overanalysis in v1
* rejects autonomous git flows in v1
* introduces a basic impact-packet phase early

That is exactly the right shape.

### Remaining Risks

There are still a few places where implementers may overbuild.

#### A. SQLite too early for every concern

The plan now formalizes SQLite, which is fine, but v1 should not force every internal datum into the DB immediately.

Recommendation:

* use JSON file reports first where simpler
* only move data into SQLite when there is a concrete durability/querying need
* keep DB usage narrow in early phases

#### B. Locks subsystem

`state/locks.rs` may be unnecessary early unless concurrent command execution becomes a real issue.

Recommendation:

* mark locking as conditional
* do not build an elaborate lock manager before a real race exists

#### C. `process_policy.rs`

This might be premature if it becomes its own abstraction layer before process execution patterns stabilize.

Recommendation:

* keep a minimal command execution policy model at first
* only split dedicated process-policy logic once at least two subsystems need it

#### D. Reference analysis

The plan should explicitly prohibit building repo-wide call graph ambitions in v1.

Recommendation:

* say that file-local and changed-file-adjacent analysis is sufficient for v1

### KISS/YAGNI Verdict

**Mostly pass, but tighten anti-overengineering instructions.** The biggest remaining risk is implementers using the plan as permission to build clever infrastructure too early.

---

## 4. Determinism Review

### Current Strengths

This is one of the strongest parts of the plan.

The v2 plan already emphasizes:

* deterministic risk scoring
* targeted verification
* explainable reasoning
* inspectable reports
* JSON packet output
* graceful degradation
* stable phase boundaries

That is excellent.

### Remaining Gaps

The plan should state these determinism requirements more concretely:

#### A. Stable ordering

All emitted file lists, symbol lists, reasons, commands, and report sections should be sorted deterministically.

#### B. Stable packet schema

The impact packet should have a versioned schema and stable field order in tests.

#### C. No silent fallback heuristics

If a parser fails, the tool should record partial results explicitly rather than quietly inventing replacement behavior.

#### D. Deterministic default verification plans

Given the same repo state and config, the verification plan must always be identical.

#### E. Clock sensitivity

Timestamps should not be embedded into comparison-sensitive test fixtures unless explicitly normalized.

### Recommended Addition

Add a “determinism contract” section:

* sort outputs before presentation/persistence where possible
* version packet schemas
* never suppress parse or scan failure silently
* annotate partial data explicitly
* normalize volatile fields in tests

### Determinism Verdict

**Strong pass with a few useful hardening additions.**

---

## 5. Error Visibility Review

### Current Strengths

The plan already names `anyhow` and `miette`, and it repeatedly asks for clear diagnostics and graceful failure.

### What Is Missing

It should define where each belongs.

Recommended rule of thumb:

* `miette` at command boundaries and user-visible diagnostics
* `anyhow` for internal orchestration where rich user display is not needed yet
* `thiserror` only for stable internal error enums when that adds clarity

The plan should also require:

* command errors must explain what failed, why it matters, and what the user can do next
* errors should name the path, command, or dependency involved when safe to do so
* missing tools should produce actionable setup guidance
* config parse errors should include file path and failing key when feasible
* verification failures should distinguish command failure from tool-not-found from timeout

### Example Quality Bar

Bad:

* “Failed to verify”

Good:

* “Verification command `cargo test watcher` exited with status 101 in package `changeguard`. Review stderr summary below. Full output saved to `.changeguard/reports/latest-verify.json`."

### Error Visibility Verdict

**Pass, but it needs a sharper operational standard.**

---

## 6. Concrete Improvements Recommended for V2

### Add a new section: Rust Implementation Rules

Include:

* no `unwrap`/`expect` in production paths
* prefer `Result` + `?`
* use `match` for explicit branching on expected failure modes
* `miette` for command/user-facing diagnostics
* `anyhow` for internal composition

### Add a new section: Determinism Contract

Include:

* stable sorting for emitted collections
* stable packet schema versioning
* explicit partial-result annotation
* no silent fallback behavior
* normalized test fixtures for volatile fields

### Tighten SRP in module descriptions

Explicitly constrain:

* `platform/` to platform adaptation only
* `index/` to changed-file intelligence only
* `state/` to persistence/layout/report storage only
* `impact/` to fact assembly, scoring, and explanation only

### Tighten KISS/YAGNI wording

Add explicit “do not build yet” statements for:

* lock manager sophistication
* repo-wide call graphing
* generalized plugin systems
* abstraction layers with only one implementation
* DB-first design where flat-file state is enough in early phases

### Tighten error expectations

Require actionable errors to include:

* what failed
* where it failed
* likely cause when known
* next step for the user

---

## 7. Final Verdict by Principle

### Follow SRP

**Verdict: Pass with tightening needed**
The structure is good, but several modules need narrower role statements.

### Be Idiomatic Rust

**Verdict: Partial pass**
Compatible with idiomatic Rust, but the plan should explicitly ban `unwrap` in production paths and require `Result`-driven error propagation.

### Stay KISS/YAGNI

**Verdict: Mostly pass**
Good overall restraint, but needs firmer warnings against premature DB, locking, and semantic-engine abstractions.

### Prefer Determinism

**Verdict: Strong pass**
One of the best parts of the plan. It should still add a stable ordering/schema/testing contract.

### Favor Error Visibility

**Verdict: Pass with sharpening**
The intent is good, but the expected structure of actionable errors should be spelled out more explicitly.

---

## Recommended Disposition

**Adopt v2, but revise it once more before implementation to add:**

1. Rust implementation rules
2. Determinism contract
3. tighter SRP boundaries
4. stronger YAGNI guardrails
5. more explicit error quality requirements
