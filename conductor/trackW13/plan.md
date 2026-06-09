# Track W13 Plan: Security Boundary, Authz, and Policy Graph

- [x] Task W13.1: Create fixtures for Cedar policies, Cedar schemas/entities where useful, auth middleware, IAM/policy files, roles/scopes, protected paths, secret references, and process policy.
- [x] Task W13.2: Write tests proving secret values stay redacted across all output modes.
- [x] Task W13.3: Add security boundary graph node and edge types using W1 helpers.
- [x] Task W13.4: Add the primary Cedar PST importer for principal/action/resource constraints, permit/forbid effects, templates, links, and conservative wildcard handling.
- [x] Task W13.5: Add secondary authz/policy extraction and configurable overlays for app-specific authorization facts outside Cedar.
- [x] Task W13.6: Link security boundaries to endpoints, services, config keys, deploy manifests, ADRs, tests, dependencies, and ledger transactions.
- [x] Task W13.7: Add impact rules for auth bypass, policy broadening, Cedar scope changes, protected path edits, external process execution, and missing security review.
- [x] Task W13.8: Implement `changeguard security impact --changed` and `changeguard security boundaries`.
- [x] Task W13.9: Run redaction, Cedar policy, generic policy, security, integration, and full verification gates; reinstall.

## Definition of Done Checklist

- [x] Security graph output is useful without exposing secrets.
- [x] Cedar principal/action/resource edges are queryable and labeled with policy IDs and source locations.
- [x] Generic auth evidence is clearly separated from Cedar-derived structured policy facts.
- [x] Auth/authz changes affect endpoint and service impact.
- [x] Protected path and process-policy changes name review requirements.
- [x] Full verification gate passes.
