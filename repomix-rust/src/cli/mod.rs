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
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputStyle {
    Xml,
    Markdown,
    Plain,
}
