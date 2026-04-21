# ChangeGuard

ChangeGuard is a local-first Rust CLI for change intelligence and Gemini-assisted development. It turns repository edits into deterministic impact packets, risk summaries, hotspot rankings, targeted verification plans, and bounded Gemini context.

The tool is designed to stay local and explain its work. It does not act as an autonomous coding agent.

## Install

One-line install for AI agents and developers:

```powershell
iwr https://raw.githubusercontent.com/UnlikelyKiller/ChangeGuard/main/install/install.ps1 -UseB | iex
```

```bash
curl -fsSL https://raw.githubusercontent.com/UnlikelyKiller/ChangeGuard/main/install/install.sh | sh
```

Manual install from a checkout:

```powershell
cargo install --path .
```

The LSP daemon is behind an optional feature:

```powershell
cargo install --path . --features daemon
```

See [docs/installation.md](docs/installation.md) for installer options, release assets, and agent bootstrap instructions.

## Quickstart

```powershell
changeguard init
changeguard doctor
changeguard scan
changeguard impact
changeguard verify
changeguard hotspots
changeguard ask "What should I verify next?"
```

## Commands

- `init`: create `.changeguard/`, starter config, starter rules, and `.gitignore` wiring.
- `doctor`: report platform, shell, path, and tool health.
- `scan`: summarize staged and unstaged git changes.
- `watch`: debounce file-system events into persisted batches.
- `impact`: generate `latest-impact.json` with symbols, imports, runtime usage, complexity, temporal coupling, hotspots, and federated impact.
- `verify`: build and run a deterministic verification plan, including predictive files from current imports, temporal coupling, and packet history.
- `ask`: send sanitized impact context to Gemini in `analyze`, `suggest`, `review-patch`, or narrative mode.
- `hotspots`: rank files by temporal change frequency multiplied by complexity.
- `federate`: export public interfaces, scan sibling repositories, and show known federated links.
- `daemon`: optional LSP server with diagnostics, Hover, CodeLens, stale-data handling, and lifecycle management.
- `reset`: remove derived local state, with opt-in flags for config/rules or the full `.changeguard/` tree.

## Common Workflows

Generate an impact report using first-parent git history:

```powershell
changeguard impact
```

Include all parent traversal for merge-heavy repositories:

```powershell
changeguard impact --all-parents
```

Run predictive verification:

```powershell
changeguard verify
```

Disable prediction and use rule-only verification:

```powershell
changeguard verify --no-predict
```

Inspect risk hotspots:

```powershell
changeguard hotspots --limit 20 --commits 500 --dir src --lang rs
changeguard hotspots --json
```

Use Gemini narrative reporting:

```powershell
changeguard ask --narrative
```

Use federated intelligence across sibling repositories:

```powershell
changeguard federate export
changeguard federate scan
changeguard federate status
changeguard impact
```

Start the optional LSP daemon:

```powershell
cargo run --features daemon -- daemon
```

## Configuration

ChangeGuard stores repo-local state in `.changeguard/`.

- `.changeguard/config.toml`: runtime configuration, watch debounce, Gemini timeout/context, temporal traversal, and hotspot defaults.
- `.changeguard/rules.toml`: policy rules, protected paths, and required verification commands.

Examples live in [docs/examples/config.toml](docs/examples/config.toml), [docs/examples/rules.toml](docs/examples/rules.toml), and [docs/examples/CHANGEGUARD.md](docs/examples/CHANGEGUARD.md).

## Reports And State

Generated state is rebuildable and stays inside `.changeguard/`.

- `.changeguard/reports/latest-scan.json`
- `.changeguard/reports/latest-impact.json`
- `.changeguard/reports/latest-verify.json`
- `.changeguard/reports/fallback-impact.json`
- `.changeguard/state/ledger.db`
- `.changeguard/state/schema.json`
- `.changeguard/state/current-batch.json`

Impact packets are redacted before SQLite persistence. Gemini prompts are sanitized and truncated before subprocess execution.

## Gemini

ChangeGuard shells out to the `gemini` CLI. Ensure it is on `PATH` before using `changeguard ask`.

- `GEMINI_API_KEY` can be supplied from the process environment or a repo-local `.env` file. `.env` is ignored by git; use `.env.example` as the template.
- By default, routine `analyze`, `suggest`, and narrative requests use `gemini-3.1-flash-lite-preview` for lower latency and cost.
- High-risk packets and `review-patch` requests use `gemini-3.1-pro-preview` for deeper reasoning and code review.
- Set `gemini.model` in `.changeguard/config.toml` only when you want one explicit model for every ask mode.
- `--mode analyze`: blast-radius and risk reasoning
- `--mode suggest`: targeted verification recommendations
- `--mode review-patch`: patch review with live diff context
- `--narrative`: senior-architect risk narrative generated from one structured prompt

If Gemini fails after an impact packet is available, ChangeGuard writes a fallback impact artifact or reports why it could not.

## Windows / WSL

- Windows 11 + PowerShell is the primary environment.
- Mixed Windows/WSL filesystem setups can be slower and may produce different tool availability.
- Keep `git` and `gemini` installed in the environment where you run ChangeGuard.

## Architecture

See [docs/architecture.md](docs/architecture.md) for module boundaries and current data flow.

## Contributing

- Work by conductor track.
- Keep changes phase-bounded and deterministic.
- Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features -j 1 -- --test-threads=1` before pushing.

## License

See [LICENSE](LICENSE).
