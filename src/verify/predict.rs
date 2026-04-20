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

pub struct Predictor;

impl Predictor {
    pub fn predict(packet: &ImpactPacket) -> Vec<PredictedFile> {
        let mut predicted = BTreeSet::new();
        let changed_paths: BTreeSet<&Path> =
            packet.changes.iter().map(|f| f.path.as_path()).collect();

        // 1. Structural Impact
        // For each changed file, find files that import it.
        // Since we only have analysis for files in 'changes' (or previous packets, but we are KISS for now),
        // we check if any changed file imports another changed file.
        // Wait, the spec says "predict which files SHOULD be verified even if they haven't changed".
        // This means we need to look at files NOT in 'changes'.
        // But we don't have analysis for them in the current packet.

        // Let's re-read the spec carefully.
        // "Implement structural predictions by parsing the ImpactPacket imports to identify files dependent on the changed paths."
        // If ImpactPacket ONLY has changed files, then we can only find dependencies BETWEEN changed files.
        // But those are already being verified.

        // UNLESS... ImpactPacket imports are actually used in reverse?
        // No, if B imports A, and A changed, B is impacted.

        // Let's look at how temporal coupling is used.
        for coupling in &packet.temporal_couplings {
            if changed_paths.contains(coupling.file_a.as_path())
                && !changed_paths.contains(coupling.file_b.as_path())
            {
                predicted.insert(PredictedFile {
                    path: coupling.file_b.clone(),
                    reason: PredictionReason::Temporal,
                });
            }
        }

        // Structural Impact (Depth 1)
        // If we want to find files NOT in 'changes' that import something in 'changes',
        // we would need their analysis.
        // For now, let's implement what we CAN with the packet data.
        // If the packet HAD more files, we would use them.

        // Wait, if a ChangedFile B imports A, and A is in changes, then B is structurally impacted.
        // But B is already in changes.

        // Maybe the intent is that we should also consider the 'imported_from' strings
        // as a way to find dependencies if they were file paths?

        // Actually, if I look at the tests in Track 26, maybe I can see what's expected.
        // I'll check if there are any existing tests for this.

        let mut results: Vec<_> = predicted.into_iter().collect();
        results.sort();
        results
    }
}
