// repomix-rust/src/config/schema.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::cli::{Cli, OutputStyleCli};


// --- Enums ---
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepomixOutputStyle {
    Xml,
    Markdown,
    Json,
    Plain,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenCountTreeConfig {
    Bool(bool),
    Threshold(u64),
    Text(String),
}

impl Default for TokenCountTreeConfig {
    fn default() -> Self {
        TokenCountTreeConfig::Bool(false)
    }
}

fn true_default() -> bool {
    true
}

impl Default for RepomixOutputStyle {
    fn default() -> Self {
        RepomixOutputStyle::Xml
    }
}


impl std::fmt::Display for RepomixOutputStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RepomixOutputStyle::Xml => "xml",
                RepomixOutputStyle::Markdown => "markdown",
                RepomixOutputStyle::Json => "json",
                RepomixOutputStyle::Plain => "plain",
            }
        )
    }
}

// --- Default file path map (Rust equivalent of defaultFilePathMap) ---
pub fn default_file_path_map() -> HashMap<RepomixOutputStyle, String> {
    let mut map = HashMap::new();
    map.insert(RepomixOutputStyle::Xml, "repomix-output.xml".to_string());
    map.insert(RepomixOutputStyle::Markdown, "repomix-output.md".to_string());
    map.insert(RepomixOutputStyle::Json, "repomix-output.json".to_string());
    map.insert(RepomixOutputStyle::Plain, "repomix-output.txt".to_string());
    map
}

// --- Nested Config Structs ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputConfig {
    #[serde(rename = "maxFileSize", default = "InputConfig::default_max_file_size")]
    pub max_file_size: u64,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            max_file_size: Self::default_max_file_size(),
        }
    }
}

impl InputConfig {
    fn default_max_file_size() -> u64 {
        50 * 1024 * 1024 // 50MB
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitOutputConfig {
    #[serde(rename = "sortByChanges", default)]
    pub sort_by_changes: bool,
    #[serde(rename = "sortByChangesMaxCommits", default)]
    pub sort_by_changes_max_commits: u32,
    #[serde(rename = "includeDiffs", default)]
    pub include_diffs: bool,
    #[serde(rename = "includeLogs", default)]
    pub include_logs: bool,
    #[serde(rename = "includeLogsCount", default)]
    pub include_logs_count: u32,
}

impl Default for GitOutputConfig {
    fn default() -> Self {
        Self {
            sort_by_changes: false,
            sort_by_changes_max_commits: 100,
            include_diffs: false,
            include_logs: false,
            include_logs_count: 50,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(rename = "filePath", default = "OutputConfig::default_file_path")]
    pub file_path: Option<String>,
    #[serde(default = "OutputConfig::default_style")]
    pub style: RepomixOutputStyle,
    #[serde(rename = "parsableStyle", default)]
    pub parsable_style: bool,
    #[serde(rename = "headerText")]
    pub header_text: Option<String>,
    #[serde(rename = "instructionFilePath")]
    pub instruction_file_path: Option<String>,
    #[serde(rename = "fileSummary", default = "true_default")]
    pub file_summary: bool,
    #[serde(rename = "directoryStructure", default = "true_default")]
    pub directory_structure: bool,
    #[serde(default = "true_default")]
    pub files: bool,
    #[serde(rename = "removeComments", default)]
    pub remove_comments: bool,
    #[serde(rename = "removeEmptyLines", default)]
    pub remove_empty_lines: bool,
    #[serde(default)]
    pub compress: bool,
    #[serde(rename = "topFilesLength", default)]
    pub top_files_length: u32,
    #[serde(rename = "showLineNumbers", default)]
    pub show_line_numbers: bool,
    #[serde(rename = "truncateBase64", default)]
    pub truncate_base64: bool,
    #[serde(rename = "copyToClipboard", default)]
    pub copy_to_clipboard: bool,
    #[serde(rename = "includeEmptyDirectories", default)]
    pub include_empty_directories: bool,
    #[serde(rename = "includeFullDirectoryStructure", default)]
    pub include_full_directory_structure: bool,
    #[serde(rename = "tokenCountTree", default)]
    pub token_count_tree: TokenCountTreeConfig,
    #[serde(default)]
    pub git: GitOutputConfig,
    
    // CLI-specific output options, like `stdout`
    pub stdout: Option<bool>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            file_path: Self::default_file_path(),
            style: RepomixOutputStyle::default(),
            parsable_style: false,
            header_text: None,
            instruction_file_path: None,
            file_summary: true,
            directory_structure: true,
            files: true,
            remove_comments: false,
            remove_empty_lines: false,
            compress: false,
            top_files_length: 5,
            show_line_numbers: false,
            truncate_base64: false,
            copy_to_clipboard: false,
            include_empty_directories: false,
            include_full_directory_structure: false,
            token_count_tree: TokenCountTreeConfig::Bool(false),
            git: GitOutputConfig::default(),
            stdout: None,
        }
    }
}

impl OutputConfig {
    fn default_file_path() -> Option<String> {
        Some(default_file_path_map().get(&RepomixOutputStyle::default()).cloned().unwrap_or_default())
    }
    fn default_style() -> RepomixOutputStyle { RepomixOutputStyle::default() }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IgnoreConfig {
    #[serde(rename = "useGitignore", default = "true_default")]
    pub use_gitignore: bool,
    #[serde(rename = "useDotIgnore", default = "true_default")]
    pub use_dot_ignore: bool,
    #[serde(rename = "useDefaultPatterns", default = "true_default")]
    pub use_default_patterns: bool,
    #[serde(rename = "customPatterns", default)]
    pub custom_patterns: Vec<String>,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            use_gitignore: true,
            use_dot_ignore: true,
            use_default_patterns: true,
            custom_patterns: Vec::new(),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(rename = "enableSecurityCheck", default = "true_default")]
    pub enable_security_check: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_security_check: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenCountConfig {
    #[serde(default)]
    pub encoding: String,
}

impl Default for TokenCountConfig {
    fn default() -> Self {
        Self {
            encoding: "o200k_base".to_string(),
        }
    }
}

// --- Main Config Struct ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepomixConfig {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    #[serde(default)]
    pub input: InputConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub ignore: IgnoreConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(rename = "tokenCount", default)]
    pub token_count: TokenCountConfig,

    // Config options only present in RepomixConfigMerged (Node.js terminology)
    // These fields are not expected to come from config files, but are set programmatically.
    #[serde(skip)]
    pub cwd: PathBuf,
    #[serde(skip)]
    pub stdin_file_paths: Vec<PathBuf>,
    #[serde(skip)]
    pub remote_branch: Option<String>,
}

impl Default for RepomixConfig {
    fn default() -> Self {
        Self {
            schema: None,
            input: InputConfig::default(),
            output: OutputConfig::default(),
            include: Vec::new(),
            ignore: IgnoreConfig::default(),
            security: SecurityConfig::default(),
            token_count: TokenCountConfig::default(),
            cwd: PathBuf::new(),
            stdin_file_paths: Vec::new(),
            remote_branch: None,
        }
    }
}

impl RepomixConfig {
    pub fn merge_with_cli(mut self, cli: &Cli) -> Self {
        // Apply input overrides
        if let Some(max_file_size) = cli.max_file_size {
            self.input.max_file_size = max_file_size;
        }

        // Apply output overrides
        if let Some(file_path) = &cli.output_file {
            self.output.file_path = Some(file_path.clone());
        }
        if let Some(style_cli) = cli.style {
            self.output.style = match style_cli {
                OutputStyleCli::Xml => RepomixOutputStyle::Xml,
                OutputStyleCli::Markdown => RepomixOutputStyle::Markdown,
                OutputStyleCli::Json => RepomixOutputStyle::Json,
                OutputStyleCli::Plain => RepomixOutputStyle::Plain,
            };
        }
        // parsable_style
        if cli.no_parsable_style {
            self.output.parsable_style = false;
        } else if cli.parsable_style {
            self.output.parsable_style = true;
        }

        if let Some(header_text) = &cli.header_text {
            self.output.header_text = Some(header_text.clone());
        }
        if let Some(instruction_file_path) = &cli.instruction_file_path {
            self.output.instruction_file_path = Some(instruction_file_path.clone());
        }
        // File summary flags
        if cli.no_file_summary {
            self.output.file_summary = false;
        } else if cli.file_summary {
            self.output.file_summary = true;
        }
        // Directory structure flags
        if cli.no_directory_structure {
            self.output.directory_structure = false;
        } else if cli.directory_structure {
            self.output.directory_structure = true;
        }
        // Files flags
        if cli.no_files {
            self.output.files = false;
        } else if cli.files {
            self.output.files = true;
        }
        // Remove comments flags
        if cli.no_remove_comments {
            self.output.remove_comments = false;
        } else if cli.remove_comments {
            self.output.remove_comments = true;
        }
        // Remove empty lines flags
        if cli.no_remove_empty_lines {
            self.output.remove_empty_lines = false;
        } else if cli.remove_empty_lines {
            self.output.remove_empty_lines = true;
        }
        // Compress flags
        if cli.no_compress {
            self.output.compress = false;
        } else if cli.compress {
            self.output.compress = true;
        }
        if let Some(top_files_length) = cli.top_files_length {
            self.output.top_files_length = top_files_length;
        }
        // Show line numbers flags
        if cli.no_show_line_numbers {
            self.output.show_line_numbers = false;
        } else if cli.show_line_numbers {
            self.output.show_line_numbers = true;
        }
        // Truncate base64 flags
        if cli.no_truncate_base64 {
            self.output.truncate_base64 = false;
        } else if cli.truncate_base64 {
            self.output.truncate_base64 = true;
        }
        // Copy to clipboard flags
        if cli.no_copy {
            self.output.copy_to_clipboard = false;
        } else if cli.copy {
            self.output.copy_to_clipboard = true;
        }

        if cli.no_include_empty_directories {
            self.output.include_empty_directories = false;
        } else if cli.include_empty_directories {
            self.output.include_empty_directories = true;
        }
        if cli.no_include_full_directory_structure {
            self.output.include_full_directory_structure = false;
        } else if cli.include_full_directory_structure {
            self.output.include_full_directory_structure = true;
        }
        // Token count tree flags
        if cli.no_token_count_tree {
             self.output.token_count_tree = TokenCountTreeConfig::Bool(false);
        } else if cli.token_count_tree {
            self.output.token_count_tree = TokenCountTreeConfig::Bool(true);
        }
        // stdout is a simple bool, so it directly overrides
        // But wait, stdout in CLI is bool, default false.
        // If user didn't provide --stdout, cli.stdout is false.
        // Should we only override if true?
        // `stdout` has no `no-stdout` flag. It's just an action.
        // So if cli.stdout is true, enable it. If false, leave it (or disable? default is None/false in config).
        // OutputConfig.stdout is Option<bool>.
        if cli.stdout {
            self.output.stdout = Some(true);
        }
        

        // Git output overrides
        // Sort by changes flags
        if cli.no_git_sort_by_changes {
            self.output.git.sort_by_changes = false;
        } else if cli.git_sort_by_changes {
            self.output.git.sort_by_changes = true;
        }
        if let Some(sort_by_changes_max_commits) = cli.git_sort_by_changes_max_commits {
            self.output.git.sort_by_changes_max_commits = sort_by_changes_max_commits;
        }
        // Include diffs flags
        if cli.no_git_include_diffs {
            self.output.git.include_diffs = false;
        } else if cli.git_include_diffs {
            self.output.git.include_diffs = true;
        }
        // Include logs flags
        if cli.no_git_include_logs {
            self.output.git.include_logs = false;
        } else if cli.git_include_logs {
            self.output.git.include_logs = true;
        }
        if let Some(include_logs_count) = cli.git_include_logs_count {
            self.output.git.include_logs_count = include_logs_count;
        }

        // Include/Ignore overrides
        if !cli.include_patterns.is_empty() {
            self.include.extend(cli.include_patterns.clone());
        }
        if !cli.ignore_patterns.is_empty() {
            self.ignore.custom_patterns.extend(cli.ignore_patterns.clone());
        }
        // Use gitignore flags
        if cli.no_use_gitignore {
            self.ignore.use_gitignore = false;
        } else if cli.use_gitignore {
            self.ignore.use_gitignore = true;
        }
        // Use dot ignore flags
        if cli.no_use_dot_ignore {
            self.ignore.use_dot_ignore = false;
        } else if cli.use_dot_ignore {
            self.ignore.use_dot_ignore = true;
        }
        // Use default patterns flags
        if cli.no_use_default_patterns {
            self.ignore.use_default_patterns = false;
        } else if cli.use_default_patterns {
            self.ignore.use_default_patterns = true;
        }

        // Security overrides
        if cli.no_enable_security_check {
            self.security.enable_security_check = false;
        } else if cli.enable_security_check {
            self.security.enable_security_check = true;
        }

        // Token Count overrides
        if let Some(encoding) = &cli.encoding {
            self.token_count.encoding = encoding.clone();
        }

        self.remote_branch = cli.remote_branch.clone();

        self
    }
}
