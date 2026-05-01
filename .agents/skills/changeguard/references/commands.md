# ChangeGuard Command Reference

This document contains the full command catalog, flags, and category definitions for ChangeGuard.

## Core Commands

### Impact & Scan

```bash
changeguard scan --impact           # Before edits: full change intelligence
changeguard impact --all-parents    # Include side-branch commits in coupling analysis
changeguard impact --summary        # One-line triage: RISK | N changed | N couplings
```

### Verification

```bash
changeguard verify                         # Run configured or predicted verification
changeguard verify -c "cargo clippy -- -D warnings"   # Manual single command
changeguard verify --no-predict            # Skip predictive suggestions
```

### Reset

```bash
changeguard reset                          # Preserves config, rules, and ledger.db
changeguard reset --remove-config          # Remove .changeguard/config.toml
changeguard reset --remove-rules           # Remove .changeguard/rules.toml
changeguard reset --include-ledger --yes   # Destructive: wipe ledger.db
changeguard reset --all --yes              # Destructive: wipe the entire .changeguard tree
```

### Audit & Search

```bash
changeguard audit [--entity PATH] [--include-unaudited]  # Holistic provenance view
changeguard ledger audit [--entity PATH]                 # Same as above (legacy alias)
changeguard ledger search QUERY [--category CAT] [--days N] [--breaking] [--limit N] # FTS5 search
```

## Ledger Subcommands (Provenance)

```bash
changeguard ledger start --entity PATH --category CAT [--message TEXT] [--issue REF]
changeguard ledger commit --tx-id ID --summary TEXT --reason TEXT [--change-type TYPE] [--breaking] [--auto-reconcile | --no-auto-reconcile]
changeguard ledger rollback --tx-id ID --reason TEXT
changeguard ledger atomic --entity PATH --summary TEXT --reason TEXT [--category CAT]
changeguard ledger note --entity PATH NOTE
changeguard ledger resume [ID]                              # Find most recent PENDING tx or resume specific    
changeguard ledger status [--entity PATH] [--compact]       # Holistic view or entity history
changeguard ledger reconcile [--tx-id ID] [--entity-pattern GLOB] [--all] --reason TEXT
changeguard ledger adopt [--tx-id ID] [--entity-pattern GLOB] [--all] [--reason TEXT]
changeguard ledger stack [--category CAT]                   # Show tech stack and validators
changeguard ledger register --rule-type TYPE --payload JSON [--force]   # Add enforcement rules
changeguard ledger adr [--output-dir DIR] [--days N]        # Export decisions to MADR
```

## Hotspots & Federation

```bash
changeguard hotspots --limit 20 --commits 500
changeguard hotspots --json
changeguard federate export
changeguard federate scan
changeguard federate status
```

## Gemini-Assisted Reporting

```bash
changeguard ask "What should I verify next?"
changeguard ask --mode suggest "What checks should I run?"
changeguard ask --mode review-patch "Review the current diff."
changeguard ask --narrative
```

## Categories

| Category | Covers |
|---|---|
| `ARCHITECTURE` | High-level system design, multi-module contracts |
| `FEATURE` | New user-facing or internal functionality |
| `BUGFIX` | Defect repairs |
| `REFACTOR` | Structural improvement without behavior change |
| `INFRA` | CI, git hooks, Docker, build system |
| `TOOLING` | Internal scripts, dev tooling |
| `DOCS` | Documentation, README, ADRs |
| `CHORE` | Dependencies, formatting, minor cleanup |
