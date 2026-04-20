# Research: Breaking Changes (Post-April 2024)

This document tracks significant breaking changes, major version bumps, and API shifts for ChangeGuard's core dependencies since April 2024.

## Summary Table

| Dependency | Current Version | Notable Breaking Changes (since 04/2024) | Impact Level |
| :--- | :--- | :--- | :--- |
| **thiserror** | 2.0.x | **v2.0.0 (Nov 2024)**: Major version bump for `no_std` support. | Moderate |
| **rusqlite** | 0.39.0 | **v0.39.0 (Mar 2026)**: Breaking changes in statement preparation and type impls. | Moderate |
| **clap** | 4.6.1 | v5.0.0 is in development (unstable-v5); v4.x remains stable. | Low (Planning) |
| **miette** | 7.6.0 | v7.0.0 released Feb 2024; v7.x stable since then. | Low |
| **tree-sitter** | 0.26.8 | Transition to v0.26.x (late 2025/2026) with CLI and API cleanups. | Low |
| **gix** | 0.81.0 | High-velocity releases, but v0.x semver avoids major breakage. | Low |
| **notify-debouncer-full** | 0.7.0 | v0.8.0-rc.1 in progress; v0.7.x stable. | Low |
| **tower-lsp-server** | 0.23.0 | **v0.23.0 (Phase 2)**: Migration from `lsp-types` to `ls-types`. | Moderate |
| **linfa** | 0.8.1 | **v0.8.1 (Phase 2)**: Breaking `ndarray` 0.16 upgrade. | Moderate |
| **ChangeGuard CLI** | Phase 2 | **Behavioral**: Probabilistic verification ordering; new subcommands. | Moderate |

---

## Detailed Findings

### Phase 2: Intelligence & Federation Shifts
The following breaking changes are introduced as part of the Phase 2 implementation plan to support temporal analysis, complexity scoring, and LSP integration.

#### CLI & Behavioral
- **Probabilistic Verification (Moderate):** The `verify` command now reorders execution based on historical failure data. The output sequence is no longer deterministic relative to input order, though it remains deterministic relative to the SQLite state.
- **Subcommand Additions (Low):** Addition of `hotspots`, `daemon`, and `export-schema`.

#### API & Dependencies
- **tower-lsp-server (v0.23.0):**
    - **Namespace Shift:** Migration from `lsp-types` to `ls-types` (community fork).
    - **Trait Update:** `LanguageServer` no longer uses `#[async_trait]`.
- **linfa (v0.8.1):**
    - **Numerical Backend:** Upgraded to `ndarray` 0.16. Version alignment is required across all crates using `ndarray`.
- **rusqlite (v0.39.0):**
    - **Statement Safety:** Multi-statement strings are now rejected in `execute()`.
    - **Integer Types:** `u64`/`usize` support is disabled by default; all database integrations must shift to `i64`.

#### Configuration & Schema
- **Impact Packet Schema (Moderate):** Introduction of `schema_version`. Phase 1 packets lack this field and may require migration for Phase 2 tools.
- **Configuration Namespaces (Low):** New `temporal.*` and `metrics.*` keys added to `config.toml`.
- **Database Migrations (High):** Phase 2 introduces tables for `temporal_coupling`, `probability_history`, and `federated_cache`. Databases migrated to Phase 2 are not backward-compatible with Phase 1 binaries.

### thiserror (v2.0.0)
- **Release Date:** November 2024
- **Key Changes:**
    - **`no_std` Support:** The primary driver for v2.0 was official `no_std` support (requiring Rust 1.81+).
    - **Raw Identifiers:** Referencing keyword-named fields in format strings using raw identifiers (e.g., `{r#type}`) is no longer accepted. Use the unraw name (e.g., `{type}`) instead.
- **Impact on ChangeGuard:** Minimal, unless raw identifiers are used in error messages.

### rusqlite (v0.39.0)
- **Release Date:** March 2026
- **Key Changes:**
    - **Statement Validation:** `Connection::execute` now checks that there is no "tail" (remaining SQL) after the first statement.
    - **Multiple Statements:** `prepare` now checks for multiple statements to prevent accidental SQL injection or logic errors.
    - **Type Defaults:** Disabled `u64` and `usize` `ToSql`/`FromSql` implementations by default to avoid ambiguity with SQLite's signed 64-bit integers.
- **Impact on ChangeGuard:** Significant if using raw `execute` calls with multi-statement strings or relying on unsigned integer conversions.

### clap (v4.6.1)
- **Status:** v4.x is the current stable branch.
- **Future:** **v5.0.0** is in development (accessible via `unstable-v5` feature).
- **Recent Breaks (v4.x):** Mostly MSRV bumps (now 1.85 for v4.6.0+).
- **v5.0.0 Preview:** `ArgPredicate` is now `non_exhaustive`; default `term_width` behavior changes to "source format".
- **Impact on ChangeGuard:** Low, provided the project sticks to the stable v4.x features.

### tree-sitter (v0.26.x)
- **Release Period:** Late 2025 – Early 2026
- **Key Changes:**
    - CLI flag cleanup (e.g., removal of `--emit=lib`).
    - Deprecation of ABI 13 in favor of newer versions.
    - Experimental native QuickJS runtime for better performance.
- **Impact on ChangeGuard:** Moderate if using the CLI for grammar generation; low for library usage.

### gix (v0.81.0)
- **Status:** `gix` follows a rapid release cycle (v0.x). While it changes frequently, it avoids v1.0 stability, meaning "breaking" changes are frequent but usually scoped to specific sub-crates.
- **Impact on ChangeGuard:** High maintenance burden to keep up with the latest `gix` abstractions, but no "wall" of breaking changes since April 2024.

### miette (v7.6.0)
- **Status:** v7.0.0 was released just before the target window (Feb 2024). Since then, releases (up to 7.6.0) have focused on bug fixes and `clippy` lint elision.
- **Impact on ChangeGuard:** Stable.

### notify-debouncer-full (0.7.0)
- **Status:** Stable. v0.8.0-rc.1 is in pre-release but v0.7.0 remains the standard.
- **Impact on ChangeGuard:** None.
