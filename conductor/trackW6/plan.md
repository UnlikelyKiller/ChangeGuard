# Track W6 Plan: Config and Environment Variable Ownership

- [ ] Task W6.1: Add config schema fixtures for required, optional, defaulted, secret, environment-scoped, and owner-scoped variables.
- [ ] Task W6.2: Write red tests for inferred versus declared metadata merge behavior.
- [ ] Task W6.3: Extend env schema data models and graph links.
- [ ] Task W6.4: Add validation for invalid required/default/secret combinations.
- [ ] Task W6.5: Add impact rules for requiredness, default, removal, and secret exposure changes.
- [ ] Task W6.6: Implement `changeguard config schema` and `changeguard config diff`.
- [ ] Task W6.7: Add redaction tests proving secret values never appear in output.
- [ ] Task W6.8: Run config, env, redaction, and full verification gates; reinstall.

## Definition of Done Checklist

- [ ] Config output separates unknown, optional, required, and defaulted states.
- [ ] Secret output is redacted in every human, JSON, prompt, and impact path.
- [ ] Service-scoped config changes map to owners and tests when known.
- [ ] Full verification gate passes.
