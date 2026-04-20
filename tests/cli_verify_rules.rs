use camino::Utf8Path;
use changeguard::commands::verify::execute_verify;
use changeguard::state::layout::Layout;
use std::fs;
use tempfile::tempdir;

mod common;
use common::{DirGuard, cwd_lock};

#[test]
fn test_verify_invalid_rules_fail_visibly() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let _guard = DirGuard::from_utf8(root);

    let layout = Layout::new(root);
    layout.ensure_state_dir().unwrap();
    fs::write(
        layout.rules_file(),
        "[global]\nmode = \"analyze\"\n\n[[overrides]]\npattern = \"[\"\n",
    )
    .unwrap();

    let err = execute_verify(None, 5, false).unwrap_err();
    assert!(format!("{err:?}").contains("Invalid glob pattern"));
}
