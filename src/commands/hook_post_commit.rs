use crate::commands::helpers::{get_layout, load_ledger_config};
use crate::ledger::{
    ChangeType, CommitRequest, TransactionManager, VerificationBasis, VerificationStatus,
};
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result, miette};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct PendingHookTx {
    pub tx_id: String,
    pub commit_msg_hash: String,
    pub summary: String,
    pub reason: String,
    pub committed_at: Option<String>,
    pub risk: Option<String>,
    pub related_tickets: Option<String>,
    pub signature: Option<String>,
    pub public_key: Option<String>,
}

pub fn execute_hook_post_commit() -> Result<()> {
    let layout = get_layout()?;
    let config = load_ledger_config(&layout)?;
    let sidecar_path = layout.state_subdir().join("pending_hook_tx");

    if !sidecar_path.exists() {
        return Ok(());
    }

    let sidecar_content = fs::read_to_string(&sidecar_path).into_diagnostic()?;
    let pending: PendingHookTx = serde_json::from_str(&sidecar_content).into_diagnostic()?;

    // Verify commit hash
    let repo_root = layout.root.clone();
    let output = std::process::Command::new("git")
        .args(["log", "-1", "--format=%B"])
        .current_dir(repo_root)
        .output()
        .into_diagnostic()?;

    let current_commit_msg = String::from_utf8_lossy(&output.stdout).to_string();
    let cleaned_msg = crate::util::text::clean_commit_msg(&current_commit_msg);

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(cleaned_msg.as_bytes());
    let current_hash = hex::encode(hasher.finalize());

    if pending.commit_msg_hash != current_hash {
        eprintln!(
            "[ChangeGuard] Pending transaction {} does not match current HEAD commit. Removing stale sidecar.",
            pending.tx_id
        );
        let _ = fs::remove_file(sidecar_path);
        return Ok(());
    }
    let verification_status = if pending.risk.as_deref() == Some("TRIVIAL") {
        None
    } else {
        Some(VerificationStatus::Verified)
    };

    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let mut tx_mgr = TransactionManager::new(&mut storage, layout.root.into(), config);

    let req = CommitRequest {
        summary: pending.summary,
        reason: pending.reason,
        change_type: ChangeType::Modify,
        is_breaking: false,
        committed_at: pending.committed_at,
        verification_status,
        verification_basis: Some(VerificationBasis::ManualInspection),
        outcome_notes: None,
        issue_ref: None,
        signature: pending.signature,
        public_key: pending.public_key,
        risk: pending.risk,
        related_tickets: pending.related_tickets,
    };

    match tx_mgr.commit_change(pending.tx_id.clone(), req, false) {
        Ok(_) => {
            let _ = fs::remove_file(sidecar_path);
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "[ChangeGuard] Post-commit hook failed to promote ledger entry: {}",
                e
            );
            let _ = tx_mgr.rollback_change(
                pending.tx_id,
                "Rollback due to promotion failure".to_string(),
            );
            let _ = fs::remove_file(sidecar_path);
            Err(miette!("{}", e))
        }
    }
}
