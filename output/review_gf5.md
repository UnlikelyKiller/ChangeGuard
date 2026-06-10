**Findings**

- Low: the new CLI tests are useful, but they do not fully lock the public facade or the full `clap` contract for this split. The new module tests in [src/cli/mod.rs](<C:/dev/changeguard/src/cli/mod.rs:7>) cover a handful of aliases/help paths, and existing integration tests cover some ledger parsing in [tests/integration/ledger_cli_parsing.rs](<C:/dev/changeguard/tests/integration/ledger_cli_parsing.rs:1>), but there is still no direct compile-time/API test that all intended facade exports remain available from `changeguard::cli::{...}` and no targeted parse test for several fragile attributes such as hidden/internal command names, `data-models`, `--verify-signatures`, `--force-unlock`, or `--no-graph-sync` defined in [src/cli/args.rs](<C:/dev/changeguard/src/cli/args.rs:154>). For a public-surface refactor, that leaves avoidable regression space.

**Answers**

1. Yes. All public items that were public in the old `src/cli.rs` appear to remain reachable through the facade because [src/cli/mod.rs](<C:/dev/changeguard/src/cli/mod.rs:4>) re-exports `args::*` and [src/cli/mod.rs](<C:/dev/changeguard/src/cli/mod.rs:5>) re-exports `run_with`. Internal and integration code already still imports `changeguard::cli::{Cli, Commands, ...}` successfully by shape, for example [src/main.rs](<C:/dev/changeguard/src/main.rs:1>) and [tests/integration/cli_surfaces.rs](<C:/dev/changeguard/tests/integration/cli_surfaces.rs:30>).

2. I did not find any dropped `clap` attributes in the split. The staged rename keeps the attribute-bearing definitions in [src/cli/args.rs](<C:/dev/changeguard/src/cli/args.rs:7>) with the important command/arg metadata still present, including `disable_help_subcommand`, `name = "data-models"`, `visible_alias = "upgrade"`, `alias = "output-dir"`, `name = "hook-commit-msg"`, `name = "hook-post-commit"`, and the long-name overrides.

3. Yes. `run_with` is still the stable entry point: it remains `pub fn run_with(cli: Cli) -> Result<()>` in [src/cli/dispatch.rs](<C:/dev/changeguard/src/cli/dispatch.rs:9>), it is re-exported from [src/cli/mod.rs](<C:/dev/changeguard/src/cli/mod.rs:5>), and `main` still calls `cli::run_with(cli_args)` in [src/main.rs](<C:/dev/changeguard/src/main.rs:62>).

4. I did not find any missed re-exports relative to the previous public surface. The old public surface was the public CLI types plus `run_with`; the new facade re-exports exactly that surface.

5. Tests are not fully sufficient for this refactor. They are enough to give confidence that the split did not obviously break parsing, but not enough to guarantee facade stability or comprehensive `clap` metadata preservation.

I did not modify files, and I did not run `cargo` verification in this read-only environment.