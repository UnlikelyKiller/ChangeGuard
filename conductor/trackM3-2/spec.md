# Specification: Track M3-2 — Ask Backend Routing & Integration

## Objective
Wire the local model backend into `changeguard ask` with auto-selection logic, the `--backend` flag, and output formatting consistent with the existing Gemini path. Extend `changeguard config verify` to report the active ask backend.

## Components

### 1. Backend Enum (`src/commands/ask.rs`)

```rust
pub enum Backend {
    Local,
    Gemini,
}
```

Add `--backend` flag to `AskArgs` in `src/cli.rs` accepting `local` or `gemini` (case-insensitive).

### 2. Auto-Selection Logic (`src/commands/ask.rs`)

```rust
pub fn resolve_backend(config: &Config, explicit: Option<Backend>) -> Backend
```

Priority order:
1. If `explicit` is `Some(b)`, return `b` (user override always wins)
2. If `config.local_model.prefer_local = true` AND `base_url` is non-empty → `Local`
3. If no Gemini API key is found (no `GEMINI_API_KEY` env var, no `.env` file entry, no `[gemini] api_key` in config.toml) AND `base_url` is non-empty → `Local`
4. Otherwise → `Gemini` (existing behavior)

### 3. Local Execution Path (`src/commands/ask.rs`)

In `execute_ask()`:
1. Resolve backend via `resolve_backend()`
2. If `Backend::Local`:
   - Call `assemble_context()` to build the full prompt
   - Call `local_model::client::complete()` to get the response
   - Print response with header: `Local Model Response:` (bold green, matching Gemini output style)
3. If `Backend::Gemini`:
   - Existing Gemini path in `wrapper.rs` (unchanged)
4. On local model `Err`: print the error message and return `Err` (same pattern as Gemini failure)

All four modes (`Analyze`, `Suggest`, `ReviewPatch`, `Narrative`) must work with `Backend::Local`.

### 4. `changeguard config verify` Extension

Extend the `config verify` subcommand output to include:

```
Ask backend:   Gemini (API key present)
```
or:
```
Ask backend:   Local (http://localhost:8080, prefer_local=true)
```
or:
```
Ask backend:   Gemini (API key present; prefer_local=false)
```

## Test Specifications

| Test | Assertion |
|---|---|
| `--backend local` parses | Produces `Backend::Local` |
| `--backend gemini` parses | Produces `Backend::Gemini` |
| `prefer_local = true`, no explicit flag | `resolve_backend` returns `Local` |
| No API key, `base_url` set, no flag | Returns `Local` |
| API key present, no explicit flag | Returns `Gemini` |
| `--backend gemini` with no API key | Returns `Gemini` (explicit overrides auto) |
| `Local` mock server returns canned response | `execute_ask` prints response and returns `Ok` |
| `Local` server unreachable | Returns `Err` with clear message |
| `ReviewPatch` with `Local`, clean tree | Falls back to general analysis, no error |
| `config verify` Local selected | Output shows local backend info |
| `config verify` Gemini selected | Output shows Gemini backend info |

## Constraints & Guidelines

- **TDD**: All tests written before implementation.
- **No duplication**: The Gemini path in `src/gemini/wrapper.rs` is unchanged.
- **Output consistency**: Local model response header uses same formatting style as Gemini response header.
- **Mock tests**: Use `httpmock` for HTTP-calling integration tests.
- **CI safety**: All tests pass when `base_url` is empty (falls through to Gemini path).
- **Backward compatibility**: Default behavior when no local model is configured is identical to current behavior.
