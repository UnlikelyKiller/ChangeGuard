use crate::gemini::modes::{GeminiMode, build_system_prompt, build_user_prompt};
use crate::impact::packet::ImpactPacket;

pub fn build_system_prompt_legacy() -> String {
    build_system_prompt(GeminiMode::Analyze)
}

pub fn build_user_prompt_legacy(packet: &ImpactPacket, query: &str) -> String {
    build_user_prompt(GeminiMode::Analyze, packet, query, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::ImpactPacket;

    #[test]
    fn test_prompt_construction() {
        let packet = ImpactPacket::default();
        let query = "What is the risk?";
        let prompt = build_user_prompt_legacy(&packet, query);

        assert!(prompt.contains("Impact Packet:"));
        assert!(prompt.contains(query));
        assert!(prompt.contains("v1")); // schema version
    }
}
