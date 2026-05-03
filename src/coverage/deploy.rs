use crate::impact::packet::{ChangedFile, DeployManifestChange, ManifestType};
use globset::{Glob, GlobSetBuilder};
use tracing::warn;

pub fn detect_deploy_manifest_changes(
    changed_files: &[ChangedFile],
    patterns: &[String],
) -> Vec<DeployManifestChange> {
    let mut builder = GlobSetBuilder::new();

    for pat in patterns {
        match Glob::new(pat) {
            Ok(glob) => {
                builder.add(glob);
            }
            Err(e) => {
                warn!("Invalid deploy manifest glob pattern '{}': {}", pat, e);
            }
        }
    }

    let glob_set = match builder.build() {
        Ok(set) => set,
        Err(e) => {
            warn!("Failed to build deploy manifest glob set: {}", e);
            return Vec::new();
        }
    };

    let mut changes = Vec::new();
    for file in changed_files {
        if glob_set.is_match(&file.path) {
            let manifest_type = classify_manifest(&file.path);
            changes.push(DeployManifestChange {
                file: file.path.clone(),
                manifest_type,
                risk_weight: 3, // Specified in M7 risk weighting table
                is_deleted: file.status == "Deleted",
            });
        }
    }

    changes
}

fn classify_manifest(path: &std::path::Path) -> ManifestType {
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("dockerfile") {
        ManifestType::Dockerfile
    } else if path_str.contains("docker-compose") || path_str.ends_with(".yml") && path_str.contains("compose") {
        ManifestType::DockerCompose
    } else if path_str.contains("k8s") || path_str.contains("kubernetes") || path_str.contains("manifests") {
        ManifestType::Kubernetes
    } else if path_str.ends_with(".tf") || path_str.contains("terraform") {
        ManifestType::Terraform
    } else if path_str.contains("charts") || path_str.contains("helm") {
        ManifestType::Helm
    } else {
        ManifestType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::FileAnalysisStatus;
    use std::path::PathBuf;

    #[test]
    fn test_classify_manifest() {
        assert_eq!(classify_manifest(&PathBuf::from("Dockerfile")), ManifestType::Dockerfile);
        assert_eq!(classify_manifest(&PathBuf::from("docker-compose.yml")), ManifestType::DockerCompose);
        assert_eq!(classify_manifest(&PathBuf::from("infra/k8s/deployment.yaml")), ManifestType::Kubernetes);
        assert_eq!(classify_manifest(&PathBuf::from("main.tf")), ManifestType::Terraform);
        assert_eq!(classify_manifest(&PathBuf::from("charts/my-app/values.yaml")), ManifestType::Helm);
    }

    #[test]
    fn test_detect_deploy_changes() {
        let files = vec![
            ChangedFile {
                path: PathBuf::from("Dockerfile"),
                status: "Modified".to_string(),
                old_path: None,
                is_staged: true,
                symbols: None,
                imports: None,
                runtime_usage: None,
                analysis_status: FileAnalysisStatus::default(),
                analysis_warnings: Vec::new(),
                api_routes: Vec::new(),
                data_models: Vec::new(),
                ci_gates: Vec::new(),
            },
            ChangedFile {
                path: PathBuf::from("src/main.rs"),
                status: "Modified".to_string(),
                old_path: None,
                is_staged: true,
                symbols: None,
                imports: None,
                runtime_usage: None,
                analysis_status: FileAnalysisStatus::default(),
                analysis_warnings: Vec::new(),
                api_routes: Vec::new(),
                data_models: Vec::new(),
                ci_gates: Vec::new(),
            },
        ];

        let patterns = vec!["**/Dockerfile*".to_string()];
        let changes = detect_deploy_manifest_changes(&files, &patterns);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].manifest_type, ManifestType::Dockerfile);
    }

    #[test]
    fn test_detect_deploy_changes_no_match() {
        let files = vec![
            ChangedFile {
                path: PathBuf::from("src/main.rs"),
                status: "Modified".to_string(),
                old_path: None,
                is_staged: true,
                symbols: None,
                imports: None,
                runtime_usage: None,
                analysis_status: FileAnalysisStatus::default(),
                analysis_warnings: Vec::new(),
                api_routes: Vec::new(),
                data_models: Vec::new(),
                ci_gates: Vec::new(),
            },
        ];

        let patterns = vec!["**/Dockerfile*".to_string()];
        let changes = detect_deploy_manifest_changes(&files, &patterns);
        assert_eq!(changes.len(), 0);
    }
}
