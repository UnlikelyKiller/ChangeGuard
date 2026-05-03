pub fn enforce_budget(_text: &str, _max_tokens: usize) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enforce_budget_short_text_unchanged() {
        let text = "hello world";
        let result = enforce_budget(text, 100);
        assert_eq!(result, text);
    }

    #[test]
    fn enforce_budget_truncates_long_text() {
        let text = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";
        // 20 words each ~5 chars avg = ~100 chars, so about 25 tokens
        let result = enforce_budget(text, 5);
        // Should be shorter than original
        assert!(result.len() < text.len());
        // Should not be empty
        assert!(!result.is_empty());
        // Should end with a word boundary
        let original_words: Vec<&str> = text.split_whitespace().collect();
        let result_words: Vec<&str> = result.split_whitespace().collect();
        assert!(result_words.len() < original_words.len());
    }

    #[test]
    fn enforce_budget_no_mid_word_truncation() {
        let text = "hello world beautiful";
        // budget of 2 tokens = 8 chars, which falls mid-word "beautiful"
        let result = enforce_budget(text, 2);
        // Should truncate to "hello world" (before "beautiful")
        assert_eq!(result, "hello world");
    }
}
