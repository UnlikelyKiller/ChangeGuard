repo{
  name:"ChangeGuard"
  os:"Windows"
  goal:"scoped edits; verified behavior; clean provenance"
}

onboarding{
  startup:"execute /.agents/workflows/onboarding.md at session start"
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
}

verify{
  scope:"targeted during work; full commands before commit"
  commands[3]:
    "cargo fmt --all -- --check"
    "cargo clippy --all-targets --all-features -- -D warnings"
    "cargo test"
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
  use[3]:
    "changeguard ask for architecture/codebase questions"
    "changeguard index --analyze-graph to refresh structure"
    "changeguard viz for deep architecture review"
}

powershell{
  forbid[7]:"&&","[[","]]","then","fi","done","echo -e"
  prefer[6]:"Get-ChildItem","Get-Content","Test-Path","Join-Path","Copy-Item","Remove-Item"
  rules[3]:
    "use $_ and object properties for pipelines"
    "use backslashes for shell-level Windows paths"
    "avoid Bash shims for complex logic"
}
