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
    pub output_file: Option<String>, // Changed to String to match schema
    
    /// Output to stdout instead of writing to a file
    #[arg(long, conflicts_with = "output_file")]
    pub stdout: bool,

    /// Specify the output style (xml, markdown, json, plain)
    #[arg(long, value_enum)]
    pub style: Option<OutputStyleCli>, // Changed to Option<OutputStyleCli>

    /// Generate parsable output (escape special characters)
    #[arg(long)]
    pub parsable_style: Option<bool>, // Changed to Option<bool>

    /// Custom header text to prepend to the output
    #[arg(long)]
    pub header_text: Option<String>,

    /// Path to an instruction file to append to the output
    #[arg(long)]
    pub instruction_file_path: Option<String>, // Changed to String

    /// Include file summary section
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub file_summary: Option<bool>,
    /// Exclude file summary section
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_file_summary: Option<bool>,

    /// Include directory structure section
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub directory_structure: Option<bool>,
    /// Exclude directory structure section
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_directory_structure: Option<bool>,

    /// Include file contents section
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub files: Option<bool>,
    /// Exclude file contents section
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_files: Option<bool>,

    /// Remove comments from supported file types
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub remove_comments: Option<bool>,
    /// Do not remove comments
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_remove_comments: Option<bool>,

    /// Remove empty lines from output
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub remove_empty_lines: Option<bool>,
    /// Do not remove empty lines
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_remove_empty_lines: Option<bool>,

    /// Perform code compression to reduce token count
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub compress: Option<bool>,
    /// Do not perform code compression
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_compress: Option<bool>,

    /// Number of top files to show in summary
    #[arg(long, value_name = "COUNT")]
    pub top_files_length: Option<u32>, // Changed to u32

    /// Show line numbers in the output
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub show_line_numbers: Option<bool>,
    /// Do not show line numbers
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_show_line_numbers: Option<bool>,

    /// Truncate long base64 strings
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub truncate_base64: Option<bool>,
    /// Do not truncate base64 strings
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_truncate_base64: Option<bool>,

    /// Copy generated output to system clipboard
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub copy: Option<bool>, // Changed to Option<bool>
    /// Do not copy to clipboard
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_copy: Option<bool>,

    /// Include empty directories in structure
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub include_empty_directories: Option<bool>,
    /// Do not include empty directories
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_include_empty_directories: Option<bool>,

    /// Include full directory structure (vs. truncated)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub include_full_directory_structure: Option<bool>,
    /// Do not include full directory structure
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_include_full_directory_structure: Option<bool>,

    /// Enable token count tree display
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub token_count_tree: Option<bool>,
    /// Disable token count tree display
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_token_count_tree: Option<bool>,

    // --- Git Output Options ---
    /// Sort files by git change frequency
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub git_sort_by_changes: Option<bool>,
    /// Do not sort files by git change frequency
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_git_sort_by_changes: Option<bool>,

    /// Max number of commits to consider for git sort
    #[arg(long, value_name = "COUNT")]
    pub git_sort_by_changes_max_commits: Option<u32>, // Changed to u32

    /// Include git diffs in output
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub git_include_diffs: Option<bool>,
    /// Do not include git diffs
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_git_include_diffs: Option<bool>,

    /// Include git logs in output
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub git_include_logs: Option<bool>,
    /// Do not include git logs
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_git_include_logs: Option<bool>,

    /// Number of git log commits to include
    #[arg(long, value_name = "COUNT")]
    pub git_include_logs_count: Option<u32>, // Changed to u32

    // --- Include/Ignore Options ---
    /// Additional include glob patterns
    #[arg(long = "include", value_name = "PATTERN")]
    pub include_patterns: Vec<String>, // Renamed for clarity in CLI args

    /// Additional ignore glob patterns (applied before ignore files)
    #[arg(short = 'i', long = "ignore", value_name = "PATTERN")]
    pub ignore_patterns: Vec<String>, // Renamed for clarity in CLI args

    /// Use .gitignore files
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub use_gitignore: Option<bool>,
    /// Disable reading of .gitignore files
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_use_gitignore: Option<bool>,

    /// Use .ignore files
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub use_dot_ignore: Option<bool>,
    /// Disable reading of .ignore files
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_use_dot_ignore: Option<bool>,

    /// Use built-in default ignore patterns
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub use_default_patterns: Option<bool>,
    /// Disable built-in default ignore patterns
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_use_default_patterns: Option<bool>,

    // --- Security Options ---
    /// Enable scanning for sensitive data
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub enable_security_check: Option<bool>,
    /// Disable scanning for sensitive data
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub no_enable_security_check: Option<bool>,

    // --- Token Count Options ---
    /// Tokenizer encoding (e.g., o200k_base, cl100k_base)
    #[arg(long)]
    pub encoding: Option<String>,

    // --- Other Options ---
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
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputStyleCli {
    Xml,
    Markdown,
    Json,
    Plain,
}