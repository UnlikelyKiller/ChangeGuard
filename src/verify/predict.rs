use crate::impact::packet::ImpactPacket;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PredictionReason {
    Structural,
    Temporal,
}

impl std::fmt::Display for PredictionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Structural => write!(f, "Structural"),
            Self::Temporal => write!(f, "Temporal"),
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

pub struct Predictor;

impl Predictor {
    pub fn predict(packet: &ImpactPacket, history: &[ImpactPacket]) -> PredictionResult {
        let mut predicted = BTreeSet::new();
        let mut warnings = Vec::new();

        let changed_paths: BTreeSet<PathBuf> =
            packet.changes.iter().map(|f| f.path.clone()).collect();

        // 1. Structural Prediction (Depth 1)
        // Identify files in history (all known files) that import any of the changed files.
        for hist_packet in history {
            for hist_file in &hist_packet.changes {
                // If this file is already in the 'changes' of the current packet, it's not a "prediction"
                // (it's already being verified).
                if changed_paths.contains(&hist_file.path) {
                    continue;
                }

                if let Some(imports) = &hist_file.imports {
                    for imp in &imports.imported_from {
                        let imp_norm = imp.replace("::", "/");
                        let imp_path = Path::new(&imp_norm);
                        
                        for changed in &changed_paths {
                            // Match if the import string matches the changed file's path (heuristically)
                            // 1. Exact match (after normalization)
                            // 2. Import is a suffix of the changed path (e.g. "models/user" matches "src/models/user.rs")
                            // 3. Changed path is a suffix of the import (less likely but possible with relative paths)
                            
                            let changed_str = changed.to_string_lossy();
                            let changed_no_ext = changed.with_extension("");
                            let changed_no_ext_str = changed_no_ext.to_string_lossy();

                            if changed == imp_path || 
                               changed_no_ext == imp_path ||
                               changed_str.ends_with(&imp_norm) ||
                               changed_no_ext_str.ends_with(&imp_norm)
                            {
                                predicted.insert(PredictedFile {
                                    path: hist_file.path.clone(),
                                    reason: PredictionReason::Structural,
                                });
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 2. Temporal Prediction
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
        
        PredictionResult {
            files,
            warnings,
        }
    }
}
