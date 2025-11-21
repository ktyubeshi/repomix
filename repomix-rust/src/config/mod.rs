pub mod default_ignore;

use crate::cli::Cli;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepomixConfig {
    #[serde(skip)]
    pub cwd: PathBuf,
    #[serde(skip)]
    pub stdin_file_paths: Vec<PathBuf>,
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
    #[serde(default)]
    pub token_count: TokenCountConfig,
}

impl Default for RepomixConfig {
    fn default() -> Self {
        Self {
            cwd: current_dir_default(),
            stdin_file_paths: Vec::new(),
            input: InputConfig::default(),
            output: OutputConfig::default(),
            include: Vec::new(),
            ignore: IgnoreConfig::default(),
            security: SecurityConfig::default(),
            token_count: TokenCountConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InputConfig {
    // Default max file size: 50MB (matches TS default)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            max_file_size: default_max_file_size(),
        }
    }
}

fn default_max_file_size() -> u64 {
    50 * 1024 * 1024
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutputConfig {
    pub file_path: Option<PathBuf>,
    pub style: Option<String>,
    #[serde(default)]
    pub parsable_style: bool,
    #[serde(default)]
    pub compress: bool,
    #[serde(default)]
    pub copy_to_clipboard: bool,
    pub header_text: Option<String>,
    pub instruction_file_path: Option<PathBuf>,
    #[serde(default = "true_default")]
    pub file_summary: bool,
    #[serde(default = "true_default")]
    pub directory_structure: bool,
    #[serde(default = "true_default")]
    pub files: bool,
    #[serde(default)]
    pub remove_comments: bool,
    #[serde(default)]
    pub remove_empty_lines: bool,
    #[serde(default = "default_top_files_length")]
    pub top_files_length: usize,
    #[serde(default)]
    pub show_line_numbers: bool,
    #[serde(default = "true_default")]
    pub truncate_base64: bool,
    #[serde(default = "true_default")]
    pub include_empty_directories: bool,
    #[serde(default)]
    pub git: GitConfig,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            file_path: None,
            style: None,
            parsable_style: false,
            compress: false,
            copy_to_clipboard: false,
            header_text: None,
            instruction_file_path: None,
            file_summary: true_default(),
            directory_structure: true_default(),
            files: true_default(),
            remove_comments: false,
            remove_empty_lines: false,
            top_files_length: default_top_files_length(),
            show_line_numbers: false,
            include_empty_directories: true_default(),
            truncate_base64: true_default(),
            git: GitConfig::default(),
        }
    }
}

fn true_default() -> bool {
    true
}

fn default_top_files_length() -> usize {
    5
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitConfig {
    #[serde(default)]
    pub sort_by_changes: bool,
    #[serde(default)]
    pub sort_by_changes_max_commits: Option<usize>,
    #[serde(default)]
    pub include_diffs: bool,
    #[serde(default)]
    pub include_logs: bool,
    #[serde(default)]
    pub include_logs_count: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IgnoreConfig {
    #[serde(default = "true_default")]
    pub use_gitignore: bool,
    #[serde(default = "true_default")]
    pub use_dot_ignore: bool,
    #[serde(default = "true_default")]
    pub use_default_patterns: bool,
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            use_gitignore: true_default(),
            use_dot_ignore: true_default(),
            use_default_patterns: true_default(),
            custom_patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecurityConfig {
    #[serde(default)]
    pub enable_security_check: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenCountConfig {
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

impl Default for TokenCountConfig {
    fn default() -> Self {
        Self {
            encoding: default_encoding(),
        }
    }
}

fn default_encoding() -> String {
    "o200k_base".to_string()
}

impl RepomixConfig {
    pub fn load_from_file(path: Option<PathBuf>) -> Result<Self> {
        let cwd = env::current_dir().context("Failed to determine current working directory")?;
        let config_path = path.unwrap_or_else(|| PathBuf::from("repomix.config.json"));

        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file at {:?}", config_path))?;

            json5::from_str(&content).with_context(|| "Failed to parse config file")?
        } else {
            tracing::debug!("Config file not found at {:?}, using defaults", config_path);
            Self::default()
        };

        config.cwd = cwd;
        config.stdin_file_paths.clear();

        Ok(config)
    }

    pub fn merge_with_cli(mut self, cli: &Cli) -> Self {
        // Merge CLI arguments into config
        // CLI args take precedence

        if let Some(output) = &cli.output {
            self.output.file_path = Some(output.clone());
        }

        // Output style enum to string conversion if needed, or just use the enum in config too?
        // For now, let's keep config simple strings or map it.
        // The CLI uses an enum.
        let style_str = format!("{:?}", cli.style).to_lowercase();
        // If CLI style is default (Xml), we might not want to override config if config specifies something else?
        // But clap default_value_t makes it always have a value.
        // We should probably check if the user *explicitly* provided the arg.
        // But clap's struct doesn't easily tell us that unless we use Option.
        // In the CLI struct: `pub style: OutputStyle` with default value.
        // So it will always be XML if not provided.
        // If we want config to override default CLI, we need to know if CLI was user-provided.
        // For now, let's assume CLI always overrides config for simplicity,
        // OR we can change CLI struct to use Option for style.

        // Let's assume for now we overwrite with CLI value.
        self.output.style = Some(style_str);

        if cli.compress {
            self.output.compress = true;
        }

        if cli.output_show_line_numbers {
            self.output.show_line_numbers = true;
        }

        if cli.copy {
            self.output.copy_to_clipboard = true;
        }

        if !cli.include.is_empty() {
            self.include.extend(cli.include.iter().cloned());
        }

        if !cli.ignore.is_empty() {
            tracing::debug!("Adding CLI ignores: {:?}", cli.ignore);
            self.ignore
                .custom_patterns
                .extend(cli.ignore.iter().cloned());
        }

        if cli.no_gitignore {
            self.ignore.use_gitignore = false;
        }

        if cli.no_dot_ignore {
            self.ignore.use_dot_ignore = false;
        }

        if cli.no_default_patterns {
            self.ignore.use_default_patterns = false;
        }

        self
    }

    pub fn with_stdin_file_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.stdin_file_paths = paths;
        self
    }
}

fn current_dir_default() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
