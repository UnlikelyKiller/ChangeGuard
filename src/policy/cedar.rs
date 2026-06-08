use regex::Regex;
use crate::state::graph_kinds::NodeKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CedarPolicy {
    pub effect: String,
    pub principal: Option<String>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub conditions: Option<String>,
    pub raw: String,
}

pub struct CedarImporter {
    policy_regex: Regex,
}

impl CedarImporter {
    pub fn new() -> Self {
        // Basic regex to match permit/forbid and the principal, action, resource
        // permit(principal, action, resource)
        // This is a heuristic and doesn't handle all Cedar syntax (like context or complex expressions)
        let policy_regex = Regex::new(r#"(?x)
            (permit|forbid)\s*\(
            \s*principal\s*(?:==|in)\s*([^,)]+),
            \s*action\s*(?:==|in)\s*([^,)]+),
            \s*resource\s*(?:==|in)\s*([^,)]+)
            \s*\)
        "#).unwrap();

        Self { policy_regex }
    }

    pub fn parse(&self, content: &str) -> Vec<CedarPolicy> {
        let mut policies = Vec::new();
        for cap in self.policy_regex.captures_iter(content) {
            policies.push(CedarPolicy {
                effect: cap[1].to_string(),
                principal: Some(cap[2].trim().to_string()),
                action: Some(cap[3].trim().to_string()),
                resource: Some(cap[4].trim().to_string()),
                conditions: None, // Regex above doesn't capture conditions yet
                raw: cap[0].to_string(),
            });
        }
        policies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_policy() {
        let content = r#"permit(principal == User::"alice", action == Action::"view", resource == Photo::"vacation.jpg");"#;
        let importer = CedarImporter::new();
        let policies = importer.parse(content);
        assert_eq!(policies.len(), 1);
        assert_eq!(policies[0].effect, "permit");
        assert_eq!(policies[0].principal, Some("User::\"alice\"".to_string()));
        assert_eq!(policies[0].action, Some("Action::\"view\"".to_string()));
        assert_eq!(policies[0].resource, Some("Photo::\"vacation.jpg\"".to_string()));
    }

    #[test]
    fn test_parse_multiple_policies() {
        let content = r#"
            permit(principal == User::"alice", action == Action::"view", resource == Photo::"vacation.jpg");
            forbid(principal == User::"bob", action == Action::"delete", resource == Photo::"vacation.jpg");
        "#;
        let importer = CedarImporter::new();
        let policies = importer.parse(content);
        assert_eq!(policies.len(), 2);
    }
}
