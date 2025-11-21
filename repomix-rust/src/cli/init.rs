use crate::config::global_directory;
use crate::config::schema::{RepomixConfig, RepomixOutputStyle, default_file_path_map};
use anyhow::{Result, Context, bail};
use console::style;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use std::fs;
use std::path::Path;

pub fn run_init_action(root_dir: &Path, is_global: bool) -> Result<()> {
    if !console::user_attended() {
        bail!("Interactive mode is required for init command. Please run in a terminal.");
    }

    println!();
    println!("{}", style(format!("Welcome to Repomix {}Configuration!", if is_global { "Global " } else { "" })).bold());
    println!();

    let created_config = create_config_file(root_dir, is_global)?;
    let created_ignore = create_ignore_file(root_dir, is_global)?;

    if !created_config && !created_ignore {
        println!("{}", style("No files were created. You can run this command again when you need to create configuration files.").yellow());
    } else {
        println!("{}", style("Initialization complete! You can now use Repomix with your specified settings.").green());
    }

    Ok(())
}

fn create_config_file(root_dir: &Path, is_global: bool) -> Result<bool> {
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

    let should_create = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(&prompt_msg)
        .default(true)
        .interact()?;

    if !should_create {
        println!("{} Skipping {} file creation.", style("◇").dim(), style("repomix.config.json").green());
        return Ok(false);
    }

    if config_path.exists() {
        let overwrite_msg = format!(
            "A {}{} file already exists. Do you want to overwrite it?",
            if is_global { "global " } else { "" },
            style("repomix.config.json").green()
        );

        let overwrite = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(&overwrite_msg)
            .default(false)
            .interact()?;

        if !overwrite {
             println!("{} Skipping {} file creation.", style("◇").dim(), style("repomix.config.json").green());
             return Ok(false);
        }
    }

    let styles = vec![
        RepomixOutputStyle::Xml,
        RepomixOutputStyle::Markdown,
        RepomixOutputStyle::Json,
        RepomixOutputStyle::Plain,
    ];
    
    // Create styled items to mimic Node.js layout: "Label      Hint"
    let items: Vec<String> = vec![
        format!("{:<10} {}", "XML", style("Structured XML format").dim()),
        format!("{:<10} {}", "Markdown", style("Markdown format").dim()),
        format!("{:<10} {}", "JSON", style("Machine-readable JSON format").dim()),
        format!("{:<10} {}", "Plain", style("Simple text format").dim()),
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Output style")
        .default(0)
        .items(&items)
        .interact()?;

    let selected_style = styles[selection].clone();
    
    let default_paths = default_file_path_map();
    let default_path = default_paths.get(&selected_style).unwrap();

    let output_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Output file path")
        .default(default_path.clone())
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Output file path is required")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let mut config = RepomixConfig::default();
    config.output.style = selected_style;
    config.output.file_path = Some(output_path);
    config.schema = Some("https://repomix.com/schemas/latest/schema.json".to_string());

    fs::create_dir_all(&target_dir).context("Failed to create configuration directory")?;

    let config_json = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, config_json).context("Failed to write config file")?;

    let display_path = if is_global {
        config_path.display().to_string()
    } else {
        match config_path.strip_prefix(root_dir) {
            Ok(p) => p.display().to_string(),
            Err(_) => config_path.display().to_string(),
        }
    };

    println!("{} {}{}", 
        style("✔").green(), 
        style(format!("{} file created!\n", if is_global { "Global config" } else { "Config" })).green(),
        style(format!("Path: {}", display_path)).dim()
    );

    Ok(true)
}

fn create_ignore_file(root_dir: &Path, is_global: bool) -> Result<bool> {
    if is_global {
        println!("{} Skipping {} file creation for global configuration.", style("◇").dim(), style(".repomixignore").green());
        return Ok(false);
    }

    let ignore_path = root_dir.join(".repomixignore");

    let prompt_msg = format!("Do you want to create a {} file?", style(".repomixignore").green());

    let should_create = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(&prompt_msg)
        .default(true)
        .interact()?;

    if !should_create {
        println!("{} Skipping {} file creation.", style("◇").dim(), style(".repomixignore").green());
        return Ok(false);
    }

    if ignore_path.exists() {
        let overwrite_msg = format!("A {} file already exists. Do you want to overwrite it?", style(".repomixignore").green());
        
        let overwrite = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(&overwrite_msg)
            .default(false)
            .interact()?;

        if !overwrite {
            println!("{} {} file creation skipped. Existing file will not be modified.", style("◇").dim(), style(".repomixignore").green());
            return Ok(false);
        }
    }

    let content = r###"# Add patterns to ignore here, one per line
# Example:
# *.log
# tmp/"###;

    fs::write(&ignore_path, content).context("Failed to write .repomixignore file")?;
    
    let display_path = match ignore_path.strip_prefix(root_dir) {
        Ok(p) => p.display().to_string(),
        Err(_) => ignore_path.display().to_string(),
    };

    println!("{} {}{}", 
        style("✔").green(),
        style("Created .repomixignore file!\n").green(),
        style(format!("Path: {}", display_path)).dim()
    );

    Ok(true)
}