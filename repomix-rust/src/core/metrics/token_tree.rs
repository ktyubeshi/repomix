use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileTokenInfo {
    pub name: String,
    pub tokens: usize,
}

#[derive(Debug, Clone)]
pub struct TokenTreeNode {
    pub token_sum: usize,
    pub files: Vec<FileTokenInfo>,
    pub children: BTreeMap<String, TokenTreeNode>,
}

impl Default for TokenTreeNode {
    fn default() -> Self {
        Self {
            token_sum: 0,
            files: Vec::new(),
            children: BTreeMap::new(),
        }
    }
}

pub fn build_token_tree(entries: &[(PathBuf, usize)]) -> TokenTreeNode {
    let mut root = TokenTreeNode::default();

    for (path, tokens) in entries {
        let parts: Vec<String> = path
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();

        if parts.is_empty() {
            continue;
        }

        let file_name = match parts.last() {
            Some(name) => name.clone(),
            None => continue,
        };

        let mut current = &mut root;
        for part in parts.iter().take(parts.len() - 1) {
            current = current
                .children
                .entry(part.clone())
                .or_insert_with(TokenTreeNode::default);
        }

        current.files.push(FileTokenInfo {
            name: file_name,
            tokens: *tokens,
        });
    }

    calculate_sums(&mut root);
    root
}

fn calculate_sums(node: &mut TokenTreeNode) -> usize {
    let file_tokens: usize = node.files.iter().map(|f| f.tokens).sum();
    let child_tokens: usize = node.children.values_mut().map(calculate_sums).sum();
    node.token_sum = file_tokens + child_tokens;
    node.token_sum
}

pub fn to_json_value(node: &TokenTreeNode) -> Value {
    let mut map = serde_json::Map::new();

    if !node.files.is_empty() {
        map.insert(
            "_files".to_string(),
            json!(node
                .files
                .iter()
                .map(|f| json!({"name": f.name, "tokens": f.tokens}))
                .collect::<Vec<_>>()),
        );
    }
    map.insert("_tokenSum".to_string(), json!(node.token_sum));

    for (name, child) in &node.children {
        map.insert(name.clone(), to_json_value(child));
    }

    Value::Object(map)
}

pub fn render_token_tree(node: &TokenTreeNode, min_tokens: usize) -> Vec<String> {
    let mut lines = Vec::new();
    render_node(node, "", true, min_tokens, &mut lines);
    lines
}

fn render_node(
    node: &TokenTreeNode,
    prefix: &str,
    is_root: bool,
    min_tokens: usize,
    lines: &mut Vec<String>,
) {
    let mut dirs: Vec<(&String, &TokenTreeNode)> = node
        .children
        .iter()
        .filter(|(_, child)| child.token_sum >= min_tokens)
        .collect();
    let mut files: Vec<&FileTokenInfo> = node
        .files
        .iter()
        .filter(|f| f.tokens >= min_tokens)
        .collect();

    files.sort_by(|a, b| a.name.cmp(&b.name));
    dirs.sort_by(|a, b| a.0.cmp(b.0));

    for (idx, file) in files.iter().enumerate() {
        let is_last_file = idx == files.len() - 1 && dirs.is_empty();
        let connector = if is_last_file {
            "└── "
        } else {
            "├── "
        };
        let line = format!(
            "{}{}{} ({} tokens)",
            prefix, connector, file.name, file.tokens
        );
        lines.push(line);
    }

    for (idx, (name, child)) in dirs.iter().enumerate() {
        let is_last = idx == dirs.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let line = format!(
            "{}{}{}{} ({} tokens)",
            prefix, connector, name, "/", child.token_sum
        );
        lines.push(line);

        let child_prefix = if is_root && prefix.is_empty() {
            if is_last {
                "    ".to_string()
            } else {
                "│   ".to_string()
            }
        } else {
            format!("{}{}", prefix, if is_last { "    " } else { "│   " })
        };

        render_node(child, &child_prefix, false, min_tokens, lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_tree_and_renders() {
        let entries = vec![
            (PathBuf::from("src/main.rs"), 120),
            (PathBuf::from("src/lib.rs"), 80),
            (PathBuf::from("README.md"), 30),
        ];
        let tree = build_token_tree(&entries);
        assert_eq!(tree.token_sum, 230);
        let lines = render_token_tree(&tree, 0);
        assert!(lines.iter().any(|l| l.contains("README.md")));
        assert!(lines.iter().any(|l| l.contains("src/")));
    }
}
