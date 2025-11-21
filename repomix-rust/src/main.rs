mod cli;
mod config;
mod core;
mod shared;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use shared::logger;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    // Parse CLI arguments
    let mut args = Cli::parse();

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

    tracing::debug!("Repomix started with args: {:?}", args);

    let stdin_paths = if args.stdin {
        let cwd =
            std::env::current_dir().context("Failed to determine current working directory")?;
        core::file::read_stdin_file_paths(&cwd)?
    } else {
        Vec::new()
    };

    // Load configuration
    let config = config::RepomixConfig::load_from_file(args.config.clone())
        .context("Failed to load configuration")?
        .merge_with_cli(&args)
        .with_stdin_file_paths(stdin_paths);

    // Run packing
    println!("\n📦 Repomix v{}\n", env!("CARGO_PKG_VERSION"));

    let result = core::pack::pack(&config, &args.directories)?;

    println!("✔ Packing completed successfully!\n");

    // Display top files
    if !result.top_files.is_empty() {
        println!("📈 Top 5 Files by Token Count:");
        println!("──────────────────────────────");
        for (i, file_stat) in result.top_files.iter().enumerate() {
            let percentage =
                (file_stat.token_count as f64 / result.token_count as f64 * 100.0).round();
            println!(
                "{}. {} ({} tokens, {} chars, {}%)",
                i + 1,
                file_stat.path.display(),
                format_number(file_stat.token_count),
                format_number(file_stat.char_count),
                percentage
            );
        }
        println!();
    }

    // Security check
    println!("🔎 Security Check:");
    println!("──────────────────");
    if result.has_secrets {
        println!("⚠ Suspicious files detected.");
    } else {
        println!("✔ No suspicious files detected.");
    }
    println!("\n");

    // Print summary
    println!("📊 Pack Summary:");
    println!("────────────────────");
    println!("  Total Files: {} files", result.total_files);
    println!(
        " Total Tokens: {} tokens",
        format_number(result.token_count)
    );
    println!("  Total Chars: {} chars", format_number(result.total_chars));

    // Handle output
    if args.stdout {
        println!("       Output: stdout");
        println!(
            "     Security: {}",
            if result.has_secrets {
                "⚠ Suspicious files detected"
            } else {
                "✔ No suspicious files detected"
            }
        );
        println!();
        println!("{}", result.output);
    } else if let Some(output_path) = &config.output.file_path {
        std::fs::write(output_path, &result.output)
            .with_context(|| format!("Failed to write output to {:?}", output_path))?;
        println!("       Output: {}", output_path.display());
        println!(
            "     Security: {}",
            if result.has_secrets {
                "⚠ Suspicious files detected"
            } else {
                "✔ No suspicious files detected"
            }
        );
        println!();
    } else {
        println!("       Output: stdout");
        println!(
            "     Security: {}",
            if result.has_secrets {
                "⚠ Suspicious files detected"
            } else {
                "✔ No suspicious files detected"
            }
        );
        println!();
        println!("{}", result.output);
    }

    // Clipboard
    if args.copy || config.output.copy_to_clipboard {
        let mut clipboard = arboard::Clipboard::new().context("Failed to initialize clipboard")?;
        clipboard
            .set_text(&result.output)
            .context("Failed to copy to clipboard")?;
        println!("📋 Output copied to clipboard");
    }

    println!("🎉 All Done!");
    println!("Your repository has been successfully packed.\n");

    Ok(())
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
