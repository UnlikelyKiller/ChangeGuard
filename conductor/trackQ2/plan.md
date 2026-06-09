## Plan: Track Q2 CLI Ergonomics

### Phase 1: Ledger Audit Argument Parsing Fix
- [x] Task 1.1: Edit `src/cli.rs` and locate `LedgerCommands::Audit`. Add `#[arg(short, long)]` to the `entity: Option<String>` field.
- [x] Task 1.2: In `src/cli.rs`, locate `Commands::Audit` (the top-level alias) and apply the same `#[arg(short, long)]` attribute to its `entity` field.

### Phase 2: Help and Version Unification
- [x] Task 2.1: Update `src/cli.rs` to include `#[command(disable_help_subcommand = true)]` on the main `Cli` struct to prevent `clap` from auto-generating the default `help` subcommand.
- [x] Task 2.2: Modify `src/main.rs` to intercept `std::env::args()` before parsing.
- [x] Task 2.3: In the `src/main.rs` intercept logic, detect if the first argument (after the binary name) is `"help"`. If so, remove it and append `"--help"` to map `changeguard help <cmd>` to `changeguard <cmd> --help`.
- [x] Task 2.4: In the `src/main.rs` intercept logic, detect if the first argument is `"version"`. If so, replace it with `"--version"` to cleanly map the subcommand behavior to the flag behavior.
- [x] Task 2.5: Update `Cli::parse()` in `src/main.rs` to use `Cli::parse_from(args)` utilizing the modified arguments vector.

### Phase 3: Verification
- [x] Task 3.1: Compile the project (`cargo build`).
- [x] Task 3.2: Manually verify `changeguard help` and `changeguard --help` produce identical output.
- [x] Task 3.3: Manually verify `changeguard help ledger audit` invokes the sub-command help perfectly.
- [x] Task 3.4: Manually verify `changeguard ledger audit --entity src/main.rs` parses successfully without positional argument errors.
- [x] Task 3.5: Run `cargo test` to ensure CLI arg parsing invariants remain intact.