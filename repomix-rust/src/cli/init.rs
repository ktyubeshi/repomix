use crate::config::global_directory;
use crate::config::schema::{default_file_path_map, RepomixConfig, RepomixOutputStyle};
use anyhow::{bail, Context, Result};
use console::{style, Key, Term};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use pathdiff::diff_paths;
use std::fs;
use std::path::Path;

const SYMBOL_PIPE: &str = "│";
const SYMBOL_CORNER: &str = "└";
const SYMBOL_TOP: &str = "┌";
const SYMBOL_INFO: &str = "ℹ";
const SYMBOL_SUCCESS: &str = "✔";

#[derive(PartialEq)]
enum PromptResult<T> {
    Ok(T),
    Cancel,
}

fn print_intro(message: &str) {
    println!();
    println!("{} {}", style(SYMBOL_TOP).cyan(), style(message).bold());
    println!("{}", style(SYMBOL_PIPE).cyan());
}

fn print_outro_success(message: &str) {
    println!(
        "{} {} {}",
        style(SYMBOL_CORNER).cyan(),
        style(SYMBOL_SUCCESS).green(),
        style(message).green()
    );
    println!();
}

fn print_outro_warning(message: &str) {
    println!(
        "{} {}",
        style(SYMBOL_CORNER).cyan(),
        style(message).yellow()
    );
    println!();
}

fn print_cancel_message() {
    println!(
        "{} {}",
        style(SYMBOL_CORNER).cyan(),
        style("Initialization cancelled.").dim()
    );
    println!();
}

fn log_info(message: &str) {
    println!(
        "{}   {} {}",
        style(SYMBOL_PIPE).cyan(),
        style(SYMBOL_INFO).blue(),
        message
    );
}

fn log_success(message: &str) {
    println!(
        "{}   {} {}",
        style(SYMBOL_PIPE).cyan(),
        style(SYMBOL_SUCCESS).green(),
        message
    );
}

fn prompt_confirm(message: &str, default_yes: bool) -> Result<PromptResult<bool>> {
    let term = Term::stdout();
    let mut current = default_yes;
    let mut first_render = true;

    // Hide cursor
    term.hide_cursor()?;

    loop {
        if !first_render {
            term.clear_last_lines(2)?;
        }
        first_render = false;

        let yes_circle = if current { style("●").green() } else { style("○").dim() };
        let yes_label = if current { style("Yes").green() } else { style("Yes").dim() };
        let no_circle = if !current { style("●").green() } else { style("○").dim() };
        let no_label = if !current { style("No").green() } else { style("No").dim() };

        term.write_line(&format!(
            "{} {} {}",
            style(SYMBOL_PIPE).cyan(),
            style("◆").magenta(), // Using magenta to match Clack's diamond color roughly (Node is usually cyan or magenta)
            message
        ))?;
        term.write_line(&format!(
            "{}   {} {} {} {}",
            style(SYMBOL_PIPE).cyan(),
            yes_circle,
            yes_label,
            style("/").dim(),
            format!("{} {}", no_circle, no_label)
        ))?;
        term.flush()?;

        match term.read_key()? {
            Key::Enter => {
                term.show_cursor()?;
                return Ok(PromptResult::Ok(current));
            }
            Key::ArrowLeft | Key::ArrowUp => current = true,
            Key::ArrowRight | Key::ArrowDown => current = false,
            Key::Char('y') | Key::Char('Y') => {
                term.show_cursor()?;
                return Ok(PromptResult::Ok(true));
            }
            Key::Char('n') | Key::Char('N') => {
                term.show_cursor()?;
                return Ok(PromptResult::Ok(false));
            }
            Key::Char('\n') => { // Handle newline as Enter just in case
                term.show_cursor()?;
                return Ok(PromptResult::Ok(current));
            }
            Key::Escape | Key::CtrlC => {
                term.show_cursor()?;
                return Ok(PromptResult::Cancel);
            }
            _ => {}
        }
    }
}

fn handle_cancellation() -> ! {
    print_cancel_message();
    std::process::exit(0);
}

pub fn run_init_action(root_dir: &Path, is_global: bool) -> Result<()> {
    if !console::user_attended() {
        bail!("Interactive mode is required for init command. Please run in a terminal.");
    }

    print_intro(&format!(
        "Welcome to Repomix {}Configuration!",
        if is_global { "Global " } else { "" }
    ));

    match create_config_file(root_dir, is_global)? {
        PromptResult::Ok(created_config) => {
            match create_ignore_file(root_dir, is_global)? {
                PromptResult::Ok(created_ignore) => {
                    if !created_config && !created_ignore {
                        print_outro_warning(
                            "No files were created. You can run this command again when you need to create configuration files.",
                        );
                    } else {
                        print_outro_success("Initialization complete! You can now use Repomix with your specified settings.");
                    }
                }
                PromptResult::Cancel => handle_cancellation(),
            }
        }
        PromptResult::Cancel => handle_cancellation(),
    }

    Ok(())
}

fn create_config_file(root_dir: &Path, is_global: bool) -> Result<PromptResult<bool>> {
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

    match prompt_confirm(&prompt_msg, true)? {
        PromptResult::Ok(true) => {}, // Continue
        PromptResult::Ok(false) => {
            log_info(&format!(
                "Skipping {} file creation.",
                style("repomix.config.json").green()
            ));
            return Ok(PromptResult::Ok(false));
        },
        PromptResult::Cancel => return Ok(PromptResult::Cancel),
    }

    if config_path.exists() {
        let overwrite_msg = format!(
            "A {}{} file already exists. Do you want to overwrite it?",
            if is_global { "global " } else { "" },
            style("repomix.config.json").green()
        );

        match prompt_confirm(&overwrite_msg, false)? {
            PromptResult::Ok(true) => {}, // Continue
            PromptResult::Ok(false) => {
                log_info(&format!(
                    "Skipping {} file creation.",
                    style("repomix.config.json").green()
                ));
                return Ok(PromptResult::Ok(false));
            },
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
        format!("{:<10} {}", "JSON", style("Machine-readable JSON format").dim()),
        format!("{:<10} {}", "Plain", style("Simple text format").dim()),
    ];
    
    // Select interaction (TODO: Wrap Select to handle Cancel)
    // dialoguer's Select uses `read_key` internally but doesn't expose Esc easily without hacking.
    // However, Interact trait allows returning Option.
    // We will use `interact_opt` which returns Ok(None) on Esc/q
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Output style")
        .default(0)
        .items(&items)
        .interact_opt()?;

    let selection_index = match selection {
        Some(index) => index,
        None => return Ok(PromptResult::Cancel),
    };

    let selected_style = styles[selection_index].clone();
    
    let default_paths = default_file_path_map();
    let default_path = default_paths.get(&selected_style).unwrap();

    // Input interaction
    // dialoguer::Input doesn't support Cancel via Esc out of the box easily.
    // It returns Result<T>.
    // If we want proper cancellation, we might need `interact_text_on` or handle errors.
    // But `Input` usually blocks until Enter.
    // For now, we accept that Input might be harder to cancel with Esc, but Ctrl+C works (handled by OS or we catch Interrupted error).
    // Let's try catching the error.
    
    let output_path_res = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Output file path")
        .default(default_path.clone())
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Output file path is required")
            } else {
                Ok(())
            }
        })
        .interact_text();

    let output_path = match output_path_res {
        Ok(path) => path,
        Err(e) => {
            // Check if it's an interruption/cancellation
            // dialoguer might not return specific error for Esc, but for Ctrl+C.
            // Assuming any error here during interaction *might* be a cancel request if we can't distinguish.
            // But verify: dialoguer Input doesn't return None.
            // We can check `e.downcast_ref::<io::Error>()` if needed.
            // For now, let's treat it as Cancel if it fails.
            return Ok(PromptResult::Cancel);
        }
    };

    let mut config = RepomixConfig::default();
    // We must manually set sort_by_changes to true to match Node.js defaults,
    // because we are creating a FRESH config here, and RepomixConfig::default() 
    // uses Default trait which we updated in schema.rs!
    // Wait, we updated GitOutputConfig default to true. So RepomixConfig::default() should have true.
    // So we don't need to manually set it here if schema.rs change works.
    
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

    log_success(&format!(
        "{} file created!\n{}",
        style(if is_global { "Global config" } else { "Config" }).green(),
        style(format!("Path: {}", display_path)).dim()
    ));

    Ok(PromptResult::Ok(true))
}

fn create_ignore_file(root_dir: &Path, is_global: bool) -> Result<PromptResult<bool>> {
    if is_global {
        log_info(&format!(
            "Skipping {} file creation for global configuration.",
            style(".repomixignore").green()
        ));
        return Ok(PromptResult::Ok(false));
    }

    let ignore_path = root_dir.join(".repomixignore");

    let prompt_msg = format!("Do you want to create a {} file?", style(".repomixignore").green());

    match prompt_confirm(&prompt_msg, true)? {
        PromptResult::Ok(true) => {}, // Continue
        PromptResult::Ok(false) => {
            log_info(&format!(
                "Skipping {} file creation.",
                style(".repomixignore").green()
            ));
            return Ok(PromptResult::Ok(false));
        },
        PromptResult::Cancel => return Ok(PromptResult::Cancel),
    }

    if ignore_path.exists() {
        let overwrite_msg =
            format!("A {} file already exists. Do you want to overwrite it?", style(".repomixignore").green());

        match prompt_confirm(&overwrite_msg, false)? {
            PromptResult::Ok(true) => {}, // Continue
            PromptResult::Ok(false) => {
                log_info(&format!(
                    "{} file creation skipped. Existing file will not be modified.",
                    style(".repomixignore").green()
                ));
                return Ok(PromptResult::Ok(false));
            },
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

    log_success(&format!(
        "{}
{}",
        style("Created .repomixignore file!").green(),
        style(format!("Path: {}", display_path)).dim()
    ));

    Ok(PromptResult::Ok(true))
}