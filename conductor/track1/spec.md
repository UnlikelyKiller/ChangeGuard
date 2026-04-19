# Specification: Track 1 - Repo-Local State Layout and Init

## 1. Overview
This track implements Phase 2 of the Changeguard roadmap (as per `Docs/Plan.md`). It focuses on establishing the `.changeguard/` repository-local state directory, scaffolding starter configurations, and safely integrating with Git's ignore mechanism.

## 2. Technical Requirements

### 2.1 Directory Layout
The CLI must idempotently create the following structure under the repository root:
```text
.changeguard/
  config.toml
  rules.toml
  logs/
  tmp/
  reports/
  state/
```

### 2.2 `.gitignore` Integration
- Must detect if `.changeguard/` is already ignored.
- Must append `.changeguard/` to the repository's root `.gitignore` if not present, ensuring a trailing newline is respected.
- Must preserve existing contents, formatting, and line endings.
- Must be bypassable via a `--no-gitignore` CLI flag.

### 2.3 Starter Configuration Files
- `config.toml`: Starter config file.
- `rules.toml`: Starter rules definitions.
- Must only be created if they do not already exist (do not overwrite user modifications).

### 2.4 Error Handling
- Strict adherence to `miette` for diagnostic-rich error types.
- Zero `unwrap()` or `expect()` calls in production paths.
- Provide meaningful contexts for all I/O and parsing errors.

## 3. CLI Interface
```bash
changeguard init [--no-gitignore]
```
