mod cli;
mod config;
mod core;
mod shared;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use config::schema::TokenCountTreeConfig;
use core::metrics::token_tree::render_token_tree;
use console::style;
use rustc_version_runtime::version as rustc_version;
use shared::logger;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    // Parse CLI arguments
    let mut args = Cli::parse();

    if args.version {
        print_version_info();
        return Ok(());
    }

    if args.init {
        let cwd = std::env::current_dir().context("Failed to get current working directory")?;
        return cli::init::run_init_action(&cwd, args.global);
    }

    // If --remote is provided, treat it as a target directory/URL
    if let Some(remote_url) = &args.remote {
        args.directories.push(std::path::PathBuf::from(remote_url));
    }

    if args.server {
        if let Err(e) = repomix::mcp::start_server() {
            tracing::error!("MCP server error: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    // Initialize logger (only if not in server mode, or handle logging differently)
    // For MCP, we should probably log to stderr so stdout is clean for JSON-RPC
    // The current logger setup logs to stdout by default?
    // Let's check shared::logger.

    logger::init(&args);
    let suppress_human_output = args.quiet || args.stdout || args.stdin;
    let show_human_output = !suppress_human_output;

    tracing::debug!("Repomix started with args: {:?}", args);

    let stdin_paths = if args.stdin {
        let cwd =
            std::env::current_dir().context("Failed to determine current working directory")?;
        core::file::read_stdin_file_paths(&cwd)?
    } else {
        Vec::new()
    };

    // Load configuration
    let mut config = config::load::load_file_config(
        &std::env::current_dir().context("Failed to get current working directory")?,
        args.config.as_ref().map(|p| p.as_path()),
    )
    .await
    .context("Failed to load configuration")?;

    config = config.merge_with_cli(&args);
    config.stdin_file_paths = stdin_paths;
    if config.cwd.as_os_str().is_empty() {
        config.cwd =
            std::env::current_dir().context("Failed to determine current working directory")?;
    }

    // Run packing
    if show_human_output {
        println!(
            "\n{}\n",
            style(format!("📦 Repomix-rs v{}", env!("CARGO_PKG_VERSION"))).dim()
        );
    }

    let result = core::pack::pack(&config, &args.directories)?;

    if show_human_output {
        println!("{}", style("✔ Packing completed successfully!").green());
        println!();

        // Display top files
        let top_files_len = config.output.top_files_length as usize;
        if top_files_len > 0 && !result.top_files.is_empty() {
            println!(
                "{}",
                style(format!(
                    "📈 Top {} File{} by Token Count:",
                    top_files_len,
                    if top_files_len == 1 { "" } else { "s" }
                ))
                .white()
            );
            println!("{}", style("──────────────────────────────").dim());
            for (i, file_stat) in result.top_files.iter().enumerate() {
                let percentage = if result.token_count > 0 {
                    (file_stat.token_count as f64 / result.token_count as f64 * 100.0).round()
                } else {
                    0.0
                };
                println!(
                    "{} {}{}",
                    style(format!("{}.", i + 1)).white().bold(),
                    style(file_stat.path.display()).white(),
                    style(format!(
                        " ({} tokens, {} chars, {}%)",
                        format_number(file_stat.token_count),
                        format_number(file_stat.char_count),
                        percentage
                    ))
                    .dim()
                );
            }
            println!();
        }

        if let Some(tree) = &result.token_count_tree {
            if let Some(threshold) = token_tree_threshold(&config.output.token_count_tree) {
                println!("{}", style("🔢 Token Count Tree:").white().bold());
                println!("{}", style("────────────────────").dim());
                if threshold > 0 {
                    println!(
                        "{}",
                        style(format!("Showing entries with {}+ tokens:", threshold)).dim()
                    );
                }
                let lines = render_token_tree(tree, threshold);
                if lines.is_empty() {
                    println!("{}", style("No files found.").dim());
                } else {
                    for line in lines {
                        println!("{}", style(line).white());
                    }
                }
                println!();
            }
        }

        // Security check
        println!("{}", style("🔎 Security Check:").white());
        println!("{}", style("──────────────────").dim());
        if result.has_secrets {
            println!(
                "{}",
                style(format!(
                    "{} suspicious file(s) detected and excluded from the output:",
                    result.suspicious_files.len()
                ))
                .yellow()
            );
            for (idx, file) in result.suspicious_files.iter().enumerate() {
                println!(
                    "{} {}",
                    style(format!("{}. ", idx + 1)).white(),
                    style(file.display()).white()
                );
            }
            println!(
                "{}",
                style("These files have been excluded from the output for security reasons.")
                    .yellow()
            );
            println!(
                "{}",
                style("Please review these files for potential sensitive information.").yellow()
            );
        } else {
            println!(
                "{}",
                style("✔ No suspicious files detected.").green()
            );
        }
        println!("\n");

        // Print summary
        println!("{}", style("📊 Pack Summary:").white().bold());
        println!("{}", style("────────────────────").dim());
        println!(
            "  {} {}",
            style("Total Files:").white(),
            style(format!("{} files", result.total_files)).white()
        );
        println!(
            " {} {}",
            style("Total Tokens:").white(),
            style(format!("{} tokens", format_number(result.token_count))).white()
        );
        println!(
            "  {} {}",
            style("Total Chars:").white(),
            style(format!("{} chars", format_number(result.total_chars))).white()
        );
        let output_target = if args.stdout {
            "stdout".to_string()
        } else if let Some(path) = &config.output.file_path {
            path.clone()
        } else {
            "stdout".to_string()
        };
        println!(
            "       {} {}",
            style("Output:").white(),
            style(output_target).white()
        );
        println!(
            "     {} {}",
            style("Security:").white(),
            if result.has_secrets {
                style(format!(
                    "⚠ {} suspicious file(s) detected and excluded",
                    result.suspicious_files.len()
                ))
                .yellow()
                .bold()
            } else {
                style("✔ No suspicious files detected".to_string()).green()
            }
        );
        println!(
            "    {} {}",
            style("Git Diffs:").white(),
            if config.output.git.include_diffs {
                style("✔ Git diffs included").white()
            } else {
                style("✖ No git diffs included").dim()
            }
        );
        println!(
            "     {} {}",
            style("Git Logs:").white(),
            if config.output.git.include_logs {
                style("✔ Git logs included").white()
            } else {
                style("✖ No git logs included").dim()
            }
        );
    }

    // Handle output
    if args.stdout {
        print!("{}", result.output);
    } else if let Some(output_path) = &config.output.file_path {
        std::fs::write(output_path, &result.output)
            .with_context(|| format!("Failed to write output to {:?}", output_path))?;
        if show_human_output {
            println!();
        }
    } else {
        if show_human_output {
            println!();
        }
        print!("{}", result.output);
    }

    // Clipboard
    if config.output.copy_to_clipboard {
        let mut clipboard = arboard::Clipboard::new().context("Failed to initialize clipboard")?;
        clipboard
            .set_text(&result.output)
            .context("Failed to copy to clipboard")?;
        if show_human_output {
            println!("{}", style("📋 Output copied to clipboard").white());
        }
    }

    if show_human_output {
        println!("{}", style("🎉 All Done!").green());
        println!(
            "{}",
            style("Your repository has been successfully packed.").white()
        );
        println!();
        println!(
            "💡 Repomix is now available in your browser! Try it at {}",
            style("https://repomix.com").underlined()
        );
    }

    Ok(())
}

fn print_version_info() {
    let pkg_version = env!("CARGO_PKG_VERSION");
    let rustc_ver = rustc_version();
    let platform = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);

    println!("Repomix (Rust) v{}", pkg_version);
    println!("Runtime: rustc {}", rustc_ver);
    println!("Platform: {}", platform);
}

fn token_tree_threshold(config_value: &TokenCountTreeConfig) -> Option<usize> {
    match config_value {
        TokenCountTreeConfig::Bool(false) => None,
        TokenCountTreeConfig::Bool(true) => Some(0),
        TokenCountTreeConfig::Threshold(n) => Some(*n as usize),
        TokenCountTreeConfig::Text(_) => Some(0),
    }
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
