use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "repomix")]
#[command(about = "Pack your repository into a single AI-friendly file", long_about = None)]
#[command(version)]
pub struct Cli {
    /// List of directories to process
    #[arg(default_value = ".")]
    pub directories: Vec<PathBuf>,

    /// Additional include glob patterns
    #[arg(long = "include", value_name = "PATTERN")]
    pub include: Vec<String>,

    /// Additional ignore glob patterns (applied before ignore files)
    #[arg(short = 'i', long = "ignore", value_name = "PATTERN")]
    pub ignore: Vec<String>,

    /// Specify the output file name
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Custom configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Output to stdout instead of writing to a file
    #[arg(long, conflicts_with = "output")]
    pub stdout: bool,

    /// Specify the output style
    #[arg(long, value_enum, default_value_t = OutputStyle::Xml)]
    pub style: OutputStyle,

    /// Perform code compression to reduce token count
    #[arg(long)]
    pub compress: bool,

    /// Show line numbers in the output
    #[arg(long)]
    pub output_show_line_numbers: bool,

    /// Copy generated output to system clipboard
    #[arg(long)]
    pub copy: bool,

    /// Disable reading of gitignore files
    #[arg(long = "no-gitignore")]
    pub no_gitignore: bool,

    /// Disable reading of .ignore files
    #[arg(long = "no-dot-ignore")]
    pub no_dot_ignore: bool,

    /// Disable built-in default ignore patterns
    #[arg(long = "no-default-patterns")]
    pub no_default_patterns: bool,

    /// Add paths from stdin (one absolute/relative path per line)
    #[arg(long)]
    pub stdin: bool,

    /// Enable verbose logging
    #[arg(long, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Disable all output to stdout
    #[arg(long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Run as MCP server
    #[arg(long)]
    pub server: bool,

    /// Remote repository URL
    #[arg(long)]
    pub remote: Option<String>,

    /// Remote repository branch/tag/commit
    #[arg(long)]
    pub remote_branch: Option<String>,

    /// Skip scanning for sensitive data
    #[arg(long = "no-security-check")]
    pub no_security_check: bool,

    /// Tokenizer encoding (e.g., o200k_base, cl100k_base)
    #[arg(long)]
    pub token_count_encoding: Option<String>,

    /// Remove comments from supported file types
    #[arg(long)]
    pub remove_comments: bool,

    /// Remove empty lines from output
    #[arg(long)]
    pub remove_empty_lines: bool,

    /// Generate parsable output (escape special characters)
    #[arg(long)]
    pub parsable_style: bool,

    /// Disable file summary section
    #[arg(long = "no-file-summary")]
    pub no_file_summary: bool,

    /// Disable directory structure section
    #[arg(long = "no-directory-structure")]
    pub no_directory_structure: bool,

    /// Disable file contents section (metadata only)
    #[arg(long = "no-files")]
    pub no_files: bool,

    /// Custom header text
    #[arg(long)]
    pub header_text: Option<String>,

    /// Path to instruction file
    #[arg(long)]
    pub instruction_file_path: Option<PathBuf>,

    /// Include empty directories in structure
    #[arg(long)]
    pub include_empty_directories: bool,

    /// Truncate long base64 strings
    #[arg(long)]
    pub truncate_base64: bool,

    /// Number of top files to show in summary
    #[arg(long)]
    pub top_files_len: Option<usize>,

    /// Disable sorting files by git change frequency
    #[arg(long = "no-git-sort-by-changes")]
    pub no_git_sort_by_changes: bool,

    /// Include git diffs in output
    #[arg(long)]
    pub include_diffs: bool,

    /// Include git logs in output
    #[arg(long)]
    pub include_logs: bool,

    /// Number of git log commits to include
    #[arg(long)]
    pub include_logs_count: Option<usize>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputStyle {
    Xml,
    Markdown,
    Plain,
}
