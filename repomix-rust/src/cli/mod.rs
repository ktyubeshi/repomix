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

    /// Specify the output file name
    #[arg(short, long)]
    pub output: Option<PathBuf>,

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

    /// Enable verbose logging
    #[arg(long, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Disable all output to stdout
    #[arg(long, conflicts_with = "verbose")]
    pub quiet: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputStyle {
    Xml,
    Markdown,
    Plain,
}
