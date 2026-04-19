use crate::state::StateError;
use camino::{Utf8Path, Utf8PathBuf};
use miette::Result;
use std::fs;

pub const STATE_DIR: &str = ".changeguard";
pub const LOGS_DIR: &str = "logs";
pub const TMP_DIR: &str = "tmp";
pub const REPORTS_DIR: &str = "reports";
pub const STATE_SUBDIR: &str = "state";
pub const CONFIG_FILE: &str = "config.toml";
pub const RULES_FILE: &str = "rules.toml";

pub struct Layout {
    pub root: Utf8PathBuf,
    pub state_dir: Utf8PathBuf,
}

impl Layout {
    pub fn new<P: AsRef<Utf8Path>>(root: P) -> Self {
        let root = root.as_ref().to_path_buf();
        let state_dir = root.join(STATE_DIR);
        Self { root, state_dir }
    }

    pub fn logs_dir(&self) -> Utf8PathBuf {
        self.state_dir.join(LOGS_DIR)
    }

    pub fn tmp_dir(&self) -> Utf8PathBuf {
        self.state_dir.join(TMP_DIR)
    }

    pub fn reports_dir(&self) -> Utf8PathBuf {
        self.state_dir.join(REPORTS_DIR)
    }

    pub fn state_subdir(&self) -> Utf8PathBuf {
        self.state_dir.join(STATE_SUBDIR)
    }

    pub fn config_file(&self) -> Utf8PathBuf {
        self.state_dir.join(CONFIG_FILE)
    }

    pub fn rules_file(&self) -> Utf8PathBuf {
        self.state_dir.join(RULES_FILE)
    }

    pub fn ensure_state_dir(&self) -> Result<()> {
        self.ensure_dir(&self.state_dir)?;
        self.ensure_dir(&self.logs_dir())?;
        self.ensure_dir(&self.tmp_dir())?;
        self.ensure_dir(&self.reports_dir())?;
        self.ensure_dir(&self.state_subdir())?;
        Ok(())
    }

    fn ensure_dir(&self, path: &Utf8Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| StateError::MkdirFailed {
                path: path.to_string(),
                source: e,
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_layout_creation() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let layout = Layout::new(root);

        assert_eq!(layout.state_dir, root.join(STATE_DIR));
        assert_eq!(layout.logs_dir(), root.join(STATE_DIR).join(LOGS_DIR));
        assert_eq!(layout.config_file(), root.join(STATE_DIR).join(CONFIG_FILE));
        assert_eq!(layout.rules_file(), root.join(STATE_DIR).join(RULES_FILE));
    }

    #[test]
    fn test_ensure_state_dir() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let layout = Layout::new(root);

        layout.ensure_state_dir().unwrap();

        assert!(layout.state_dir.exists());
        assert!(layout.logs_dir().exists());
        assert!(layout.tmp_dir().exists());
        assert!(layout.reports_dir().exists());
        assert!(layout.state_subdir().exists());
    }
}
