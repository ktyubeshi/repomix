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
    let result = core::pack::pack(&config, &args.directories)?;

    // Handle output
    if args.stdout || config.output.file_path.is_none() {
        // Write to stdout
        // We use println! here, but we might want to avoid mixing logs and output.
        // Logs are on stderr by default with tracing-subscriber fmt, so this is fine.
        println!("{}", result.output);
    } else {
        // Write to file
        let output_path = config.output.file_path.as_ref().unwrap();
        std::fs::write(output_path, &result.output)
            .with_context(|| format!("Failed to write output to {:?}", output_path))?;
        tracing::info!("Output written to {:?}", output_path);
    }

    // Clipboard
    if args.copy || config.output.copy_to_clipboard {
        let mut clipboard = arboard::Clipboard::new().context("Failed to initialize clipboard")?;
        clipboard.set_text(&result.output).context("Failed to copy to clipboard")?;
        tracing::info!("Output copied to clipboard");
    }

    Ok(())
}
