# Track U18 Plan: Audit and Fix All Option<T> Serde Defaults in Config

- [ ] Task U18.1: Run the audit grep: `rg "pub (\w+): Option<" src/config/model.rs` and identify every `Option<T>` field with a non-`None` intended default.
- [ ] Task U18.2: For each affected field, add a `default_<field>() -> Option<T>` helper function.
- [ ] Task U18.3: Switch the `#[serde(default)]` attribute to `#[serde(default = "default_<field>")]`.
- [ ] Task U18.4: For each `Option<T>` field where `None` is the intended default, add a doc comment explaining the contract.
- [ ] Task U18.5: Write "partial section preserves default" tests for every fixed field.
- [ ] Task U18.6: Run CI gate.
- [ ] Task U18.7: Manual "kitchen sink" config: one field per section, verify every other field's accessor returns the constant default.
- [ ] Task U18.8: Ledger provenance + commit + push.
