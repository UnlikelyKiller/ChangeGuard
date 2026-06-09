use camino::Utf8PathBuf;
use changeguard::config::model::{Config, ServiceDefinition};
use changeguard::index::graph_loader::build_native_graph;
use changeguard::index::orchestrator::ProjectIndexer;
use changeguard::state::storage::StorageManager;
use std::fs;

#[test]
fn test_observability_and_cedar_graph_wiring() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();

    // Setup basic git repo structure
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("lib.rs"), "pub fn main() {}").unwrap();

    // 1. Create OpenSLO fixture
    let obs_dir = root.join("observability");
    fs::create_dir_all(&obs_dir).unwrap();
    let openslo_yaml = r#"
apiVersion: openslo/v1
kind: SLO
metadata:
  name: user-service-availability
  displayName: User Service Availability
  owner: platform-team
spec:
  service: user-service
  indicator:
    thresholdMetric:
      metricSource:
        metricSourceRef: prometheus
        type: prometheus
      metricQuery: sum(rate(http_requests_total{status=~"2.."}[5m]))
  objectives:
    - target: 0.999
"#;
    fs::write(obs_dir.join("slo.yaml"), openslo_yaml).unwrap();

    // 2. Create Cedar policy fixture
    let policy_dir = root.join("policies");
    fs::create_dir_all(&policy_dir).unwrap();
    let cedar_policy = r#"
permit(
    principal == User::"alice",
    action == Action::"view",
    resource == Photo::"vacation.jpg"
);
"#;
    fs::write(policy_dir.join("policy.cedar"), cedar_policy).unwrap();

    // 3. Setup Config with service definitions
    let mut config = Config::default();
    config.services.definitions = vec![ServiceDefinition {
        name: "user-service".to_string(),
        root: "src/".to_string(),
        owners: vec!["platform-team".to_string()],
        runtime_name: None,
        queues: vec![],
        topics: vec![],
        rpc_endpoints: vec![],
    }];

    // 4. Run indexer and build call graph
    let db_dir = root.join(".changeguard").join("state");
    fs::create_dir_all(&db_dir).unwrap();
    let storage = StorageManager::init(db_dir.join("ledger.db").as_std_path()).unwrap();
    let mut indexer = ProjectIndexer::new(storage, root.clone(), config.clone());
    indexer.full_index().unwrap();
    indexer.build_call_graph().unwrap();

    let cozo = indexer.cozo().expect("CozoDB should be available");
    build_native_graph(indexer.storage(), cozo, "full", &config).unwrap();

    // 5. Query and verify OpenSLO nodes and edges
    // Verify Slo node exists
    let res_slo = cozo.run_script(
        "?[id, label] := *node{id, label, category: 'slo'}, id = 'urn:changeguard:slo:user-service-availability'"
    ).unwrap();
    assert_eq!(res_slo.rows.len(), 1, "SLO node should be inserted");

    // Verify Metric node exists
    let res_metric = cozo.run_script(
        "?[id, label] := *node{id, label, category: 'metric'}, id = 'urn:changeguard:metric:user-service-availability-threshold'"
    ).unwrap();
    assert_eq!(res_metric.rows.len(), 1, "Metric node should be inserted");

    // Verify Service node exists
    let res_service = cozo.run_script(
        "?[id, label] := *node{id, label, category: 'service'}, id = 'urn:changeguard:service:user-service'"
    ).unwrap();
    assert_eq!(res_service.rows.len(), 1, "Service node should be inserted");

    // Verify Owner node exists
    let res_owner = cozo.run_script(
        "?[id, label] := *node{id, label, category: 'role'}, id = 'urn:changeguard:role:platform-team'"
    ).unwrap();
    assert_eq!(res_owner.rows.len(), 1, "Owner node should be inserted");

    // Verify SLO Monitors Service edge exists
    let res_monitors = cozo
        .run_script(
            "?[src, tgt] := *edge{source: src, target: tgt, relation: 'monitors'}, \
         src = 'urn:changeguard:slo:user-service-availability', \
         tgt = 'urn:changeguard:service:user-service'",
        )
        .unwrap();
    assert_eq!(
        res_monitors.rows.len(),
        1,
        "SLO monitors Service edge should exist"
    );

    // Verify SLO DependsOn Metric edge exists
    let res_depends = cozo
        .run_script(
            "?[src, tgt] := *edge{source: src, target: tgt, relation: 'depends_on'}, \
         src = 'urn:changeguard:slo:user-service-availability', \
         tgt = 'urn:changeguard:metric:user-service-availability-threshold'",
        )
        .unwrap();
    assert_eq!(
        res_depends.rows.len(),
        1,
        "SLO depends_on Metric edge should exist"
    );

    // Verify Owner Owns Service edge exists
    let res_owns_svc = cozo
        .run_script(
            "?[src, tgt] := *edge{source: src, target: tgt, relation: 'owns'}, \
         src = 'urn:changeguard:role:platform-team', \
         tgt = 'urn:changeguard:service:user-service'",
        )
        .unwrap();
    assert_eq!(
        res_owns_svc.rows.len(),
        1,
        "Owner owns Service edge should exist"
    );

    // Verify Owner Owns SLO edge exists
    let res_owns_slo = cozo
        .run_script(
            "?[src, tgt] := *edge{source: src, target: tgt, relation: 'owns'}, \
         src = 'urn:changeguard:role:platform-team', \
         tgt = 'urn:changeguard:slo:user-service-availability'",
        )
        .unwrap();
    assert_eq!(
        res_owns_slo.rows.len(),
        1,
        "Owner owns SLO edge should exist"
    );

    // 6. Query and verify Cedar Policy nodes and edges
    // Verify Policy node exists
    let res_policy = cozo
        .run_script("?[id, label] := *node{id, label, category: 'policy'}")
        .unwrap();
    assert_eq!(res_policy.rows.len(), 1, "Policy node should be inserted");

    // Verify Principal node exists
    let res_principal = cozo.run_script(
        "?[id, label] := *node{id, label, category: 'principal'}, id = 'urn:changeguard:principal:User::\"alice\"'"
    ).unwrap();
    assert_eq!(
        res_principal.rows.len(),
        1,
        "Principal node should be inserted"
    );

    // Verify Action node exists
    let res_action = cozo.run_script(
        "?[id, label] := *node{id, label, category: 'action'}, id = 'urn:changeguard:action:Action::\"view\"'"
    ).unwrap();
    assert_eq!(res_action.rows.len(), 1, "Action node should be inserted");

    // Verify Resource node exists
    let res_resource = cozo.run_script(
        "?[id, label] := *node{id, label, category: 'resource'}, id = 'urn:changeguard:resource:Photo::\"vacation.jpg\"'"
    ).unwrap();
    assert_eq!(
        res_resource.rows.len(),
        1,
        "Resource node should be inserted"
    );

    // Verify Policy Authorizes Principal edge exists
    let res_auth_p = cozo
        .run_script(
            "?[src, tgt] := *edge{source: src, target: tgt, relation: 'authorizes'}, \
         tgt = 'urn:changeguard:principal:User::\"alice\"'",
        )
        .unwrap();
    assert_eq!(
        res_auth_p.rows.len(),
        1,
        "Policy authorizes Principal edge should exist"
    );

    // Verify Policy Authorizes Action edge exists
    let res_auth_a = cozo
        .run_script(
            "?[src, tgt] := *edge{source: src, target: tgt, relation: 'authorizes'}, \
         tgt = 'urn:changeguard:action:Action::\"view\"'",
        )
        .unwrap();
    assert_eq!(
        res_auth_a.rows.len(),
        1,
        "Policy authorizes Action edge should exist"
    );

    // Verify Policy Authorizes Resource edge exists
    let res_auth_r = cozo
        .run_script(
            "?[src, tgt] := *edge{source: src, target: tgt, relation: 'authorizes'}, \
         tgt = 'urn:changeguard:resource:Photo::\"vacation.jpg\"'",
        )
        .unwrap();
    assert_eq!(
        res_auth_r.rows.len(),
        1,
        "Policy authorizes Resource edge should exist"
    );
}

#[test]
fn test_obs_node_source_file_in_metadata() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();

    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("lib.rs"), "pub fn main() {}").unwrap();

    let obs_dir = root.join("observability");
    fs::create_dir_all(&obs_dir).unwrap();
    let openslo_yaml = r#"
apiVersion: openslo/v1
kind: SLO
metadata:
  name: checkout-latency
  displayName: Checkout Latency SLO
spec:
  service: checkout-service
  indicator:
    thresholdMetric:
      metricSource:
        type: prometheus
      metricQuery: histogram_quantile(0.99, http_request_duration_seconds_bucket)
"#;
    fs::write(obs_dir.join("checkout.yaml"), openslo_yaml).unwrap();

    let mut config = Config::default();
    config.services.definitions = vec![ServiceDefinition {
        name: "checkout-service".to_string(),
        root: "src/".to_string(),
        owners: vec![],
        runtime_name: None,
        queues: vec![],
        topics: vec![],
        rpc_endpoints: vec![],
    }];

    let db_dir = root.join(".changeguard").join("state");
    fs::create_dir_all(&db_dir).unwrap();
    let storage = StorageManager::init(db_dir.join("ledger.db").as_std_path()).unwrap();
    let mut indexer = ProjectIndexer::new(storage, root.clone(), config.clone());
    indexer.full_index().unwrap();
    indexer.build_call_graph().unwrap();

    let cozo = indexer.cozo().expect("CozoDB should be available");
    build_native_graph(indexer.storage(), cozo, "full", &config).unwrap();

    // The SLO node metadata must include source_file so observability diff can match it
    let res = cozo
        .run_script(
            "?[id, metadata] := *node{id, metadata, category: 'slo'}, \
             id = 'urn:changeguard:slo:checkout-latency'",
        )
        .unwrap();
    assert_eq!(res.rows.len(), 1, "SLO node should exist");

    let meta = &res.rows[0][1];
    let meta_json = if let cozo::DataValue::Json(j) = meta {
        j.clone()
    } else {
        panic!("metadata should be JSON");
    };
    let source_file = meta_json
        .get("source_file")
        .and_then(|v| v.as_str())
        .expect("source_file must be present in SLO node metadata");
    assert_eq!(
        source_file, "observability/checkout.yaml",
        "source_file must be repo-relative so it matches git diff paths; got: {source_file}"
    );
}

#[test]
fn test_policy_adr_security_cross_link() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();

    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("lib.rs"), "pub fn main() {}").unwrap();

    // Cedar policy fixture
    let policy_dir = root.join("policies");
    fs::create_dir_all(&policy_dir).unwrap();
    let cedar_policy = r#"
permit(
    principal == User::"alice",
    action == Action::"read",
    resource == Resource::"data"
);
"#;
    fs::write(policy_dir.join("auth.cedar"), cedar_policy).unwrap();

    let config = Config::default();

    let db_dir = root.join(".changeguard").join("state");
    fs::create_dir_all(&db_dir).unwrap();
    let storage = StorageManager::init(db_dir.join("ledger.db").as_std_path()).unwrap();
    let mut indexer = ProjectIndexer::new(storage, root.clone(), config.clone());
    indexer.full_index().unwrap();
    indexer.build_call_graph().unwrap();

    // Insert an ADR whose summary contains "security" — should link to the policy via Governs
    {
        let conn = indexer.storage_mut().get_connection_mut();
        conn.execute(
            "INSERT INTO transactions (tx_id, status, category, entity, entity_normalized, session_id, source, started_at) \
             VALUES ('adr-tx-001', 'COMMITTED', 'ARCHITECTURE', 'src/auth.rs', 'src/auth.rs', 'test-session', 'CLI', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO ledger_entries (tx_id, category, entity, entity_normalized, change_type, summary, reason, committed_at) \
             VALUES ('adr-tx-001', 'ARCHITECTURE', 'src/auth.rs', 'src/auth.rs', 'MODIFY', \
             'Adopt JWT-based security policy for all API endpoints', \
             'Compliance requirement', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO adr_metadata (adr_id, status, last_updated_at) \
             VALUES ('adr-tx-001', 'accepted', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
    }

    let cozo = indexer.cozo().expect("CozoDB should be available");
    build_native_graph(indexer.storage(), cozo, "full", &config).unwrap();

    // ADR node should exist with label containing "security"
    let adr_res = cozo
        .run_script("?[id, label] := *node{id, label, category: 'adr'}")
        .unwrap();
    assert_eq!(adr_res.rows.len(), 1, "ADR node should be inserted");
    let adr_label = if let cozo::DataValue::Str(l) = &adr_res.rows[0][1] {
        l.to_string()
    } else {
        panic!("label should be string");
    };
    assert!(
        adr_label.to_lowercase().contains("security"),
        "ADR label should contain 'security', got: {adr_label}"
    );

    // Policy → ADR Governs edge must exist
    let governs_res = cozo
        .run_script(
            "?[src, tgt] := *edge{source: src, target: tgt, relation: 'governs'}, \
             tgt = 'urn:changeguard:adr:adr-tx-001'",
        )
        .unwrap();
    assert_eq!(
        governs_res.rows.len(),
        1,
        "Policy should have a Governs edge to the security ADR"
    );
}

#[test]
fn test_policy_endpoint_cross_link() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();

    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("lib.rs"), "pub fn main() {}").unwrap();

    // Cedar policy that mentions the /api/users endpoint in its raw text
    let policy_dir = root.join("policies");
    fs::create_dir_all(&policy_dir).unwrap();
    let cedar_policy = r#"
// protects /api/users resource
permit(
    principal == User::"alice",
    action == Action::"GET",
    resource == Resource::"/api/users"
);
"#;
    fs::write(policy_dir.join("users.cedar"), cedar_policy).unwrap();

    let config = Config::default();

    let db_dir = root.join(".changeguard").join("state");
    fs::create_dir_all(&db_dir).unwrap();
    let storage = StorageManager::init(db_dir.join("ledger.db").as_std_path()).unwrap();
    let mut indexer = ProjectIndexer::new(storage, root.clone(), config.clone());
    indexer.full_index().unwrap();
    indexer.build_call_graph().unwrap();

    // Insert an api_routes row whose path_pattern appears in the policy raw text
    {
        let conn = indexer.storage_mut().get_connection_mut();
        conn.execute(
            "INSERT OR IGNORE INTO project_files (file_path, language, last_indexed_at) \
             VALUES ('src/lib.rs', 'rust', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let file_id: i64 = conn
            .query_row(
                "SELECT id FROM project_files WHERE file_path = 'src/lib.rs'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO api_routes (method, path_pattern, framework, handler_file_id, last_indexed_at) \
             VALUES ('GET', '/api/users', 'axum', ?1, '2026-01-01T00:00:00Z')",
            [file_id],
        )
        .unwrap();
    }

    let cozo = indexer.cozo().expect("CozoDB should be available");
    build_native_graph(indexer.storage(), cozo, "full", &config).unwrap();

    // Policy → endpoint ProtectedBy edge must exist
    let protected_res = cozo
        .run_script(
            "?[src, tgt] := *edge{source: src, target: tgt, relation: 'protected_by'}, \
             tgt = 'urn:changeguard:endpoint:GET:/api/users'",
        )
        .unwrap();
    assert_eq!(
        protected_res.rows.len(),
        1,
        "Policy should have a ProtectedBy edge to the /api/users endpoint"
    );
}
