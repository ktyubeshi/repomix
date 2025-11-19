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
    logger::init();

    // Parse CLI arguments
    let args = Cli::parse();

    tracing::debug!("Repomix started with args: {:?}", args);

    // Load configuration
    let config = config::RepomixConfig::load_from_file(args.config.clone())
        .context("Failed to load configuration")?
        .merge_with_cli(&args);

    tracing::debug!("Configuration loaded: {:?}", config);

    // Collect files
    let walker = core::file::FileWalker::new(config.clone());
    let mut files = std::collections::HashMap::new();
    let mut total_chars = 0;

    walker.walk(&args.directories, |path| {
        match core::file::read_file(&path, &config) {
            Ok(Some(content)) => {
                tracing::debug!("Read file: {:?} (len: {})", path, content.len());
                
                // Security check
                if config.security.enable_security_check {
                    if let Ok(Some(result)) = core::security::scan_content(&path, &content) {
                        for secret in result.secrets {
                            tracing::warn!("Potential secret found in {:?}: {}", path, secret);
                        }
                    }
                }

                // Compression
                let final_content = if config.output.compress {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    match core::compress::compress_content(&content, ext) {
                        Ok(c) => {
                            tracing::debug!("Compressed {:?} ({} -> {} chars)", path, content.len(), c.len());
                            c
                        }
                        Err(e) => {
                            tracing::warn!("Failed to compress {:?}: {}", path, e);
                            content
                        }
                    }
                } else {
                    content
                };

                total_chars += final_content.len();
                files.insert(path, final_content);
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!("Failed to read file {:?}: {}", path, e);
            }
        }
        Ok(())
    })?;

    tracing::info!("Found {} files, total characters: {}", files.len(), total_chars);

    // Generate output
    let output = core::output::format(&config, &files)?;

    // Count tokens
    let token_count = core::metrics::count_tokens(&output.content, &config.token_count.encoding)?;
    tracing::info!("Generated output: {} tokens (encoding: {})", token_count, config.token_count.encoding);

    // Write output
    if args.stdout {
        println!("{}", output.content);
    } else {
        let output_path = config.output.file_path.clone().unwrap_or_else(|| PathBuf::from("repomix-output.xml"));
        std::fs::write(&output_path, &output.content)?;
        tracing::info!("Output written to {:?}", output_path);
    }

    // Copy to clipboard
    if args.copy || config.output.copy_to_clipboard {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&output.content) {
                    tracing::error!("Failed to copy to clipboard: {}", e);
                } else {
                    tracing::info!("Output copied to clipboard");
                }
            }
            Err(e) => {
                tracing::error!("Failed to initialize clipboard: {}", e);
            }
        }
    }

    Ok(())
}
