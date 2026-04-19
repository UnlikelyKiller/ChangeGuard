use crate::impact::packet::ImpactPacket;

pub fn build_system_prompt() -> String {
    r#"You are ChangeGuard, an expert software engineering assistant.
Your goal is to help developers understand the impact and risk of their changes.
You have access to "Impact Packets" which describe repository state, changed files, and extracted symbols.
Provide concise, technical, and actionable insights. Focus on potential regressions, architectural shifts, and verification needs."#
        .to_string()
}

pub fn build_user_prompt(packet: &ImpactPacket, query: &str) -> String {
    let packet_json = serde_json::to_string_pretty(packet).unwrap_or_else(|_| "{}".to_string());
    
    format!(
        r#"Context:
---
Impact Packet:
{}
---

Question:
{}

Please analyze the provided context and answer the question above."#,
        packet_json, query
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::ImpactPacket;

    #[test]
    fn test_prompt_construction() {
        let packet = ImpactPacket::default();
        let query = "What is the risk?";
        let prompt = build_user_prompt(&packet, query);
        
        assert!(prompt.contains("Impact Packet:"));
        assert!(prompt.contains(query));
        assert!(prompt.contains("v1")); // schema version
    }
}
