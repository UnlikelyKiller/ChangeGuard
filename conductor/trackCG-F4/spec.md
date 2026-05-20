## Spec: Fix federate scan Noisy Schema Warnings (Track CG-F4)

### Acceptance Criteria

1. **No noise for non-ChangeGuard repos**: `changeguard federate scan` does not warn about schema for sibling repos that lack `.changeguard/`
2. **Still warns for malformed schemas**: If a repo has `.changeguard/` but invalid schema, the warning still appears
3. **Federated discovery still works**: Sibling repos WITH valid schemas are still discovered and linked
4. **CI gate passes**: `cargo fmt --check`, `cargo clippy`, `cargo test` all pass
