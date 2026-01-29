use crate::config::schema::RepomixConfig;
use crate::core::metrics::token_tree::{to_json_value, TokenTreeNode};
use crate::core::pack::FileStats;
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct RepomixOutput {
    pub content: String,
}

pub struct FormatContext<'a> {
    pub files: &'a HashMap<PathBuf, String>,
    pub sorted_paths: &'a [PathBuf],
    pub tree_paths: &'a [PathBuf],
    pub top_files: &'a [FileStats],
    pub token_count_tree: Option<&'a TokenTreeNode>,
    pub token_count: usize,
    pub total_chars: usize,
}

fn add_line_numbers(content: &str) -> String {
    let mut numbered = String::new();
    for (i, line) in content.lines().enumerate() {
        numbered.push_str(&format!("{:4}: {}\n", i + 1, line));
    }
    numbered
}

pub fn format(config: &RepomixConfig, ctx: FormatContext) -> Result<RepomixOutput> {
    let mut output = String::new();
    let style = config.output.style.to_string(); // Get String
    let style_str = style.as_str(); // Get &str

    match style_str {
        "markdown" => format_markdown(&mut output, &ctx, config),
        "plain" => format_plain(&mut output, &ctx, config),
        "json" => format_json(&mut output, &ctx, config)?,
        _ => format_xml_full(&mut output, &ctx, config)?,
    }

    Ok(RepomixOutput { content: output })
}

fn format_xml_full(output: &mut String, ctx: &FormatContext, config: &RepomixConfig) -> Result<()> {
    output.push_str(&generate_header(config));
    output.push_str("\n\n");

    // File Summary
    output.push_str("<file_summary>\n");
    output.push_str("This section contains a summary of this file.\n\n");
    output.push_str("<purpose>\n");
    output.push_str(&generate_summary_purpose(config));
    output.push('\n');
    output.push_str("</purpose>\n\n");

    output.push_str("<file_format>\n");
    output.push_str(&generate_summary_file_format_text());
    output.push('\n');
    output.push_str("</file_format>\n\n");

    output.push_str("<usage_guidelines>\n");
    output.push_str(&generate_summary_usage_guidelines(config));
    output.push('\n');
    output.push_str("</usage_guidelines>\n\n");

    output.push_str("<notes>\n");
    output.push_str(&generate_summary_notes(config));
    output.push('\n');
    output.push_str("</notes>\n\n");
    output.push_str("</file_summary>\n\n");

    // User Provided Header
    if let Some(header) = &config.output.header_text {
        output.push_str("<user_provided_header>\n");
        output.push_str(header);
        if !header.ends_with('\n') {
            output.push('\n');
        }
        output.push_str("</user_provided_header>\n\n");
    }

    // Directory Structure
    if config.output.directory_structure {
        output.push_str("<directory_structure>\n");
        let structure = generate_directory_structure(ctx.tree_paths.iter());
        output.push_str(&structure);
        output.push_str("\n</directory_structure>\n\n");
    }

    // Files
    if config.output.files {
        output.push_str("<files>\n");
        output.push_str("This section contains the contents of the repository's files.\n\n");
        for path in ctx.sorted_paths {
            let content = ctx.files.get(path).unwrap();
            output.push_str(&format!("<file path=\"{}\">\n", path.display()));
            if config.output.show_line_numbers {
                output.push_str(&add_line_numbers(content));
            } else {
                output.push_str(content);
                if !content.ends_with('\n') {
                    output.push('\n');
                }
            }
            output.push_str("</file>\n");
        }
        output.push_str("\n</files>\n\n");
    }

    // Git Diffs
    if config.output.git.include_diffs {
        output.push_str("<git_diffs>\n");
        if let Ok(diff) = crate::core::git::get_git_diff(std::path::Path::new("."), false) {
            output.push_str("<git_diff_work_tree>\n");
            if !diff.is_empty() {
                output.push_str(&diff);
                if !diff.ends_with('\n') {
                    output.push('\n');
                }
            }
            output.push_str("</git_diff_work_tree>\n");
        }

        if let Ok(diff) = crate::core::git::get_git_diff(std::path::Path::new("."), true) {
            output.push_str("<git_diff_staged>\n");
            if !diff.is_empty() {
                output.push_str(&diff);
                if !diff.ends_with('\n') {
                    output.push('\n');
                }
            }
            output.push_str("</git_diff_staged>\n");
        }
        output.push_str("</git_diffs>\n\n");
    }

    // Git Logs
    if config.output.git.include_logs {
        let max_commits = config.output.git.include_logs_count;
        output.push_str("<git_logs>\n");
        if let Ok(commits) =
            crate::core::git::get_git_log(std::path::Path::new("."), max_commits as usize)
        {
            for commit in commits {
                output.push_str("<git_log_commit>\n");
                output.push_str(&format!("<date>{}</date>\n", commit.date));
                output.push_str(&format!("<message>{}</message>\n", commit.message));
                output.push_str("<files>\n");
                for file in commit.files {
                    output.push_str(&file);
                    output.push('\n');
                }
                output.push_str("</files>\n");
                output.push_str("</git_log_commit>\n");
            }
        }
        output.push_str("</git_logs>\n\n");
    }

    // Instruction
    if let Some(instruction_path) = &config.output.instruction_file_path {
        output.push_str("<instruction>\n");
        if let Ok(instruction) = std::fs::read_to_string(instruction_path) {
            output.push_str(&instruction);
            if !instruction.ends_with('\n') {
                output.push('\n');
            }
        }
        output.push_str("</instruction>\n");
    }

    Ok(())
}

fn generate_header(config: &RepomixConfig) -> String {
    let is_entire_codebase = config.include.is_empty() && config.ignore.custom_patterns.is_empty();
    let mut description = String::new();

    if is_entire_codebase {
        description.push_str("This file is a merged representation of the entire codebase");
    } else {
        let mut parts = Vec::new();
        if !config.include.is_empty() {
            parts.push("specifically included files");
        }
        if !config.ignore.custom_patterns.is_empty() {
            parts.push("files not matching ignore patterns");
        }
        description.push_str(&format!(
            "This file is a merged representation of a subset of the codebase, containing {}",
            parts.join(" and ")
        ));
    }

    let mut processing_notes = Vec::new();
    if config.output.remove_comments {
        processing_notes.push("comments have been removed".to_string());
    }
    if config.output.remove_empty_lines {
        processing_notes.push("empty lines have been removed".to_string());
    }
    if config.output.show_line_numbers {
        processing_notes.push("line numbers have been added".to_string());
    }
    if config.output.parsable_style {
        processing_notes.push(format!(
            "content has been formatted for parsing in {} style",
            &config.output.style.to_string()
        ));
    }
    if config.output.compress {
        processing_notes.push(
            "content has been compressed (code blocks are separated by ⋮---- delimiter)"
                .to_string(),
        );
    }
    if !config.security.enable_security_check {
        processing_notes.push("security check has been disabled".to_string());
    }

    let processing_info = if !processing_notes.is_empty() {
        format!(
            "The content has been processed where {}.",
            processing_notes.join(", ")
        )
    } else {
        String::new()
    };

    let mut result = format!(
        "{}, combined into a single document by Repomix.",
        description
    );
    if !processing_info.is_empty() {
        result.push('\n');
        result.push_str(&processing_info);
    }
    result
}

fn generate_summary_purpose(config: &RepomixConfig) -> String {
    let is_entire_codebase = config.include.is_empty() && config.ignore.custom_patterns.is_empty();
    let content_description = if is_entire_codebase {
        "the entire repository's contents"
    } else {
        "a subset of the repository's contents that is considered the most important context"
    };

    format!(
        "This file contains a packed representation of {}.\nIt is designed to be easily consumable by AI systems for analysis, code review,\nor other automated processes.",
        content_description
    )
}

fn generate_summary_notes(config: &RepomixConfig) -> String {
    let mut notes = vec![
        "- Some files may have been excluded based on .gitignore rules and Repomix's configuration".to_string(),
        "- Binary files are not included in this packed representation. Please refer to the Repository Structure section for a complete list of file paths, including binary files".to_string(),
    ];

    if !config.include.is_empty() {
        notes.push(format!(
            "- Only files matching these patterns are included: {}",
            config.include.join(", ")
        ));
    }
    if !config.ignore.custom_patterns.is_empty() {
        notes.push(format!(
            "- Files matching these patterns are excluded: {}",
            config.ignore.custom_patterns.join(", ")
        ));
    }
    if config.ignore.use_gitignore {
        notes.push("- Files matching patterns in .gitignore are excluded".to_string());
    }
    if config.ignore.use_default_patterns {
        notes.push("- Files matching default ignore patterns are excluded".to_string());
    }

    if config.output.remove_comments {
        notes.push("- Code comments have been removed from supported file types".to_string());
    }
    if config.output.remove_empty_lines {
        notes.push("- Empty lines have been removed from all files".to_string());
    }
    if config.output.show_line_numbers {
        notes.push("- Line numbers have been added to the beginning of each line".to_string());
    }
    if config.output.parsable_style {
        notes.push(format!(
            "- Content has been formatted for parsing in {} style",
            &config.output.style.to_string()
        ));
    }
    if config.output.compress {
        notes.push(
            "- Content has been compressed - code blocks are separated by ⋮---- delimiter"
                .to_string(),
        );
    }
    if config.output.truncate_base64 {
        notes.push("- Long base64 data strings (e.g., data:image/png;base64,...) have been truncated to reduce token count".to_string());
    }
    if !config.security.enable_security_check {
        notes.push(
            "- Security check has been disabled - content may contain sensitive information"
                .to_string(),
        );
    }

    if config.output.git.sort_by_changes {
        notes.push(
            "- Files are sorted by Git change count (files with more changes are at the bottom)"
                .to_string(),
        );
    }

    if config.output.git.include_diffs {
        notes.push("- Git diffs from the worktree and staged changes are included".to_string());
    }

    if config.output.git.include_logs {
        let max_commits = config.output.git.include_logs_count;
        notes.push(format!(
            "- Git logs ({} commits) are included to show development patterns",
            max_commits
        ));
    }

    notes.join("\n")
}

fn generate_summary_file_format_text() -> String {
    vec![
        "The content is organized as follows:",
        "1. This summary section",
        "2. Repository information",
        "3. Directory structure",
        "4. Repository files (if enabled)",
        "5. Multiple file entries, each consisting of:",
        "  - File path as an attribute",
        "  - Full contents of the file",
    ]
    .join("\n")
}

fn generate_summary_usage_guidelines(config: &RepomixConfig) -> String {
    let mut guidelines = vec![
        "- This file should be treated as read-only. Any changes should be made to the".to_string(),
        "  original repository files, not this packed version.".to_string(),
        "- When processing this file, use the file path to distinguish".to_string(),
        "  between different files in the repository.".to_string(),
        "- Be aware that this file may contain sensitive information. Handle it with".to_string(),
        "  the same level of security as you would the original repository.".to_string(),
    ];

    if config.output.header_text.is_some() {
        guidelines.push(
            "- Pay special attention to the Repository Description. These contain important context and guidelines specific to this project."
                .to_string(),
        );
    }
    if config.output.instruction_file_path.is_some() {
        guidelines.push(
            "- Pay special attention to the Repository Instruction. These contain important context and guidelines specific to this project."
                .to_string(),
        );
    }

    guidelines.join("\n")
}

fn format_json(output: &mut String, ctx: &FormatContext, config: &RepomixConfig) -> Result<()> {
    let mut root = Map::new();

    if config.output.file_summary {
        root.insert(
            "fileSummary".to_string(),
            json!({
                "generationHeader": generate_header(config),
                "purpose": generate_summary_purpose(config),
                "fileFormat": generate_summary_file_format_text(),
                "usageGuidelines": generate_summary_usage_guidelines(config),
                "notes": generate_summary_notes(config),
            }),
        );
    }

    if let Some(header) = &config.output.header_text {
        root.insert("userProvidedHeader".to_string(), json!(header));
    }

    if config.output.directory_structure {
        root.insert(
            "directoryStructure".to_string(),
            json!(generate_directory_structure(ctx.tree_paths.iter())),
        );
    }

    if config.output.files {
        let mut files = Map::new();
        for path in ctx.sorted_paths {
            if let Some(content) = ctx.files.get(path) {
                let rendered = if config.output.show_line_numbers {
                    add_line_numbers(content)
                } else {
                    content.to_string()
                };
                files.insert(path.to_string_lossy().to_string(), json!(rendered));
            }
        }
        root.insert("files".to_string(), Value::Object(files));
    }

    if config.output.git.include_diffs {
        let mut diffs = Map::new();
        if let Ok(diff) = crate::core::git::get_git_diff(std::path::Path::new("."), false) {
            if !diff.is_empty() {
                diffs.insert("workTree".to_string(), json!(diff));
            }
        }
        if let Ok(diff) = crate::core::git::get_git_diff(std::path::Path::new("."), true) {
            if !diff.is_empty() {
                diffs.insert("staged".to_string(), json!(diff));
            }
        }
        if !diffs.is_empty() {
            root.insert("gitDiffs".to_string(), Value::Object(diffs));
        }
    }

    if config.output.git.include_logs {
        if let Ok(commits) = crate::core::git::get_git_log(
            std::path::Path::new("."),
            config.output.git.include_logs_count as usize,
        ) {
            let serialized: Vec<Value> = commits
                .into_iter()
                .map(|commit| {
                    json!({
                        "date": commit.date,
                        "message": commit.message,
                        "files": commit.files,
                    })
                })
                .collect();
            root.insert("gitLogs".to_string(), json!(serialized));
        }
    }

    if let Some(instruction_path) = &config.output.instruction_file_path {
        if let Ok(instruction) = std::fs::read_to_string(instruction_path) {
            root.insert("instruction".to_string(), json!(instruction));
        }
    }

    let mut metadata = Map::new();
    metadata.insert("totalFiles".to_string(), json!(ctx.files.len()));
    metadata.insert("totalTokens".to_string(), json!(ctx.token_count));
    metadata.insert("totalChars".to_string(), json!(ctx.total_chars));
    metadata.insert(
        "topFiles".to_string(),
        json!(ctx
            .top_files
            .iter()
            .map(|file| {
                json!({
                    "path": file.path.to_string_lossy(),
                    "tokenCount": file.token_count,
                    "charCount": file.char_count,
                })
            })
            .collect::<Vec<_>>()),
    );
    if let Some(tree) = ctx.token_count_tree {
        metadata.insert("tokenCountTree".to_string(), to_json_value(tree));
    }
    root.insert("metadata".to_string(), Value::Object(metadata));

    *output = serde_json::to_string_pretty(&root)?;
    Ok(())
}

fn format_markdown(output: &mut String, ctx: &FormatContext, config: &RepomixConfig) {
    output.push_str("# Files\n\n");
    for path in ctx.sorted_paths {
        let content = ctx.files.get(path).unwrap();
        output.push_str(&format!("## File: {}\n\n", path.display()));

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        output.push_str(&format!("```{}\n", ext));
        if config.output.show_line_numbers {
            output.push_str(&add_line_numbers(content));
        } else {
            output.push_str(content);
            if !content.ends_with('\n') {
                output.push('\n');
            }
        }
        output.push_str("```\n\n");
    }
}

fn format_plain(output: &mut String, ctx: &FormatContext, config: &RepomixConfig) {
    for path in ctx.sorted_paths {
        let content = ctx.files.get(path).unwrap();
        output.push_str(&format!("File: {}\n", path.display()));
        output.push_str("================================================================\n");
        if config.output.show_line_numbers {
            output.push_str(&add_line_numbers(content));
        } else {
            output.push_str(content);
            if !content.ends_with('\n') {
                output.push('\n');
            }
        }
        output.push_str("\n\n");
    }
}
fn generate_directory_structure<'a, I>(paths: I) -> String
where
    I: Iterator<Item = &'a PathBuf>,
{
    struct Node {
        is_dir: bool,
        children: std::collections::BTreeMap<String, Node>,
    }

    impl Node {
        fn new(is_dir: bool) -> Self {
            Self {
                is_dir,
                children: std::collections::BTreeMap::new(),
            }
        }

        fn insert(&mut self, path_parts: &[&str]) {
            if path_parts.is_empty() {
                return;
            }
            let name = path_parts[0];
            let is_last = path_parts.len() == 1;

            let entry = self
                .children
                .entry(name.to_string())
                .or_insert_with(|| Node::new(!is_last));
            // If we are inserting a file, ensure the node is marked as not a dir (though it might have been created as a dir if we saw a longer path before?)
            // Actually, in repomix, a path is either a file or a dir.
            // If we see "a/b", "a" is a dir.
            // If we later see "a", that would be a conflict if "a" is a file.
            // But paths come from a file walker, so "a" and "a/b" won't coexist as file and dir.
            // However, if we see "a/b" and then "a/c", "a" is dir.

            if !is_last {
                entry.is_dir = true;
                entry.insert(&path_parts[1..]);
            } else {
                // It's a leaf in this path.
                // If it was already a dir, it stays a dir? (e.g. explicitly included dir?)
                // But here input `paths` are files.
                // So if it's a leaf, it's a file.
                // Unless we support empty directories?
                // For now assume file.
            }
        }

        fn to_string(&self, prefix: &str) -> String {
            let mut result = String::new();

            let mut children: Vec<(&String, &Node)> = self.children.iter().collect();

            // Sort: directories first, then files. Both alphabetical.
            children.sort_by(|(name_a, node_a), (name_b, node_b)| {
                if node_a.is_dir == node_b.is_dir {
                    name_a.cmp(name_b)
                } else {
                    if node_a.is_dir {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }
            });

            for (name, child) in children {
                result.push_str(prefix);
                result.push_str(name);
                if child.is_dir {
                    result.push('/');
                }
                result.push('\n');
                if child.is_dir {
                    result.push_str(&child.to_string(&format!("{}  ", prefix)));
                }
            }
            result
        }
    }

    let mut root = Node::new(true);

    for path in paths {
        let path_str = path.to_string_lossy();
        // Convert to parts
        let parts: Vec<&str> = path_str.split('/').collect();
        root.insert(&parts);
    }

    root.to_string("").trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{RepomixConfig, RepomixOutputStyle};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn json_output_respects_line_numbers_flag() {
        let mut config = RepomixConfig::default();
        config.output.style = RepomixOutputStyle::Json;
        config.output.show_line_numbers = true;
        config.output.file_summary = false;
        config.output.directory_structure = false;
        config.output.git.include_diffs = false;
        config.output.git.include_logs = false;

        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("src/main.rs"),
            "fn main() {\nprintln!(\"hi\");\n}".to_string(),
        );
        let sorted_paths = vec![PathBuf::from("src/main.rs")];
        let ctx = FormatContext {
            files: &files,
            sorted_paths: &sorted_paths,
            tree_paths: &sorted_paths,
            top_files: &[],
            token_count_tree: None,
            token_count: 0,
            total_chars: files.values().map(|c| c.chars().count()).sum(),
        };

        let output = format(&config, ctx).unwrap().content;
        let value: Value = serde_json::from_str(&output).unwrap();
        let file_text = value["files"]["src/main.rs"]
            .as_str()
            .expect("file content should be string");

        assert!(file_text.starts_with("   1: fn main() {"));
        assert!(file_text.contains("\n   2: println!(\"hi\");\n"));
        assert!(file_text.trim_end().ends_with("   3: }"));
    }
}
