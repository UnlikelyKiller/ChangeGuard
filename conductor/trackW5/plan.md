# Track W5 Plan: Data Model and Migration Compatibility Graph

- [ ] Task W5.1: Build fixtures for SQL migrations, ORM entities, raw SQL usage, and service ownership.
- [ ] Task W5.2: Write tests for schema-change classification and compatibility scoring.
- [ ] Task W5.3: Add graph relations for tables, columns, models, migrations, and compatibility classes.
- [ ] Task W5.4: Implement migration parser adapters for the repo-visible formats ChangeGuard can support locally.
- [ ] Task W5.5: Link data model usage to endpoints, services, tests, ADRs, and ledger transactions.
- [ ] Task W5.6: Add impact rules for destructive, untested, or unowned schema changes.
- [ ] Task W5.7: Add `changeguard data-models impact --changed` and JSON output.
- [ ] Task W5.8: Run focused data/model tests, migration tests, and the full verification gate; reinstall.

## Definition of Done Checklist

- [ ] Compatibility risk is explicit rather than implied.
- [ ] Endpoint and service blast radius includes affected data models.
- [ ] Backfill and test gaps appear in impact output.
- [ ] Full verification gate passes.
