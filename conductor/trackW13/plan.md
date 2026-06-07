# Track W13 Plan: Security Boundary, Authz, and Policy Graph

- [ ] Task W13.1: Create fixtures for Cedar policies, Cedar schemas/entities where useful, auth middleware, IAM/policy files, roles/scopes, protected paths, secret references, and process policy.
- [ ] Task W13.2: Write tests proving secret values stay redacted across all output modes.
- [ ] Task W13.3: Add security boundary graph node and edge types using W1 helpers.
- [ ] Task W13.4: Add the primary Cedar PST importer for principal/action/resource constraints, permit/forbid effects, templates, links, and conservative wildcard handling.
- [ ] Task W13.5: Add secondary authz/policy extraction and configurable overlays for app-specific authorization facts outside Cedar.
- [ ] Task W13.6: Link security boundaries to endpoints, services, config keys, deploy manifests, ADRs, tests, dependencies, and ledger transactions.
- [ ] Task W13.7: Add impact rules for auth bypass, policy broadening, Cedar scope changes, protected path edits, external process execution, and missing security review.
- [ ] Task W13.8: Implement `changeguard security impact --changed` and `changeguard security boundaries`.
- [ ] Task W13.9: Run redaction, Cedar policy, generic policy, security, integration, and full verification gates; reinstall.

## Definition of Done Checklist

- [ ] Security graph output is useful without exposing secrets.
- [ ] Cedar principal/action/resource edges are queryable and labeled with policy IDs and source locations.
- [ ] Generic auth evidence is clearly separated from Cedar-derived structured policy facts.
- [ ] Auth/authz changes affect endpoint and service impact.
- [ ] Protected path and process-policy changes name review requirements.
- [ ] Full verification gate passes.
