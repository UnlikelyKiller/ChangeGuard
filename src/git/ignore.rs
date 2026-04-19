use crate::git::GitError;
use camino::Utf8Path;
use miette::Result;
use std::fs;
use std::io::{Read, Write};

pub fn add_to_gitignore(root: &Utf8Path, pattern: &str) -> Result<bool> {
    let ignore_path = root.join(".gitignore");
    
    if !ignore_path.exists() {
        let mut file = fs::File::create(&ignore_path).map_err(|e| GitError::WriteIgnoreFailed {
            path: ignore_path.to_string(),
            source: e,
        })?;
        let content = format!("{}\n", pattern);
        file.write_all(content.as_bytes()).map_err(|e| GitError::WriteIgnoreFailed {
            path: ignore_path.to_string(),
            source: e,
        })?;
        return Ok(true);
    }

    let mut content = String::new();
    let mut file = fs::File::open(&ignore_path).map_err(|e| GitError::ReadIgnoreFailed {
        path: ignore_path.to_string(),
        source: e,
    })?;
    file.read_to_string(&mut content).map_err(|e| GitError::ReadIgnoreFailed {
        path: ignore_path.to_string(),
        source: e,
    })?;

    // Check if pattern is already there
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == pattern || trimmed == pattern.trim_end_matches('/') {
            return Ok(false);
        }
    }

    // Append pattern, trying to respect existing line endings
    let has_newline = content.ends_with('\n') || content.ends_with('\r');
    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };

    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&ignore_path)
        .map_err(|e| GitError::WriteIgnoreFailed {
            path: ignore_path.to_string(),
            source: e,
        })?;

    let to_append = if has_newline {
        format!("{}{}", pattern, line_ending)
    } else {
        format!("{}{}{}", line_ending, pattern, line_ending)
    };

    file.write_all(to_append.as_bytes()).map_err(|e| GitError::WriteIgnoreFailed {
        path: ignore_path.to_string(),
        source: e,
    })?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_add_to_gitignore_new_file() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        
        let changed = add_to_gitignore(root, ".changeguard/").unwrap();
        assert!(changed);
        
        let content = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert_eq!(content, ".changeguard/\n");
    }

    #[test]
    fn test_add_to_gitignore_existing_no_newline() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        fs::write(root.join(".gitignore"), "target").unwrap();
        
        let changed = add_to_gitignore(root, ".changeguard/").unwrap();
        assert!(changed);
        
        let content = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert_eq!(content, "target\n.changeguard/\n");
    }

    #[test]
    fn test_add_to_gitignore_idempotent() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        fs::write(root.join(".gitignore"), "target\n.changeguard/\n").unwrap();
        
        let changed = add_to_gitignore(root, ".changeguard/").unwrap();
        assert!(!changed);
        
        let content = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert_eq!(content, "target\n.changeguard/\n");
    }
}
