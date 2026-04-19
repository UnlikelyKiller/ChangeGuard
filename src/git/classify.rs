use crate::git::{ChangeType, FileChange};
use gix::Repository;

pub fn classify_status(_repo: &Repository, item: &gix::status::Item) -> Option<Vec<FileChange>> {
    let mut results = Vec::new();

    match item {
        gix::status::Item::IndexWorktree(change) => {
            let path = gix::path::from_bstr(change.rela_path()).to_path_buf();
            // This is Index -> Worktree (Unstaged)
            results.push(FileChange {
                path,
                change_type: ChangeType::Modified,
                is_staged: false,
            });
        }
        gix::status::Item::TreeIndex(change) => {
            let path = gix::path::from_bstr(change.location()).to_path_buf();
            // Tree -> Index is Staged
            results.push(FileChange {
                path,
                change_type: ChangeType::Modified,
                is_staged: true,
            });
        }
    }

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}
