use crate::config::RepomixConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct RepomixOutput {
    pub content: String,
}

pub fn format(config: &RepomixConfig, files: &HashMap<PathBuf, String>) -> Result<RepomixOutput> {
    let mut output = String::new();
    let style = config.output.style.as_deref().unwrap_or("xml");

    match style {
        "markdown" => format_markdown(&mut output, files, config),
        "plain" => format_plain(&mut output, files, config),
        _ => format_xml_full(&mut output, files, config)?,
    }

    Ok(RepomixOutput { content: output })
}

fn format_xml_full(
    output: &mut String,
    files: &HashMap<PathBuf, String>,
    config: &RepomixConfig,
) -> Result<()> {
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
    output.push_str("The content is organized as follows:\n");
    output.push_str("1. This summary section\n");
    output.push_str("2. Repository information\n");
    output.push_str("3. Directory structure\n");
    output.push_str("4. Repository files (if enabled)\n");
    output.push_str("5. Multiple file entries, each consisting of:\n");
    output.push_str("  - File path as an attribute\n");
    output.push_str("  - Full contents of the file\n");
    output.push_str("</file_format>\n\n");

    output.push_str("<usage_guidelines>\n");
    output.push_str("- This file should be treated as read-only. Any changes should be made to the\n");
    output.push_str("  original repository files, not this packed version.\n");
    output.push_str("- When processing this file, use the file path to distinguish\n");
    output.push_str("  between different files in the repository.\n");
    output.push_str("- Be aware that this file may contain sensitive information. Handle it with\n");
    output.push_str("  the same level of security as you would the original repository.\n");
    if config.output.header_text.is_some() {
        output.push_str("- Pay special attention to the Repository Description. These contain important context and guidelines specific to this project.\n");
    }
    if config.output.instruction_file_path.is_some() {
        output.push_str("- Pay special attention to the Repository Instruction. These contain important context and guidelines specific to this project.\n");
    }
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
        let structure = generate_directory_structure(files.keys());
        output.push_str(&structure);
        output.push_str("\n</directory_structure>\n\n");
    }

    // Files
    if config.output.files {
        output.push_str("<files>\n");
        output.push_str("This section contains the contents of the repository's files.\n\n");
        let mut paths: Vec<_> = files.keys().collect();
        paths.sort();
        for path in paths {
            let content = files.get(path).unwrap();
            output.push_str(&format!("<file path=\"{}\">\n", path.display()));
            if config.output.show_line_numbers {
                for (i, line) in content.lines().enumerate() {
                    output.push_str(&format!("{:4}: {}\n", i + 1, line));
                }
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
        if let Ok(diff) = crate::core::git::get_git_diff(std::path::Path::new(".")) {
            output.push_str("<git_diff_work_tree>\n");
            if !diff.is_empty() {
                output.push_str(&diff);
                if !diff.ends_with('\n') {
                    output.push('\n');
                }
            }
            output.push_str("\n</git_diff_work_tree>\n");
        }
        
        // TODO: Separate worktree and staged diffs properly. 
        // Current get_git_diff likely combines them or gets one.
        // For now, just putting placeholder for staged if we can't separate easily yet.
        output.push_str("<git_diff_staged>\n\n</git_diff_staged>\n");
        output.push_str("</git_diffs>\n\n");
    }

    // Git Logs
    if config.output.git.include_logs {
        let max_commits = config.output.git.include_logs_count.unwrap_or(50);
        output.push_str("<git_logs>\n");
        if let Ok(commits) = crate::core::git::get_git_log(std::path::Path::new("."), max_commits) {
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
            config.output.style.as_deref().unwrap_or("xml")
        ));
    }
    if config.output.compress {
        processing_notes.push("content has been compressed (code blocks are separated by ⋮---- delimiter)".to_string());
    }
    if !config.security.enable_security_check {
        processing_notes.push("security check has been disabled".to_string());
    }

    let processing_info = if !processing_notes.is_empty() {
        format!("The content has been processed where {}.", processing_notes.join(", "))
    } else {
        String::new()
    };

    let mut result = format!("{}, combined into a single document by Repomix.", description);
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
        notes.push(format!("- Only files matching these patterns are included: {}", config.include.join(", ")));
    }
    if !config.ignore.custom_patterns.is_empty() {
        notes.push(format!("- Files matching these patterns are excluded: {}", config.ignore.custom_patterns.join(", ")));
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
            config.output.style.as_deref().unwrap_or("xml")
        ));
    }
    if config.output.compress {
        notes.push("- Content has been compressed - code blocks are separated by ⋮---- delimiter".to_string());
    }
    if config.output.truncate_base64 {
        notes.push("- Long base64 data strings (e.g., data:image/png;base64,...) have been truncated to reduce token count".to_string());
    }
    if !config.security.enable_security_check {
        notes.push("- Security check has been disabled - content may contain sensitive information".to_string());
    }

    if config.output.git.sort_by_changes {
        notes.push("- Files are sorted by Git change count (files with more changes are at the bottom)".to_string());
    }

    if config.output.git.include_diffs {
        notes.push("- Git diffs from the worktree and staged changes are included".to_string());
    }

    if config.output.git.include_logs {
        let max_commits = config.output.git.include_logs_count.unwrap_or(50);
        notes.push(format!("- Git logs ({} commits) are included to show development patterns", max_commits));
    }

    notes.join("\n")
}

fn format_markdown(output: &mut String, files: &HashMap<PathBuf, String>, config: &RepomixConfig) {
    output.push_str("# Files\n\n");
    let mut paths: Vec<_> = files.keys().collect();
    paths.sort();
    for path in paths {
        let content = files.get(path).unwrap();
        output.push_str(&format!("## File: {}\n\n", path.display()));

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        output.push_str(&format!("```{}\n", ext));
        if config.output.show_line_numbers {
            for (i, line) in content.lines().enumerate() {
                output.push_str(&format!("{:4}: {}\n", i + 1, line));
            }
        } else {
            output.push_str(content);
            if !content.ends_with('\n') {
                output.push('\n');
            }
        }
        output.push_str("```\n\n");
    }
}

fn format_plain(output: &mut String, files: &HashMap<PathBuf, String>, config: &RepomixConfig) {
    let mut paths: Vec<_> = files.keys().collect();
    paths.sort();
    for path in paths {
        let content = files.get(path).unwrap();
        output.push_str(&format!("File: {}\n", path.display()));
        output.push_str("================================================================\n");
        if config.output.show_line_numbers {
            for (i, line) in content.lines().enumerate() {
                output.push_str(&format!("{:4}: {}\n", i + 1, line));
            }
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
            
            let entry = self.children.entry(name.to_string()).or_insert_with(|| Node::new(!is_last));
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

