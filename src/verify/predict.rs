use crate::impact::packet::ImpactPacket;
use crate::index::references::ImportExport;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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
        Self::predict_with_current_imports(packet, history, &BTreeMap::new())
    }

    pub fn predict_with_current_imports(
        packet: &ImpactPacket,
        history: &[ImpactPacket],
        current_imports: &BTreeMap<PathBuf, ImportExport>,
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
