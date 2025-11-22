use crate::cli::tui::{PromptResult, Tui};
use crate::config::global_directory;
use crate::config::schema::{default_file_path_map, RepomixConfig, RepomixOutputStyle};
use anyhow::{bail, Context, Result};
use console::style;
use pathdiff::diff_paths;
use std::fs;
use std::path::Path;

pub fn run_init_action(root_dir: &Path, is_global: bool) -> Result<()> {
    if !console::user_attended() {
        bail!("Interactive mode is required for init command. Please run in a terminal.");
    }

    let tui = Tui::new();

    tui.intro(&format!(
        "Welcome to Repomix {}Configuration!",
        if is_global { "Global " } else { "" }
    ));

    match create_config_file(&tui, root_dir, is_global)? {
        PromptResult::Ok(created_config) => match create_ignore_file(&tui, root_dir, is_global)? {
            PromptResult::Ok(created_ignore) => {
                if !created_config && !created_ignore {
                    tui.outro_warning(
                        "No files were created. You can run this command again when you need to create configuration files.",
                    );
                } else {
                    tui.outro_success(
                        "Initialization complete! You can now use Repomix with your specified settings.",
                    );
                }
            }
            PromptResult::Cancel => tui.cancel_and_exit(),
        },
        PromptResult::Cancel => tui.cancel_and_exit(),
    }

    Ok(())
}

fn create_config_file(tui: &Tui, root_dir: &Path, is_global: bool) -> Result<PromptResult<bool>> {
    let target_dir = if is_global {
        global_directory::get_global_directory()? 
    } else {
        root_dir.to_path_buf()
    };

    let config_path = target_dir.join("repomix.config.json");

    let prompt_msg = format!(
        "Do you want to create a {}{} file?",
        if is_global { "global " } else { "" },
        style("repomix.config.json").green()
    );

    match tui.confirm(&prompt_msg, true)? {
        PromptResult::Ok(true) => {} 
        PromptResult::Ok(false) => {
            tui.log_info(&format!(
                "Skipping {} file creation.",
                style("repomix.config.json").green()
            ));
            return Ok(PromptResult::Ok(false));
        }
        PromptResult::Cancel => return Ok(PromptResult::Cancel),
    }

    if config_path.exists() {
        let overwrite_msg = format!(
            "A {}{} file already exists. Do you want to overwrite it?",
            if is_global { "global " } else { "" },
            style("repomix.config.json").green()
        );

        match tui.confirm(&overwrite_msg, false)? {
            PromptResult::Ok(true) => {} 
            PromptResult::Ok(false) => {
                tui.log_info(&format!(
                    "Skipping {} file creation.",
                    style("repomix.config.json").green()
                ));
                return Ok(PromptResult::Ok(false));
            }
            PromptResult::Cancel => return Ok(PromptResult::Cancel),
        }
    }

    let styles = vec![
        RepomixOutputStyle::Xml,
        RepomixOutputStyle::Markdown,
        RepomixOutputStyle::Json,
        RepomixOutputStyle::Plain,
    ];

    let items: Vec<String> = vec![
        format!("{:<10} {}", "XML", style("Structured XML format").dim()),
        format!("{:<10} {}", "Markdown", style("Markdown format").dim()),
        format!(
            "{:<10} {}",
            "JSON",
            style("Machine-readable JSON format").dim()
        ),
        format!("{:<10} {}", "Plain", style("Simple text format").dim()),
    ];

    let selection_index = match tui.select("Output style", &items, 0)? {
        PromptResult::Ok(index) => index,
        PromptResult::Cancel => return Ok(PromptResult::Cancel),
    };

    let selected_style = styles[selection_index].clone();
    let default_paths = default_file_path_map();
    let default_path = default_paths.get(&selected_style).unwrap();

    let output_path = match tui.input(
        "Output file path",
        default_path,
        |input: &str| -> Result<(), String> {
            if input.trim().is_empty() {
                Err("Output file path is required".to_string())
            } else {
                Ok(())
            }
        },
    )? {
        PromptResult::Ok(path) => path,
        PromptResult::Cancel => return Ok(PromptResult::Cancel),
    };

    let mut config = RepomixConfig::default();
    config.output.style = selected_style;
    config.output.file_path = Some(output_path);
    config.schema = Some("https://repomix.com/schemas/latest/schema.json".to_string());

    fs::create_dir_all(&target_dir).context("Failed to create configuration directory")?;

    let config_json = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, config_json).context("Failed to write config file")?;

    let display_path = diff_paths(&config_path, root_dir)
        .unwrap_or(config_path.clone())
        .display()
        .to_string();

    tui.log_success(&format!(
        "Config file created!\n{}",
        style(format!("Path: {}", display_path)).dim()
    ));

    Ok(PromptResult::Ok(true))
}

fn create_ignore_file(tui: &Tui, root_dir: &Path, is_global: bool) -> Result<PromptResult<bool>> {
    if is_global {
        tui.log_info(&format!(
            "Skipping {} file creation for global configuration.",
            style(".repomixignore").green()
        ));
        return Ok(PromptResult::Ok(false));
    }

    let ignore_path = root_dir.join(".repomixignore");

    let prompt_msg = format!(
        "Do you want to create a {} file?",
        style(".repomixignore").green()
    );

    match tui.confirm(&prompt_msg, true)? {
        PromptResult::Ok(true) => {} 
        PromptResult::Ok(false) => {
            tui.log_info(&format!(
                "Skipping {} file creation.",
                style(".repomixignore").green()
            ));
            return Ok(PromptResult::Ok(false));
        },
        PromptResult::Cancel => return Ok(PromptResult::Cancel),
    }

    if ignore_path.exists() {
        let overwrite_msg = format!(
            "A {} file already exists. Do you want to overwrite it?",
            style(".repomixignore").green()
        );

        match tui.confirm(&overwrite_msg, false)? {
            PromptResult::Ok(true) => {} 
            PromptResult::Ok(false) => {
                tui.log_info(&format!(
                    "{} file creation skipped. Existing file will not be modified.",
                    style(".repomixignore").green()
                ));
                return Ok(PromptResult::Ok(false));
            }
            PromptResult::Cancel => return Ok(PromptResult::Cancel),
        }
    }

    let content = r###"# Add patterns to ignore here, one per line
# Example:
# *.log
# tmp/ 
"###;

    fs::write(&ignore_path, content).context("Failed to write .repomixignore file")?;

    let display_path = diff_paths(&ignore_path, root_dir)
        .unwrap_or(ignore_path.clone())
        .display()
        .to_string();

    tui.log_success(&format!(
        "Created .repomixignore file!\n{}",
        style(format!("Path: {}", display_path)).dim()
    ));

    Ok(PromptResult::Ok(true))
}