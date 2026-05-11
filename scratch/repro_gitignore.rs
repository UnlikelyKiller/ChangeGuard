use std::fs;
use std::io::{Read, Write};

fn main() {
    let pattern = ".changeguard/";
    let ignore_content = "claude.md"; // No newline
    
    let has_newline = ignore_content.ends_with('\n') || ignore_content.ends_with('\r');
    let line_ending = if ignore_content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };

    let to_append = if has_newline {
        format!("{}{}", pattern, line_ending)
    } else {
        format!("{}{}{}", line_ending, pattern, line_ending)
    };

    let mut final_content = ignore_content.to_string();
    final_content.push_str(&to_append);
    
    println!("Content: {:?}", final_content);
    assert_eq!(final_content, "claude.md\n.changeguard/\n");
}
