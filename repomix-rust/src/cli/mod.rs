use clap::{Parser, ValueEnum};
use std::path::PathBuf;

pub mod init;
pub mod tui;

#[derive(Parser, Debug)]
#[command(name = "repomix")]
#[command(about = "Pack your repository into a single AI-friendly file", long_about = None)]
#[command(version)]
pub struct Cli {
    /// List of directories to process
    #[arg(default_value = ".")]
    pub directories: Vec<PathBuf>,

    /// Custom configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    // --- Input Options ---
    /// Maximum file size in bytes to include (e.g., 50MB)
    #[arg(long, value_name = "BYTES")]
    pub max_file_size: Option<u64>,

    // --- Output Options ---
    /// Specify the output file path (defaults to repomix-output.xml/md/json/txt)
    #[arg(short, long = "output", value_name = "FILE")]
    pub output_file: Option<String>,

    /// Output to stdout instead of writing to a file
    #[arg(long, conflicts_with = "output_file")]
    pub stdout: bool, // This remains bool, as it's a direct toggle and handled separately

    /// Specify the output style (xml, markdown, json, plain)
    #[arg(long, value_enum)]
    pub style: Option<OutputStyleCli>,

    /// Generate parsable output (escape special characters)
    #[arg(long)]
    pub parsable_style: bool,
    /// Do not generate parsable output
    #[arg(long = "no-parsable-style")]
    pub no_parsable_style: bool,

    /// Custom header text to prepend to the output
    #[arg(long)]
    pub header_text: Option<String>,

    /// Path to an instruction file to append to the output
    #[arg(long)]
    pub instruction_file_path: Option<String>,

    /// Include file summary section
    #[arg(long)]
    pub file_summary: bool,
    /// Exclude file summary section
    #[arg(long = "no-file-summary")]
    pub no_file_summary: bool,

    /// Include directory structure section
    #[arg(long)]
    pub directory_structure: bool,
    /// Exclude directory structure section
    #[arg(long = "no-directory-structure")]
    pub no_directory_structure: bool,

    /// Include file contents section
    #[arg(long)]
    pub files: bool,
    /// Exclude file contents section
    #[arg(long = "no-files")]
    pub no_files: bool,

    /// Remove comments from supported file types
    #[arg(long)]
    pub remove_comments: bool,
    /// Do not remove comments
    #[arg(long = "no-remove-comments")]
    pub no_remove_comments: bool,

    /// Remove empty lines from output
    #[arg(long)]
    pub remove_empty_lines: bool,
    /// Do not remove empty lines
    #[arg(long = "no-remove-empty-lines")]
    pub no_remove_empty_lines: bool,

    /// Perform code compression to reduce token count
    #[arg(long)]
    pub compress: bool,
    /// Do not perform code compression
    #[arg(long = "no-compress")]
    pub no_compress: bool,

    /// Number of top files to show in summary
    #[arg(long, value_name = "COUNT")]
    pub top_files_length: Option<u32>,

    /// Show line numbers in the output
    #[arg(long)]
    pub show_line_numbers: bool,
    /// Do not show line numbers
    #[arg(long = "no-show-line-numbers")]
    pub no_show_line_numbers: bool,

    /// Truncate long base64 strings
    #[arg(long)]
    pub truncate_base64: bool,
    /// Do not truncate base64 strings
    #[arg(long = "no-truncate-base64")]
    pub no_truncate_base64: bool,

    /// Copy generated output to system clipboard
    #[arg(long)]
    pub copy: bool,
    /// Do not copy to clipboard
    #[arg(long = "no-copy")]
    pub no_copy: bool,

    /// Include empty directories in structure
    #[arg(long)]
    pub include_empty_directories: bool,
    /// Do not include empty directories
    #[arg(long = "no-include-empty-directories")]
    pub no_include_empty_directories: bool,

    /// Include full directory structure (vs. truncated)
    #[arg(long)]
    pub include_full_directory_structure: bool,
    /// Do not include full directory structure
    #[arg(long = "no-include-full-directory-structure")]
    pub no_include_full_directory_structure: bool,

    /// Enable token count tree display
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    pub token_count_tree: Option<String>,
    /// Disable token count tree display
    #[arg(long = "no-token-count-tree")]
    pub no_token_count_tree: bool,

    // --- Git Output Options ---
    /// Sort files by git change frequency
    #[arg(long)]
    pub git_sort_by_changes: bool,
    /// Do not sort files by git change frequency
    #[arg(long = "no-git-sort-by-changes")]
    pub no_git_sort_by_changes: bool,

    /// Max number of commits to consider for git sort
    #[arg(long, value_name = "COUNT")]
    pub git_sort_by_changes_max_commits: Option<u32>,

    /// Include git diffs in output
    #[arg(long)]
    pub git_include_diffs: bool,
    /// Do not include git diffs
    #[arg(long = "no-git-include-diffs")]
    pub no_git_include_diffs: bool,

    /// Include git logs in output
    #[arg(long)]
    pub git_include_logs: bool,
    /// Do not include git logs
    #[arg(long = "no-git-include-logs")]
    pub no_git_include_logs: bool,

    /// Number of git log commits to include
    #[arg(long, value_name = "COUNT")]
    pub git_include_logs_count: Option<u32>,

    // --- Include/Ignore Options ---
    /// Additional include glob patterns
    #[arg(long = "include", value_name = "PATTERN")]
    pub include_patterns: Vec<String>,

    /// Additional ignore glob patterns (applied before ignore files)
    #[arg(short = 'i', long = "ignore", value_name = "PATTERN")]
    pub ignore_patterns: Vec<String>,

    /// Use .gitignore files
    #[arg(long)]
    pub use_gitignore: bool,
    /// Disable reading of .gitignore files
    #[arg(long = "no-use-gitignore")]
    pub no_use_gitignore: bool,

    /// Use .ignore files
    #[arg(long)]
    pub use_dot_ignore: bool,
    /// Disable reading of .ignore files
    #[arg(long = "no-use-dot-ignore")]
    pub no_use_dot_ignore: bool,

    /// Use built-in default ignore patterns
    #[arg(long)]
    pub use_default_patterns: bool,
    /// Disable built-in default ignore patterns
    #[arg(long = "no-use-default-patterns")]
    pub no_use_default_patterns: bool,

    // --- Security Options ---
    /// Enable scanning for sensitive data
    #[arg(long)]
    pub enable_security_check: bool,
    /// Disable scanning for sensitive data
    #[arg(long = "no-security-check")]
    pub no_enable_security_check: bool,

    // --- Token Count Options ---
    /// Tokenizer encoding (e.g., o200k_base, cl100k_base)
    #[arg(long)]
    pub encoding: Option<String>,

    // --- Other Options ---
    /// Add paths from stdin (one absolute/relative path per line)
    #[arg(long)]
    pub stdin: bool,

    /// Enable verbose logging
    #[arg(long)]
    pub verbose: bool,

    /// Disable all output to stdout
    #[arg(long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Run as MCP server
    #[arg(long = "mcp", alias = "server")]
    pub server: bool,

    /// Initialize a new configuration file
    #[arg(long)]
    pub init: bool,

    /// Use global configuration directory for init
    #[arg(long, requires = "init")]
    pub global: bool,

    /// Remote repository URL
    #[arg(long)]
    pub remote: Option<String>,

    /// Remote repository branch/tag/commit
    #[arg(long)]
    pub remote_branch: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputStyleCli {
    Xml,
    Markdown,
    Json, // Added Json
    Plain,
}
