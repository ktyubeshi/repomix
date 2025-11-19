mod cli;
mod config;
mod core;
mod shared;

use clap::Parser;
use cli::Cli;
use shared::logger;
use anyhow::{Result, Context};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    // Parse CLI arguments
    let args = cli::Cli::parse();

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
    
    shared::logger::init(&args);

    tracing::debug!("Repomix started with args: {:?}", args);

    // Load configuration
    let config = config::RepomixConfig::load_from_file(args.config.clone())
        .context("Failed to load configuration")?
        .merge_with_cli(&args);

    // Run packing
    println!("\n📦 Repomix v{}\n", env!("CARGO_PKG_VERSION"));
    
    let result = core::pack::pack(&config, &args.directories)?;

    println!("✔ Packing completed successfully!\n");

    // Print summary
    println!("📊 Pack Summary:");
    println!("────────────────────");
    println!("  Total Files: {} files", result.total_files);
    println!(" Total Tokens: {} tokens", result.token_count);
    println!("  Total Chars: {} chars", result.total_chars);
    
    // Handle output
    if args.stdout || config.output.file_path.is_none() {
        println!("       Output: stdout\n");
        println!("{}", result.output);
    } else {
        let output_path = config.output.file_path.as_ref().unwrap();
        std::fs::write(output_path, &result.output)
            .with_context(|| format!("Failed to write output to {:?}", output_path))?;
        println!("       Output: {}\n", output_path.display());
    }

    // Clipboard
    if args.copy || config.output.copy_to_clipboard {
        let mut clipboard = arboard::Clipboard::new().context("Failed to initialize clipboard")?;
        clipboard.set_text(&result.output).context("Failed to copy to clipboard")?;
        println!("📋 Output copied to clipboard");
    }

    println!("🎉 All Done!");
    println!("Your repository has been successfully packed.\n");

    Ok(())
}
