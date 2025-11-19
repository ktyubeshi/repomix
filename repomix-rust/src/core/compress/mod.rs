use anyhow::Result;
use tree_sitter::{Parser, Language};

pub fn compress_content(content: &str, extension: &str) -> Result<String> {
    let language = match extension {
        "rs" => tree_sitter_rust::language(),
        "ts" | "tsx" => tree_sitter_typescript::language_typescript(),
        "js" | "jsx" => tree_sitter_javascript::language(),
        "py" => tree_sitter_python::language(),
        "go" => tree_sitter_go::language(),
        _ => return Ok(content.to_string()), // Unsupported language, return as is
    };

    let mut parser = Parser::new();
    parser.set_language(language)?;

    let tree = parser.parse(content, None).ok_or_else(|| anyhow::anyhow!("Failed to parse code"))?;
    let root_node = tree.root_node();

    // Simple compression: remove comments and empty lines
    // A more robust approach would be to traverse the tree and reconstruct the code without comments.
    // However, reconstructing code from AST is complex.
    // An alternative is to identify comment nodes and remove their ranges from the original string.
    
    let mut ranges_to_remove = Vec::new();
    let mut cursor = root_node.walk();
    
    // Traverse the tree to find comment nodes
    // Note: This is a simplified traversal. For full coverage we need recursive traversal.
    // tree-sitter's cursor.goto_next_sibling() / goto_first_child() logic.
    
    // Actually, let's use a query if possible, or just simple recursion.
    collect_comments(root_node, &mut ranges_to_remove, content);

    // Sort ranges by start byte in descending order to remove safely
    ranges_to_remove.sort_by(|a, b| b.0.cmp(&a.0));

    let mut compressed = content.to_string();
    for (start, end) in ranges_to_remove {
        if start < compressed.len() && end <= compressed.len() {
             compressed.replace_range(start..end, "");
        }
    }

    // Remove empty lines
    let lines: Vec<&str> = compressed.lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    
    Ok(lines.join("\n"))
}

fn collect_comments(node: tree_sitter::Node, ranges: &mut Vec<(usize, usize)>, _source: &str) {
    if node.kind() == "comment" || node.kind() == "line_comment" || node.kind() == "block_comment" {
        ranges.push((node.start_byte(), node.end_byte()));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_comments(child, ranges, _source);
    }
}
