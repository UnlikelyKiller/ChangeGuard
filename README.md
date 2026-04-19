# ChangeGuard

ChangeGuard is a local-first Rust CLI for change intelligence and Gemini-assisted development. It watches substantive repository changes, builds impact packets, recommends targeted verification, and prepares bounded context for Gemini without turning into an autonomous agent.

## Install

```powershell
cargo install --path .
```

## Quickstart

```powershell
changeguard init
changeguard doctor
changeguard scan
changeguard impact
changeguard verify
changeguard ask "What should I verify next?"
```

## Commands

- `init`: create `.changeguard/`, starter config, starter rules, and `.gitignore` wiring.
- `doctor`: report platform, shell, path, and tool health.
- `scan`: summarize staged and unstaged git changes.
- `watch`: debounce file-system events into persisted batches.
- `impact`: generate a structured impact packet with symbol/runtime/import enrichment.
- `verify`: execute targeted verification and persist the report.
- `ask`: send the latest impact context to Gemini in `analyze`, `suggest`, or `review-patch` mode.
- `reset`: reset derived local state.

## Configuration

ChangeGuard stores repo-local state in `.changeguard/`.

- `.changeguard/config.toml`: runtime configuration like watch debounce and Gemini timeout.
- `.changeguard/rules.toml`: policy rules, protected paths, and required verification commands.

Examples live in [docs/examples/config.toml](docs/examples/config.toml), [docs/examples/rules.toml](docs/examples/rules.toml), and [docs/examples/CHANGEGUARD.md](docs/examples/CHANGEGUARD.md).

## Gemini

ChangeGuard shells out to the `gemini` CLI. Ensure it is on `PATH` before using `changeguard ask`.

- `--mode analyze`: blast-radius and risk reasoning
- `--mode suggest`: targeted verification recommendations
- `--mode review-patch`: patch review with live diff context

## Windows / WSL

- Windows 11 + PowerShell is the primary environment.
- Mixed Windows/WSL filesystem setups can be slower and may produce different tool availability.
- Keep `git` and `gemini` installed in the environment where you run ChangeGuard.

## Architecture

See [docs/architecture.md](docs/architecture.md) for module boundaries and data flow.

## Contributing

- Work by conductor track.
- Keep changes phase-bounded and deterministic.
- Run `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test` before pushing.

## License

See [LICENSE](LICENSE).
