# Track U19 Plan: Data-Driven config verify Section Table

- [ ] Task U19.1: Create `src/commands/config_verify.rs` with `ConfigSection` trait, `ConfigRow`, `ValueSource` enum.
- [ ] Task U19.2: Implement `BackendSection` as a `ConfigSection`.
- [ ] Task U19.3: Implement `SemanticSection` as a `ConfigSection` (handles U13 + U15 + U16 outputs).
- [ ] Task U19.4: Implement `all_sections()` and `render_verify_report()`.
- [ ] Task U19.5: Add `pub mod config_verify;` to `src/commands/mod.rs`.
- [ ] Task U19.6: Refactor `src/commands/config.rs::execute_config_verify` to use the new renderer.
- [ ] Task U19.7: Add `--json` and `--section=<name>` flags to the `ConfigVerify` subcommand in `src/cli.rs`.
- [ ] Task U19.8: Write 5 unit tests in `config_verify.rs` (red phase first, then implement).
- [ ] Task U19.9: Run CI gate.
- [ ] Task U19.10: Manual: default output byte-identical to U14 baseline; `--json` produces valid JSON; `--section=semantic` filters correctly.
- [ ] Task U19.11: Ledger provenance + commit + push.
