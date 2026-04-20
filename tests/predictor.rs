use changeguard::impact::packet::{ImpactPacket, ChangedFile, TemporalCoupling, FileAnalysisStatus};
use changeguard::index::references::ImportExport;
use changeguard::verify::predict::{Predictor, PredictionReason};
use std::path::PathBuf;

#[test]
fn test_structural_prediction() {
    let mut current = ImpactPacket::default();
    current.changes.push(ChangedFile {
        path: PathBuf::from("src/models/user.rs"),
        status: "Modified".to_string(),
        is_staged: true,
        symbols: None,
        imports: None,
        runtime_usage: None,
        analysis_status: FileAnalysisStatus::default(),
        analysis_warnings: Vec::new(),
    });

    let mut history = ImpactPacket::default();
    history.changes.push(ChangedFile {
        path: PathBuf::from("src/handlers/auth.rs"),
        status: "Modified".to_string(),
        is_staged: false,
        symbols: None,
        imports: Some(ImportExport {
            imported_from: vec!["src/models/user.rs".to_string()],
            exported_symbols: vec![],
        }),
        runtime_usage: None,
        analysis_status: FileAnalysisStatus::default(),
        analysis_warnings: Vec::new(),
    });

    let result = Predictor::predict(&current, &[history]);
    
    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[0].path, PathBuf::from("src/handlers/auth.rs"));
    assert_eq!(result.files[0].reason, PredictionReason::Structural);
}

#[test]
fn test_temporal_prediction() {
    let mut current = ImpactPacket::default();
    current.changes.push(ChangedFile {
        path: PathBuf::from("src/a.rs"),
        status: "Modified".to_string(),
        is_staged: true,
        symbols: None,
        imports: None,
        runtime_usage: None,
        analysis_status: FileAnalysisStatus::default(),
        analysis_warnings: Vec::new(),
    });
    current.temporal_couplings.push(TemporalCoupling {
        file_a: PathBuf::from("src/a.rs"),
        file_b: PathBuf::from("src/b.rs"),
        score: 0.9,
    });

    let result = Predictor::predict(&current, &[]);
    
    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[0].path, PathBuf::from("src/b.rs"));
    assert_eq!(result.files[0].reason, PredictionReason::Temporal);
}

#[test]
fn test_deduplication_and_sorting() {
    let mut current = ImpactPacket::default();
    current.changes.push(ChangedFile {
        path: PathBuf::from("src/a.rs"),
        status: "Modified".to_string(),
        is_staged: true,
        symbols: None,
        imports: None,
        runtime_usage: None,
        analysis_status: FileAnalysisStatus::default(),
        analysis_warnings: Vec::new(),
    });
    // Temporal predicts B
    current.temporal_couplings.push(TemporalCoupling {
        file_a: PathBuf::from("src/a.rs"),
        file_b: PathBuf::from("src/b.rs"),
        score: 0.9,
    });

    // Structural also predicts B
    let mut history = ImpactPacket::default();
    history.changes.push(ChangedFile {
        path: PathBuf::from("src/b.rs"),
        status: "Modified".to_string(),
        is_staged: false,
        symbols: None,
        imports: Some(ImportExport {
            imported_from: vec!["src/a.rs".to_string()],
            exported_symbols: vec![],
        }),
        runtime_usage: None,
        analysis_status: FileAnalysisStatus::default(),
        analysis_warnings: Vec::new(),
    });

    let result = Predictor::predict(&current, &[history]);
    
    // B should appear twice if reasons are different but they are in a BTreeSet of PredictedFile
    // Wait, PredictedFile Eq/Ord includes reason. So it SHOULD appear twice if both reasons exist.
    // Let's verify intended behavior. Usually we want to know ALL reasons.
    assert_eq!(result.files.len(), 2);
    assert_eq!(result.files[0].path, PathBuf::from("src/b.rs"));
    assert_eq!(result.files[1].path, PathBuf::from("src/b.rs"));
}
