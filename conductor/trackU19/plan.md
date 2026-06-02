# Track U19 Plan: Data-Driven config verify Section Table

- [x] Task U19.1: Create `src/commands/config_verify.rs` with `ConfigSection` trait, `ConfigRow`, `ValueSource` enum.
- [x] Task U19.2: Implement `BackendSection` as a `ConfigSection`.
- [x] Task U19.3: Implement `SemanticSection` as a `ConfigSection` (handles U13 + U15 + U16 outputs).
- [x] Task U19.4: Implement `all_sections()` and `render_verify_report()`.
- [x] Task U19.5: Add `pub mod config_verify;` to `src/commands/mod.rs`.
- [x] Task U19.6: Refactor `src/commands/config.rs::execute_config_verify` to use the new renderer.
- [x] Task U19.7: Add `--json` and `--section=<name>` flags to the `ConfigVerify` subcommand in `src/cli.rs`.
- [x] Task U19.8: Write 5 unit tests in `config_verify.rs` (red phase first, then implement).
- [x] Task U19.9: Run CI gate.
- [x] Task U19.10: Manual: default output byte-identical to U14 baseline; `--json` produces valid JSON; `--section=semantic` filters correctly.
- [x] Task U19.11: Ledger provenance + commit + push.
