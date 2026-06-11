repo{
  name:"ChangeGuard"
  os:"Windows"
  goal:"scoped edits; verified behavior; clean provenance"
}

onboarding{
  skill:".agents/skills/onboarding/SKILL.md"
}

changeguard{
  before[3]:
    "changeguard ledger status --compact"
    "changeguard scan --impact for meaningful code/config/policy edits"
    "read .changeguard/reports/latest-impact.json if present"
  edit[3]:
    "do not edit .changeguard state files"
    "inspect hotspots"
    "inspect temporal couplings >70%"
  after[3]:
    "changeguard verify; if aliases fail, use verify.commands"
    "cargo install --path . after ChangeGuard source edits"
    "report risk, verification, pending tx, drift"
  skip[5]:
    "format-only"
    "scratch files"
    "binary/media-only"
    "lockfile-only dependency churn"
    "explicit user bypass"
  fail{
    unavailable:"continue with native checks; report missing signals"
    drift:"reconcile/adopt before continuing unless user says otherwise"
    verify:"report exact failed command and continue with justified fallback"
  }
}

ledger{
  start:"changeguard ledger start <entity> --category <CATEGORY> --message <intent>"
  commit:"changeguard ledger commit <tx-id> --summary <what> --reason <why>"
  hooks[2]:
    "pre-commit: changeguard ledger status --compact --exit-code"
    "pre-push: changeguard ledger status --compact --exit-code"
  stale_sidecar:"after git commit, if ledger status shows 1 pending, run ledger commit immediately; the hook removes the sidecar before post-commit can promote it"
}

verify{
  scope:"targeted during work; full commands before commit"
  commands[4]:
    "cargo fmt --all -- --check"
    "cargo clippy --all-targets --all-features -- -D warnings"
    "cargo nextest run --lib --bins --workspace"
    "cargo nextest run --test integration (when integration test files are touched)"
  hygiene[2]:
    "no secrets or .env commits"
    "temporary output belongs in output/ and should be removed before finish"
}

rust{
  forbid[2]:".unwrap()","expect() in production"
  errors:"use miette + Result"
  boundaries[2]:
    "src/search owns search"
    "src/state owns persistence"
  invariants[2]:
    "features work offline with local model"
    "preserve Windows paths; prefer camino for UTF-8 paths"
}

kg{
  backend:"CozoDB"
  state:".changeguard/state/ledger.cozo"
  use[5]:
    "changeguard search for high-precision regex/text discovery (prefer over grep)"
    "changeguard ask --semantic for conceptual discovery (prefer over semantic search)"
    "changeguard ask for architecture/codebase questions"
    "changeguard index --analyze-graph to refresh structure"
    "changeguard viz for deep architecture review"
  surfaces[8]:
    "changeguard endpoints --changed / --json"
    "changeguard services diff"
    "changeguard data-models impact --changed"
    "changeguard config schema / config diff"
    "changeguard observability diff / observability coverage"
    "changeguard hotspots trend / hotspots explain"
    "changeguard security boundaries / security impact --changed"
    "changeguard ledger graph <tx-id>"
}

powershell{
  forbid[7]:"&&","[[","]]","then","fi","done","echo -e"
  prefer[6]:"Get-ChildItem","Get-Content","Test-Path","Join-Path","Copy-Item","Remove-Item"
  rules[3]:
    "use $_ and object properties for pipelines"
    "use backslashes for shell-level Windows paths"
    "avoid Bash shims for complex logic"
}

aibrains{
  preflight:"session start briefing: run 'ai-brains preflight --summary'"
  pre_edit:"check constraints before risky edits: run 'ai-brains preflight --summary'"
  unified_search:"query memory + code symbols: run 'ai-brains sync query \"<query>\"'"
  recall:"query past decisions only: run 'ai-brains recall \"<query>\" --semantic'"
  pin:"persist decisions/constraints: run 'ai-brains pin \"<DECISION/CONSTRAINT/HOTSPOT: message>\"'"
}

