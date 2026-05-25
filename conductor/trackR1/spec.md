# Specification: Advanced CLI Help Interceptor

## Objective
Enhance the CLI to smartly handle standard and nested `help <cmd>` subcommands. Currently, `changeguard help <cmd>` falls back to global `--help`. It should intercept calls like `changeguard help ledger audit` and translate them cleanly to `changeguard ledger audit --help` before parsing by Clap.

## Requirements
- Intercept CLI arguments early in `src/main.rs` or `src/cli.rs`.
- Detect `help` as the first argument following the executable.
- Re-order arguments: Move `--help` to the end, drop the literal `help` string.
- Support deep nesting (e.g., `help ledger audit` -> `ledger audit --help`).
- Prevent interference with normal valid subcommands or native clap `help` if properly handled, but since clap's default help behavior is sometimes insufficient for deep subcommands, this interceptor ensures consistent manual override.

## Architecture & Integration
- Update argument preprocessing loop before `Cli::parse()`.
- Use a safe mutation of `std::env::args()`.
