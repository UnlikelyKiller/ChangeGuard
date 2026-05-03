use crate::local_model::client::ChatMessage;

pub fn assemble_context(
    _system_prompt: &str,
    _user_message: &str,
    _context_chunks: &[&str],
    _max_context_tokens: usize,
) -> Vec<ChatMessage> {
    todo!()
}

pub fn get_system_prompt(_mode: &str) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assemble_correct_order() {
        let messages = assemble_context(
            "You are helpful.",
            "What is Rust?",
            &["Context A", "Context B"],
            1000,
        );
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "You are helpful.");
        assert_eq!(messages[1].role, "system");
        assert_eq!(messages[1].content, "Context A");
        assert_eq!(messages[2].role, "system");
        assert_eq!(messages[2].content, "Context B");
        assert_eq!(messages[3].role, "user");
        assert_eq!(messages[3].content, "What is Rust?");
    }

    #[test]
    fn assemble_budget_trimming() {
        // Budget = 10 tokens = 40 chars
        // system_prompt: 10 chars, user: 10 chars, chunks: 30 + 30 chars
        // Total: 10 + 30 + 30 + 10 = 80 > 40, so second chunk trimmed
        let messages = assemble_context(
            "0123456789",                         // 10 chars
            "user query",                         // 10 chars
            &["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"], // 30 + 30 chars
            10,                                   // 10 tokens = 40 chars
        );
        // After trim: system(10) + chunk1(30) + user(10) = 50 > 40... still over
        // NOTE: system_prompt + user = 20, leaving 20 for chunks
        // First chunk is 30 > 20, so NO chunks fit
        // Only system + user returned
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[1].role, "user");
    }

    #[test]
    fn assemble_empty_context() {
        let messages = assemble_context("System prompt", "User question", &[], 1000);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "System prompt");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[1].content, "User question");
    }

    #[test]
    fn assemble_system_prompt_exceeds_budget() {
        let messages = assemble_context(
            "This is a very long system prompt that exceeds token budget",
            "short",
            &[],
            2, // 2 tokens = 8 chars
        );
        // system_prompt truncated to 8 chars
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "This is ");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[1].content, "short");
    }

    #[test]
    fn get_system_prompt_analyze() {
        let prompt = get_system_prompt("analyze");
        assert!(prompt.contains("impact and risk"));
    }

    #[test]
    fn get_system_prompt_suggest() {
        let prompt = get_system_prompt("suggest");
        assert!(prompt.contains("verification"));
    }

    #[test]
    fn get_system_prompt_review() {
        let prompt = get_system_prompt("review");
        assert!(prompt.contains("code reviewer"));
    }

    #[test]
    fn get_system_prompt_unknown_defaults_to_analyze() {
        let prompt = get_system_prompt("nonexistent");
        assert!(prompt.contains("impact and risk"));
    }
}
