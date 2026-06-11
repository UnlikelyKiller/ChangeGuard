# Track Z-R4 Plan: CozoDB Parameterized Queries & Test Precision

## Phase 1 — Safe Helpers in CozoStorage
- [ ] 1. In `src/state/storage_cozo.rs`, add:
  ```rust
  pub fn query_nodes_by_category(&self, category: &str) -> Result<NamedRows> {
      let mut params = BTreeMap::new();
      params.insert("cat".into(), DataValue::Str(category.into()));
      self.run_script_with_params(
          "?[id, label] := *node{id, label, category: $cat}",
          params,
          ScriptMutability::Immutable,
      )
  }

  pub fn query_edges_by_source(&self, source: &str, relation: &str,
  ) -> Result<NamedRows> {
      let mut params = BTreeMap::new();
      params.insert("src".into(), DataValue::Str(source.into()));
      params.insert("rel".into(), DataValue::Str(relation.into()));
      self.run_script_with_params(
          "?[source, target, relation] := *edge{source, target, relation}, source = $src, relation = $rel",
          params,
          ScriptMutability::Immutable,
      )
  }
  ```
  (Add `query_edges_by_target` symmetrically.)
- [ ] 2. Run `cargo check`.

## Phase 2 — Migrate Tests
- [ ] 3. In `tests/integration/track_z5_repro.rs`:
  - Replace `format!` queries with helper calls.
  - Replace `src_label.contains("test_add") && tgt_label.contains("add")` with exact equality checks on the extracted `DataValue::Str` contents.
- [ ] 4. In `tests/integration/track_z6_repro.rs`:
  - Replace `v.to_string().contains("test.rs")` with exact URN equality.
- [ ] 5. In `tests/integration/ledger_graph_edges.rs`:
  - Replace all `format!` Datalog with helpers or inline `run_script_with_params`.
- [ ] 6. In `tests/integration/track_z2_repro.rs`:
  - Add `test_data_models_impact_json_output` that invokes `data-models impact --changed --json` and asserts valid JSON with `impacted` array.
  - Remove any `format!` Cozo queries if present.

## Phase 3 — Production Hardening
- [ ] 7. In `src/index/graph_loader.rs` (lines 1250–1266), replace:
  ```rust
  // Before
  format!("?[id] := *node{{id, category: '{cat_str}'}} :rm node {{id}}")
  ```
  with parameterized equivalents using `run_script_with_params`.

## Phase 4 — Verification
- [ ] 8. Run `cargo nextest run --lib --bins --workspace`.
- [ ] 9. Run `cargo nextest run --test integration`.
- [ ] 10. Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] 11. Run `cargo fmt --all -- --check`.
- [ ] 12. Install binary with `cargo install --path .`.
