# Track O1-6: SOC2 Evidence Export

## Objective
Provide an auditor-ready JSON or CSV export of the ChangeGuard ledger that specifically maps to AICPA TSP 100 Common Criteria (CC6.1, CC6.6, CC7.2, CC8.1) for change management controls.

## Requirements
*   **CLI Command:** Implement `changeguard export --format=soc2 --period=Q1-2026` (or explicit date range flags).
*   **Data Mapping:** Query the ledger and map the intent fields (`summary`, `reason`, `category`, `risk`, `author`, `signature`) into a structured schema that aligns with standard auditor requirements.
*   **File Generation:** Output a formatted JSON or CSV file to the `reports/` directory.

## Definition of Done (DoD)
*   [ ] The `export` command correctly parses date ranges/periods.
*   [ ] Ledger entries are mapped to the SOC2 evidence schema.
*   [ ] The export file is successfully written.
*   [ ] A unit test verifies the exported schema structure against sample ledger data.