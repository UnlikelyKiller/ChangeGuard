pub mod m11_to_m20;
pub mod m1_to_m10;
pub mod m21_to_m29;
pub mod m30_scip;
pub mod m31_ci_predict;
pub mod m32_symbol_metadata;
pub mod m33_intent_provenance;
pub mod m34_api_route_enrichment;

use rusqlite_migration::Migrations;

pub fn get_migrations() -> Migrations<'static> {
    let mut all_m = Vec::new();
    all_m.extend(m1_to_m10::m1_to_m10());
    all_m.extend(m11_to_m20::m11_to_m20());
    all_m.extend(m21_to_m29::m21_to_m29());
    all_m.extend(m30_scip::m30_scip());
    all_m.extend(m31_ci_predict::m31_ci_predict());
    all_m.extend(m32_symbol_metadata::m32_symbol_metadata());
    all_m.extend(m33_intent_provenance::m33_intent_provenance());
    all_m.extend(m34_api_route_enrichment::m34_api_route_enrichment());

    Migrations::new(all_m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_validate() {
        let migrations = get_migrations();
        migrations.validate().unwrap();
    }

    #[test]
    fn test_all_tables_exist() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        let expected_tables = [
            "snapshots",
            "batches",
            "changed_files",
            "verification_runs",
            "verification_results",
            "symbols",
            "federated_links",
            "federated_dependencies",
            "transactions",
            "ledger_entries",
            "ledger_fts",
            "tech_stack",
            "commit_validators",
            "category_stack_mappings",
            "watcher_patterns",
            "token_provenance",
            "project_files",
            "index_metadata",
            "project_symbols",
            "project_docs",
            "project_topology",
            "structural_edges",
            "api_routes",
            "data_models",
            "symbol_centrality",
            "observability_patterns",
            "test_mapping",
            "ci_gates",
            "env_declarations",
            "env_references",
            "embeddings",
            "doc_chunks",
            "api_endpoints",
            "test_outcome_history",
            "observability_snapshots",
            "scip_indices",
            "ci_outcome_history",
        ];

        for table in &expected_tables {
            let count: i64 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "Table {} should exist", table);
        }
    }
}
