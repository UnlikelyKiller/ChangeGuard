use changeguard::impact::analysis::analyze_risk;
use changeguard::impact::packet::{ChangedFile, FileAnalysisStatus, ImpactPacket, RiskLevel};
use changeguard::index::symbols::{Symbol, SymbolKind};
use changeguard::policy::rules::Rules;
use std::path::PathBuf;

#[test]
fn test_risk_analysis_integration() {
    let mut packet = ImpactPacket::default();

    // Scenario: Modified a public symbol
    packet.changes.push(ChangedFile {
        path: PathBuf::from("src/lib.rs"),
        status: "Modified".to_string(),
        is_staged: true,
        symbols: Some(vec![Symbol {
            name: "highly_risky".into(),
            kind: SymbolKind::Function,
            is_public: true,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: None,
            byte_start: None,
            byte_end: None,
            entrypoint_kind: None,
        }]),

        imports: None,
        runtime_usage: None,
        analysis_status: FileAnalysisStatus::default(),
        analysis_warnings: Vec::new(),
        api_routes: Vec::new(),
    });

    let rules = Rules::default();
    analyze_risk(&mut packet, &rules).unwrap();

    // Weight: 30 (public symbol) -> Medium
    assert_eq!(packet.risk_level, RiskLevel::Medium);
    assert!(
        packet
            .risk_reasons
            .iter()
            .any(|r| r.contains("Public symbol modified"))
    );
}

#[test]
fn test_risk_analysis_high_volume() {
    let mut packet = ImpactPacket::default();

    // Scenario: Many files changed
    for i in 0..10 {
        packet.changes.push(ChangedFile {
            path: PathBuf::from(format!("file_{}.rs", i)),
            status: "Added".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
        });
    }

    let rules = Rules::default();
    analyze_risk(&mut packet, &rules).unwrap();

    // Weight: 20 (volume) -> Medium (because 20 is Low, wait, 21-60 is Medium)
    // Actually in my implementation 20 is Low. Let's add more weight.

    // Add public symbols
    packet.changes[0].symbols = Some(vec![Symbol {
        name: "api".into(),
        kind: SymbolKind::Function,
        is_public: true,
        cognitive_complexity: None,
        cyclomatic_complexity: None,
        line_start: None,
        line_end: None,
        qualified_name: None,
        byte_start: None,
        byte_end: None,
        entrypoint_kind: None,
    }]);

    analyze_risk(&mut packet, &rules).unwrap();

    // Weight: 20 (volume) + 30 (public symbol) = 50 -> Medium
    assert_eq!(packet.risk_level, RiskLevel::Medium);
}

#[test]
fn test_risk_analysis_protected_and_public() {
    let mut packet = ImpactPacket::default();

    packet.changes.push(ChangedFile {
        path: PathBuf::from("Cargo.toml"),
        status: "Modified".to_string(),
        is_staged: true,
        symbols: None,
        imports: None,
        runtime_usage: None,
        analysis_status: FileAnalysisStatus::default(),
        analysis_warnings: Vec::new(),
        api_routes: Vec::new(),
    });

    packet.changes.push(ChangedFile {
        path: PathBuf::from("src/api.rs"),
        status: "Modified".to_string(),
        is_staged: true,
        symbols: Some(vec![Symbol {
            name: "highly_risky".into(),
            kind: SymbolKind::Function,
            is_public: true,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: None,
            byte_start: None,
            byte_end: None,
            entrypoint_kind: None,
        }]),

        imports: None,
        runtime_usage: None,
        analysis_status: FileAnalysisStatus::default(),
        analysis_warnings: Vec::new(),
        api_routes: Vec::new(),
    });

    let rules = Rules {
        protected_paths: vec!["Cargo.toml".to_string()],
        ..Rules::default()
    };

    analyze_risk(&mut packet, &rules).unwrap();

    // Weight: 70 (protected) + 30 (public) = 100 -> High
    assert_eq!(packet.risk_level, RiskLevel::High);
    assert!(packet.risk_reasons.len() >= 2);
}
