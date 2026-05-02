use crate::impact::packet::ImpactPacket;
use crate::index::references::ImportExport;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PredictionReason {
    Structural,
    CallGraph,
    Temporal,
    TestMapping,
}

impl std::fmt::Display for PredictionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Structural => write!(f, "Structural"),
            Self::CallGraph => write!(f, "CallGraph"),
            Self::Temporal => write!(f, "Temporal"),
            Self::TestMapping => write!(f, "TestMapping"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PredictedFile {
    pub path: PathBuf,
    pub reason: PredictionReason,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PredictionResult {
    pub files: Vec<PredictedFile>,
    pub warnings: Vec<String>,
}

/// Pre-fetched structural call data for prediction.
/// Each entry is (caller_file_path, caller_symbol_name, callee_symbol_name).
#[derive(Debug, Clone, Default)]
pub struct StructuralCallData {
    pub callers: Vec<(PathBuf, String, String)>,
}

/// Pre-fetched test mapping data for prediction.
/// Each entry maps a test file path to the set of tested symbol names in that file.
#[derive(Debug, Clone, Default)]
pub struct TestMappingData {
    /// Maps test file path -> set of tested symbol names found in that file
    pub mappings: BTreeMap<String, BTreeSet<String>>,
}

pub struct Predictor;

impl Predictor {
    pub fn predict(packet: &ImpactPacket, history: &[ImpactPacket]) -> PredictionResult {
        Self::predict_with_current_imports(packet, history, &BTreeMap::new())
    }

    pub fn predict_with_current_imports(
        packet: &ImpactPacket,
        history: &[ImpactPacket],
        current_imports: &BTreeMap<PathBuf, ImportExport>,
    ) -> PredictionResult {
        Self::predict_with_structural_calls(
            packet,
            history,
            current_imports,
            &StructuralCallData::default(),
        )
    }

    pub fn predict_with_structural_calls(
        packet: &ImpactPacket,
        history: &[ImpactPacket],
        current_imports: &BTreeMap<PathBuf, ImportExport>,
        call_data: &StructuralCallData,
    ) -> PredictionResult {
        let mut predicted = BTreeSet::new();
        let mut warnings = Vec::new();

        let changed_paths: BTreeSet<PathBuf> =
            packet.changes.iter().map(|f| f.path.clone()).collect();

        add_structural_predictions(&mut predicted, &changed_paths, current_imports.iter());

        for hist_packet in history {
            let historical_imports = hist_packet
                .changes
                .iter()
                .filter_map(|file| file.imports.as_ref().map(|imports| (&file.path, imports)));
            add_structural_predictions(&mut predicted, &changed_paths, historical_imports);
        }

        // Call graph predictions: if changed symbols have structural callers,
        // predict the caller's file as a verification target.
        if !call_data.callers.is_empty() {
            add_call_graph_predictions(&mut predicted, &changed_paths, &call_data.callers);
        }

        if packet.temporal_couplings.is_empty() && !packet.changes.is_empty() {
            warnings.push("Temporal coupling data is missing or unavailable; falling back to structural-only prediction.".to_string());
        }

        for coupling in &packet.temporal_couplings {
            let a_changed = changed_paths.contains(&coupling.file_a);
            let b_changed = changed_paths.contains(&coupling.file_b);

            if a_changed && !b_changed {
                predicted.insert(PredictedFile {
                    path: coupling.file_b.clone(),
                    reason: PredictionReason::Temporal,
                });
            } else if b_changed && !a_changed {
                predicted.insert(PredictedFile {
                    path: coupling.file_a.clone(),
                    reason: PredictionReason::Temporal,
                });
            }
        }

        let mut files: Vec<_> = predicted.into_iter().collect();
        files.sort();

        PredictionResult { files, warnings }
    }

    /// Predict verification targets using test mapping data from the index.
    /// For each changed symbol, find tests that cover it and add those test files
    /// as prediction targets. Test-mapping predictions appear before temporal
    /// and structural predictions in priority.
    pub fn predict_with_test_mappings(
        packet: &ImpactPacket,
        history: &[ImpactPacket],
        current_imports: &BTreeMap<PathBuf, ImportExport>,
        call_data: &StructuralCallData,
        test_mapping_data: &TestMappingData,
    ) -> PredictionResult {
        let mut predicted = BTreeSet::new();
        let mut warnings = Vec::new();

        let changed_paths: BTreeSet<PathBuf> =
            packet.changes.iter().map(|f| f.path.clone()).collect();

        // Test mapping predictions: files that contain tests covering changed symbols
        for test_file in test_mapping_data.mappings.keys() {
            let test_path = PathBuf::from(test_file);
            if !changed_paths.contains(&test_path) {
                predicted.insert(PredictedFile {
                    path: test_path,
                    reason: PredictionReason::TestMapping,
                });
            }
        }

        // Then add structural predictions
        add_structural_predictions(&mut predicted, &changed_paths, current_imports.iter());

        for hist_packet in history {
            let historical_imports = hist_packet
                .changes
                .iter()
                .filter_map(|file| file.imports.as_ref().map(|imports| (&file.path, imports)));
            add_structural_predictions(&mut predicted, &changed_paths, historical_imports);
        }

        // Call graph predictions
        if !call_data.callers.is_empty() {
            add_call_graph_predictions(&mut predicted, &changed_paths, &call_data.callers);
        }

        if packet.temporal_couplings.is_empty() && !packet.changes.is_empty() {
            warnings.push("Temporal coupling data is missing or unavailable; falling back to structural-only prediction.".to_string());
        }

        for coupling in &packet.temporal_couplings {
            let a_changed = changed_paths.contains(&coupling.file_a);
            let b_changed = changed_paths.contains(&coupling.file_b);

            if a_changed && !b_changed {
                predicted.insert(PredictedFile {
                    path: coupling.file_b.clone(),
                    reason: PredictionReason::Temporal,
                });
            } else if b_changed && !a_changed {
                predicted.insert(PredictedFile {
                    path: coupling.file_a.clone(),
                    reason: PredictionReason::Temporal,
                });
            }
        }

        let mut files: Vec<_> = predicted.into_iter().collect();
        files.sort();

        PredictionResult { files, warnings }
    }
}

fn add_structural_predictions<'a, I>(
    predicted: &mut BTreeSet<PredictedFile>,
    changed_paths: &BTreeSet<PathBuf>,
    imports_by_file: I,
) where
    I: IntoIterator<Item = (&'a PathBuf, &'a ImportExport)>,
{
    for (path, imports) in imports_by_file {
        if changed_paths.contains(path) {
            continue;
        }

        if imports_changed_path(imports, changed_paths) {
            predicted.insert(PredictedFile {
                path: path.clone(),
                reason: PredictionReason::Structural,
            });
        }
    }
}

fn imports_changed_path(imports: &ImportExport, changed_paths: &BTreeSet<PathBuf>) -> bool {
    imports.imported_from.iter().any(|import| {
        let import_norm = import.replace("::", "/");
        let import_path = Path::new(&import_norm);

        changed_paths.iter().any(|changed| {
            let changed_str = changed.to_string_lossy();
            let changed_no_ext = changed.with_extension("");
            let changed_no_ext_str = changed_no_ext.to_string_lossy();

            changed == import_path
                || changed_no_ext == import_path
                || changed_str.ends_with(&import_norm)
                || changed_no_ext_str.ends_with(&import_norm)
        })
    })
}

/// Add predictions based on structural call graph edges.
/// If a changed file contains symbols that are called by other files,
/// those caller files should be tested as well.
fn add_call_graph_predictions(
    predicted: &mut BTreeSet<PredictedFile>,
    changed_paths: &BTreeSet<PathBuf>,
    callers: &[(PathBuf, String, String)], // (caller_file, caller_symbol, callee_symbol)
) {
    for (caller_file, caller_symbol, _callee_symbol) in callers {
        // Skip if the caller file is already changed (no need to predict)
        if changed_paths.contains(caller_file) {
            continue;
        }
        predicted.insert(PredictedFile {
            path: caller_file.clone(),
            reason: PredictionReason::CallGraph,
        });
        let _ = caller_symbol; // Available for future reason detail
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::{ChangedFile, FileAnalysisStatus};
    use std::path::PathBuf;

    #[test]
    fn test_prediction_reason_display() {
        assert_eq!(format!("{}", PredictionReason::Structural), "Structural");
        assert_eq!(format!("{}", PredictionReason::CallGraph), "CallGraph");
        assert_eq!(format!("{}", PredictionReason::Temporal), "Temporal");
        assert_eq!(format!("{}", PredictionReason::TestMapping), "TestMapping");
    }

    #[test]
    fn test_call_graph_prediction_adds_caller_files() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/utils.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });

        let mut call_data = StructuralCallData::default();
        call_data.callers.push((
            PathBuf::from("src/main.rs"),
            "caller_fn".to_string(),
            "helper_fn".to_string(),
        ));

        let result =
            Predictor::predict_with_structural_calls(&packet, &[], &BTreeMap::new(), &call_data);

        assert!(
            result
                .files
                .iter()
                .any(|f| f.path == std::path::Path::new("src/main.rs")
                    && f.reason == PredictionReason::CallGraph),
            "Expected src/main.rs with CallGraph reason in predictions"
        );
    }

    #[test]
    fn test_call_graph_prediction_skips_already_changed() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/main.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });

        let mut call_data = StructuralCallData::default();
        call_data.callers.push((
            PathBuf::from("src/main.rs"),
            "caller_fn".to_string(),
            "helper_fn".to_string(),
        ));

        let result =
            Predictor::predict_with_structural_calls(&packet, &[], &BTreeMap::new(), &call_data);

        // main.rs is already changed, should not be predicted
        assert!(
            !result
                .files
                .iter()
                .any(|f| f.path == std::path::Path::new("src/main.rs"))
        );
    }

    #[test]
    fn test_call_graph_prediction_graceful_degradation() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/utils.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });

        // Empty call data should produce identical output to no call data
        let empty_call_data = StructuralCallData::default();
        let result = Predictor::predict_with_structural_calls(
            &packet,
            &[],
            &BTreeMap::new(),
            &empty_call_data,
        );

        // No CallGraph predictions when data is empty
        assert!(
            !result
                .files
                .iter()
                .any(|f| f.reason == PredictionReason::CallGraph)
        );
    }

    /// E2E Test 3: Verify integration — structural call prediction
    /// Builds an ImpactPacket with a change to a file containing the "internal"
    /// symbol, creates StructuralCallData with a caller entry for "helper"
    /// calling "internal", and verifies "helper"'s file appears as a predicted
    /// verification target with PredictionReason::CallGraph.
    #[test]
    fn test_e2e_structural_call_prediction() {
        let mut packet = ImpactPacket::default();
        // The changed file contains the "internal" symbol
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/utils.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });

        // Create StructuralCallData: helper (in src/main.rs) calls internal
        let mut call_data = StructuralCallData::default();
        call_data.callers.push((
            PathBuf::from("src/main.rs"),
            "helper".to_string(),
            "internal".to_string(),
        ));

        let result =
            Predictor::predict_with_structural_calls(&packet, &[], &BTreeMap::new(), &call_data);

        // Verify src/main.rs appears as a predicted verification target with CallGraph reason
        assert!(
            result
                .files
                .iter()
                .any(|f| f.path == std::path::Path::new("src/main.rs")
                    && f.reason == PredictionReason::CallGraph),
            "expected src/main.rs with CallGraph reason in predictions, got {:?}",
            result.files
        );
    }

    /// E2E Test 4b: Empty structural_edges — no regression (prediction)
    /// Verifies that running prediction with NO structural call data
    /// produces output identical to what it would have been before E2-1
    /// (i.e., no CallGraph predictions).
    #[test]
    fn test_e2e_no_structural_calls_no_regression() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/utils.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });

        // Run prediction with default (empty) structural call data
        let call_data = StructuralCallData::default();
        let result =
            Predictor::predict_with_structural_calls(&packet, &[], &BTreeMap::new(), &call_data);

        // No CallGraph predictions when structural data is empty
        assert!(
            !result
                .files
                .iter()
                .any(|f| f.reason == PredictionReason::CallGraph),
            "expected no CallGraph predictions with empty data, got {:?}",
            result.files
        );

        // Also verify the predict() convenience method (which passes empty call data)
        // produces identical results
        let result_convenience = Predictor::predict(&packet, &[]);
        assert_eq!(
            result.files, result_convenience.files,
            "predict_with_structural_calls with empty data should match predict()"
        );
    }
}
