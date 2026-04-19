use gix::Repository;
use gix::bstr::BString;
use crate::git::{FileChange, GitError};
use anyhow::Result;
use crate::git::classify::classify_status;

pub fn get_repo_status(repo: &Repository) -> Result<Vec<FileChange>, GitError> {
    let mut file_changes = Vec::new();
    
    // Configure status to include untracked files and handle renames if possible
    // In gix 0.81.0, repo.status() requires progress.
    let status = repo.status(gix::progress::Discard).map_err(|e| GitError::MetadataError { source: e.into() })?
        .index_worktree_rewrites(None);
    
    let items = status.into_iter(Vec::<BString>::new()).map_err(|e| GitError::MetadataError { source: e.into() })?;

    for item in items {
        let item = item.map_err(|e| GitError::MetadataError { source: e.into() })?;
        if let Some(changes) = classify_status(repo, &item) {
            file_changes.extend(changes);
        }
    }

    // Sort changes by path for determinism
    file_changes.sort_by(|a, b| a.path.cmp(&b.path));
    
    Ok(file_changes)
}
