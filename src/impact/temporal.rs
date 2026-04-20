use crate::config::model::TemporalConfig;
use crate::git::GitError;
use crate::impact::packet::TemporalCoupling;
use camino::Utf8PathBuf;
use gix::Repository;
use gix::object::tree::diff::ChangeDetached;
use miette::Result;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct CommitFileSet {
    pub files: HashSet<Utf8PathBuf>,
    pub is_merge: bool,
}

pub trait HistoryProvider {
    fn get_history(&self, max_commits: usize, all_parents: bool) -> Result<Vec<CommitFileSet>, GitError>;
}

#[derive(Clone)]
pub struct GixHistoryProvider<'repo> {
    repo: &'repo Repository,
}

impl<'repo> GixHistoryProvider<'repo> {
    pub fn new(repo: &'repo Repository) -> Self {
        Self { repo }
    }
}

impl<'repo> HistoryProvider for GixHistoryProvider<'repo> {
    fn get_history(&self, max_commits: usize, all_parents: bool) -> Result<Vec<CommitFileSet>, GitError> {
        if self.repo.is_shallow() {
            return Err(GitError::ShallowClone);
        }

        let head = self
            .repo
            .head_commit()
            .map_err(|e| GitError::MetadataError { source: e.into() })?;

        let mut history = Vec::new();
        let mut walk = head.id().ancestors();
        
        if !all_parents {
            walk = walk.first_parent_only();
        } else {
            walk = walk.sorting(gix::revision::walk::Sorting::BreadthFirst);
        }

        let walk = walk.all().map_err(|e| GitError::MetadataError { source: e.into() })?;

        for res in walk {
            if history.len() >= max_commits {
                break;
            }

            let info = match res {
                Ok(info) => info,
                Err(e) => {
                    tracing::warn!("Failed to retrieve commit info during history walk: {e}");
                    continue;
                }
            };
            
            let commit = match info.id().object().and_then(|obj| Ok(obj.into_commit())) {
                Ok(commit) => commit,
                Err(e) => {
                    tracing::warn!("Failed to retrieve commit object for {}: {e}", info.id());
                    continue;
                }
            };

            let is_merge = commit.parent_ids().count() > 1;
            let mut files = HashSet::new();

            if !is_merge {
                let current_tree = match commit.tree() {
                    Ok(tree) => tree,
                    Err(e) => {
                        tracing::warn!("Failed to retrieve tree for commit {}: {e}", info.id());
                        continue;
                    }
                };

                let parent_id = commit.parent_ids().next();
                let parent_tree = if let Some(p_id) = parent_id {
                    match p_id.object().and_then(|obj| Ok(obj.into_commit().tree())) {
                        Ok(Ok(tree)) => tree,
                        _ => {
                            tracing::warn!("Failed to retrieve parent tree for commit {}: parent {}", info.id(), p_id);
                            self.repo.empty_tree()
                        }
                    }
                } else {
                    self.repo.empty_tree()
                };

                let changes = match self.repo.diff_tree_to_tree(Some(&parent_tree), Some(&current_tree), None) {
                    Ok(changes) => changes,
                    Err(e) => {
                        tracing::warn!("Failed to diff tree for commit {}: {e}", info.id());
                        continue;
                    }
                };

                for change in changes {
                    match change {
                        ChangeDetached::Addition { location, .. }
                        | ChangeDetached::Deletion { location, .. }
                        | ChangeDetached::Modification { location, .. } => {
                            files.insert(Utf8PathBuf::from(
                                String::from_utf8_lossy(&location).into_owned(),
                            ));
                        }
                        ChangeDetached::Rewrite {
                            location,
                            source_location,
                            ..
                        } => {
                            files.insert(Utf8PathBuf::from(
                                String::from_utf8_lossy(&location).into_owned(),
                            ));
                            files.insert(Utf8PathBuf::from(
                                String::from_utf8_lossy(&source_location).into_owned(),
                            ));
                        }
                    }
                }
            }

            history.push(CommitFileSet { files, is_merge });
        }

        Ok(history)
    }
}

pub struct TemporalEngine<P: HistoryProvider> {
    provider: P,
    config: TemporalConfig,
}

impl<P: HistoryProvider> TemporalEngine<P> {
    pub fn new(provider: P, config: TemporalConfig) -> Self {
        Self { provider, config }
    }

    pub fn calculate_couplings(&self) -> Result<Vec<TemporalCoupling>, GitError> {
        let history = self.provider.get_history(self.config.max_commits, self.config.all_parents)?;

        let mut commit_count = 0;
        let mut file_commit_map: HashMap<Utf8PathBuf, HashSet<usize>> = HashMap::new();
        let mut commit_id = 0;

        for commit_set in history {
            // Skip merge commits
            if commit_set.is_merge {
                continue;
            }

            if commit_set.files.len() > self.config.max_files_per_commit
                || commit_set.files.is_empty()
            {
                continue;
            }

            for file in commit_set.files {
                file_commit_map.entry(file).or_default().insert(commit_id);
            }

            commit_count += 1;
            commit_id += 1;
        }

        if commit_count < 10 {
            return Err(GitError::InsufficientHistory {
                found: commit_count,
                required: 10,
            });
        }

        let mut couplings = Vec::new();
        let mut files: Vec<_> = file_commit_map.keys().cloned().collect();
        files.sort_unstable();

        for i in 0..files.len() {
            for j in i + 1..files.len() {
                let file_a = &files[i];
                let file_b = &files[j];

                let commits_a = &file_commit_map[file_a];
                let commits_b = &file_commit_map[file_b];

                let intersection: HashSet<_> = commits_a.intersection(commits_b).collect();
                let common_count = intersection.len() as f32;

                let score_a = common_count / commits_a.len() as f32;
                let score_b = common_count / commits_b.len() as f32;

                if score_a > self.config.coupling_threshold {
                    couplings.push(TemporalCoupling {
                        file_a: file_a.clone().into(),
                        file_b: file_b.clone().into(),
                        score: score_a,
                    });
                }

                if score_b > self.config.coupling_threshold {
                    couplings.push(TemporalCoupling {
                        file_a: file_b.clone().into(),
                        file_b: file_a.clone().into(),
                        score: score_b,
                    });
                }
            }
        }

        // Deterministic sorting by score (desc) then paths
        couplings.sort_by(|a, b| {
            b.score.partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.file_a.cmp(&b.file_a))
                .then_with(|| a.file_b.cmp(&b.file_b))
        });
        
        Ok(couplings)
    }
}
