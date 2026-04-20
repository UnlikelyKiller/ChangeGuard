use camino::Utf8PathBuf;
use changeguard::config::model::TemporalConfig;
use changeguard::git::GitError;
use changeguard::impact::temporal::{CommitFileSet, HistoryProvider, TemporalEngine};
use std::collections::HashSet;

struct MockHistoryProvider {
    history: Vec<CommitFileSet>,
}

impl HistoryProvider for MockHistoryProvider {
    fn get_history(&self, _max_commits: usize) -> Result<Vec<CommitFileSet>, GitError> {
        Ok(self.history.clone())
    }
}

#[test]
fn test_temporal_coupling_logic() {
    let mut history = Vec::new();
    
    // File A and B changed together in 10 commits
    for _ in 0..10 {
        let mut files = HashSet::new();
        files.insert(Utf8PathBuf::from("src/a.rs"));
        files.insert(Utf8PathBuf::from("src/b.rs"));
        history.push(CommitFileSet { files, is_merge: false });
    }

    let config = TemporalConfig {
        max_commits: 100,
        max_files_per_commit: 50,
        coupling_threshold: 0.8,
    };

    let provider = MockHistoryProvider { history };
    let engine = TemporalEngine::new(provider, config);
    let couplings = engine.calculate_couplings().unwrap();

    assert_eq!(couplings.len(), 2);
    
    // A -> B coupling
    assert_eq!(couplings[0].file_a.to_str().unwrap(), "src/a.rs");
    assert_eq!(couplings[0].file_b.to_str().unwrap(), "src/b.rs");
    assert_eq!(couplings[0].score, 1.0);

    // B -> A coupling
    assert_eq!(couplings[1].file_a.to_str().unwrap(), "src/b.rs");
    assert_eq!(couplings[1].file_b.to_str().unwrap(), "src/a.rs");
    assert_eq!(couplings[1].score, 1.0);
}

#[test]
fn test_temporal_coupling_threshold() {
    let mut history = Vec::new();
    
    // File A changes in 10 commits
    // File B changes in 5 of those commits
    for i in 0..10 {
        let mut files = HashSet::new();
        files.insert(Utf8PathBuf::from("src/a.rs"));
        if i < 5 {
            files.insert(Utf8PathBuf::from("src/b.rs"));
        }
        history.push(CommitFileSet { files, is_merge: false });
    }

    let config = TemporalConfig {
        max_commits: 100,
        max_files_per_commit: 50,
        coupling_threshold: 0.6,
    };

    let provider = MockHistoryProvider { history };
    let engine = TemporalEngine::new(provider, config);
    let couplings = engine.calculate_couplings().unwrap();

    // Score A -> B = 5/10 = 0.5 (below 0.6)
    // Score B -> A = 5/5 = 1.0 (above 0.6)
    assert_eq!(couplings.len(), 1);
    assert_eq!(couplings[0].file_a.to_str().unwrap(), "src/b.rs");
    assert_eq!(couplings[0].file_b.to_str().unwrap(), "src/a.rs");
    assert_eq!(couplings[0].score, 1.0);
}

#[test]
fn test_insufficient_history_error() {
    let mut history = Vec::new();
    for _ in 0..5 {
        history.push(CommitFileSet { files: HashSet::new(), is_merge: false });
    }

    let config = TemporalConfig::default();
    let provider = MockHistoryProvider { history };
    let engine = TemporalEngine::new(provider, config);
    let result = engine.calculate_couplings();

    assert!(matches!(result, Err(GitError::InsufficientHistory { .. })));
}
