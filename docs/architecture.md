# ChangeGuard Architecture

ChangeGuard keeps the CLI thin and pushes behavior into focused subsystems.

## Data Flow

```text
CLI
  -> commands/
    -> git/         scan repository state
    -> index/       extract symbols, imports, runtime usage
    -> impact/      assemble packet and score risk
    -> verify/      build plans and persist results
    -> gemini/      render prompts and invoke Gemini
    -> state/       layout, reports, SQLite persistence
    -> watch/       debounce filesystem events into batches
```

## Boundaries

- `commands/`: command entry points and orchestration only.
- `git/`: repository discovery, status, diff, classification.
- `index/`: changed-file intelligence, not whole-program analysis.
- `impact/`: packet assembly, redaction, reasoning, risk scoring.
- `verify/`: deterministic plan generation and verification report persistence.
- `gemini/`: prompt construction, sanitization, and subprocess invocation.
- `state/`: repo-local paths, migrations, reports, and SQLite storage.
- `watch/`: event filtering, normalization, batching, and callback dispatch.
- `platform/`: environment detection and policy seams.

## State Layout

```text
.changeguard/
  config.toml
  rules.toml
  logs/
  tmp/
  reports/
    latest-impact.json
    latest-verify.json
  state/
    current-batch.json
    ledger.db
```
