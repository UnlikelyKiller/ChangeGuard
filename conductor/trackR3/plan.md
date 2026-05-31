## Plan: Proactive Index Repair & Health
### Phase 1: Health Probes
- [x] Task 1.1: Implement an integrity check in `src/commands/doctor.rs` that probes Tantivy metadata/locks.
- [x] Task 1.2: Implement a staleness check that queries CozoDB for the last indexed git hash and compares with current `HEAD`.
### Phase 2: UX & Reporting
- [x] Task 2.1: Update the doctor command output to include "Index Health" section.
- [x] Task 2.2: Provide specific `repair` instructions (e.g., `changeguard index --full`) when degradation is detected.
### Phase 3: Verification
- [x] Task 3.1: Manually modify a file without indexing, run `doctor`, verify staleness warning.
- [x] Task 3.2: Delete Tantivy `meta.json`, run `doctor`, verify corruption warning.