# Track I2-3: Agent Dotfile Exclusion

**Milestone:** I — Issue Remediation  
**Phase:** 2 — Reliability  
**Issue:** CG-7  
**Status:** In Planning

## Objective

`scan --impact` flags `.claude`, `.codex` (Windows junctions to `.agents`), and `.opencode/opencode.json` as "analysis unsupported" files, inserting them into the impact report as partially-analyzed changes. These are agent configuration directories, not source code, and should be silently excluded from all ChangeGuard analysis.

## Requirements

### Default Ignore Patterns (Config)
Add to `DEFAULT_CONFIG` in `src/config/defaults.rs`, under `[watch].ignore_patterns`:
```toml
ignore_patterns = [
    "target/**", ".git/**", "node_modules/**",
    ".claude/**", ".codex/**", ".opencode/**",
    ".agents/**"
]
```

Rationale for `.agents/**`: `.claude` and `.codex` are junctions pointing to `.agents`, so all three should be excluded. Excluding `.agents/**` directly prevents double-scanning via the real path.

### Hardcoded Exclusion (Index/Scan)
If `src/index/mod.rs` or the project index orchestrator maintains a hardcoded exclusion list (separate from the config-driven `ignore_patterns`), add these same four patterns there as well.

### Extension: `.json` in Agent Dirs
The AST parser currently flags `.json` as "unsupported language." The scope of this track is **exclusion** (don't touch those files at all), not parser support. No parser changes are needed.

## API Contract

No public API changes. The `ignore_patterns` config key is already a `Vec<String>` and accepts glob patterns.

## Testing Strategy

- Unit test `agent_dotfiles_excluded_from_scan`: create a temp repo containing `.claude/`, `.codex/`, `.opencode/opencode.json`, and a real `.rs` file. Run the file-walker with the default config. Assert none of the agent paths appear in the walked results.
- Assert the real `.rs` file IS present in the walked results (regression check).

## Out of Scope

- No change to the `.json` AST parser.
- No change to how junctions are followed (the exclusion pattern covers both the junction name and the real path).
