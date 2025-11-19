use crate::config::RepomixConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct RepomixOutput {
    pub content: String,
}

pub fn format(config: &RepomixConfig, files: &HashMap<PathBuf, String>) -> Result<RepomixOutput> {
    let mut output = String::new();

    // Add header
    if let Some(header) = &config.output.header_text {
        output.push_str(header);
        output.push('\n');
    }

    // Add directory structure if enabled
    if config.output.directory_structure {
        output.push_str("Directory Structure:\n");
        let structure = generate_directory_structure(files.keys());
        output.push_str(&structure);
        output.push('\n');
    }

    // Add git info if enabled
    // Note: We need to pass the root directory to get_git_log/diff. 
    // For now, assuming current directory or we need to pass it in config.
    // Let's assume the user runs repomix from the root of the repo for now.
    let git_config = &config.output.git;
    if git_config.include_diffs {
            // TODO: We need a way to know the root dir. For now using "."
            if let Ok(diff) = crate::core::git::get_git_diff(std::path::Path::new(".")) {
                if !diff.is_empty() {
                    output.push_str("\nGit Diff:\n");
                    output.push_str(&diff);
                    output.push('\n');
                }
            }
    }

    // Add files
    if config.output.files {
        let style = config.output.style.as_deref().unwrap_or("xml");
        match style {
            "markdown" => format_markdown(&mut output, files, config),
            "plain" => format_plain(&mut output, files, config),
            _ => format_xml(&mut output, files, config), // Default to XML
        }
    }

    Ok(RepomixOutput { content: output })
}

fn generate_directory_structure<'a, I>(paths: I) -> String
where
    I: Iterator<Item = &'a PathBuf>,
{
    use std::collections::BTreeMap;
    
    // Build tree structure
    let mut tree: BTreeMap<String, BTreeMap<String, ()>> = BTreeMap::new();
    
    for path in paths {
        let path_str = path.to_string_lossy();
        let parts: Vec<&str> = path_str.split('/').collect();
        
        if parts.is_empty() {
            continue;
        }
        
        // Add to tree
        if parts.len() == 1 {
            tree.entry(parts[0].to_string()).or_default();
        } else if parts.len() > 1 {
            let dir = parts[0].to_string();
            let rest = parts[1..].join("/");
            tree.entry(dir).or_default().insert(rest, ());
        }
    }
    
    // Generate tree string
    let mut result = String::new();
    let dirs: Vec<_> = tree.keys().collect();
    
    for (i, dir) in dirs.iter().enumerate() {
        let is_last = i == dirs.len() - 1;
        let prefix = if is_last { "└── " } else { "├── " };
        
        result.push_str(&format!("{}{}\n", prefix, dir));
        
        // Add subdirectories/files
        if let Some(children) = tree.get(*dir) {
            let child_prefix = if is_last { "    " } else { "│   " };
            let child_list: Vec<_> = children.keys().collect();
            
            for (j, child) in child_list.iter().enumerate() {
                let is_last_child = j == child_list.len() - 1;
                let child_marker = if is_last_child { "└── " } else { "├── " };
                result.push_str(&format!("{}{}{}\n", child_prefix, child_marker, child));
            }
        }
    }
    
    result
}

fn format_xml(output: &mut String, files: &HashMap<PathBuf, String>, config: &RepomixConfig) {
    output.push_str("<files>\n");
    for (path, content) in files {
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
    output.push_str("</files>\n");
}

fn format_markdown(output: &mut String, files: &HashMap<PathBuf, String>, config: &RepomixConfig) {
    output.push_str("# Files\n\n");
    for (path, content) in files {
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
    for (path, content) in files {
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
