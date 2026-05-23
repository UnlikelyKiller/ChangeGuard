use tree_sitter::Node;

pub fn node_text<'a>(node: Node<'a>, content: &'a str) -> &'a str {
    node.utf8_text(content.as_bytes()).unwrap_or("")
}

pub fn find_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == kind {
            return Some(child);
        }
    }
    None
}
