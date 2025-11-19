use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::config::RepomixConfig;
use crate::core::{file, output, metrics, security, compress, remote};

pub struct FileStats {
    pub path: PathBuf,
    pub token_count: usize,
    pub char_count: usize,
}

pub struct PackResult {
    pub output: String,
    pub token_count: usize,
    pub total_files: usize,
    pub total_chars: usize,
    pub top_files: Vec<FileStats>,
    pub has_secrets: bool,
}

pub fn pack(config: &RepomixConfig, paths: &[PathBuf]) -> Result<PackResult> {
    // Check for remote repositories and clone if necessary
    let mut temp_dirs = Vec::new();
    let mut target_paths = Vec::new();

    for path_buf in paths {
        let path_str = path_buf.to_string_lossy();
        if remote::is_remote_url(&path_str) {
            let temp_dir = remote::clone_repo(&path_str)?;
            target_paths.push(temp_dir.path().to_path_buf());
            temp_dirs.push(temp_dir);
        } else {
            target_paths.push(path_buf.clone());
        }
    }

    // Collect files
    let walker = file::FileWalker::new(config.clone());
    let mut files = HashMap::new();
    let mut total_chars = 0;

    walker.walk(&target_paths, |path| {
        match file::read_file(&path, config) {
            Ok(Some(content)) => {
                tracing::debug!("Read file: {:?} (len: {})", path, content.len());
                
                // Security check
                if config.security.enable_security_check {
                    if let Ok(Some(result)) = security::scan_content(&path, &content) {
                        for secret in result.secrets {
                            tracing::warn!("Potential secret found in {:?}: {}", path, secret);
                        }
                    }
                }

                // Compression
                let final_content = if config.output.compress {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    match compress::compress_content(&content, ext) {
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
            Err(e) => {
                tracing::warn!("Failed to read file {:?}: {}", path, e);
            }
            Ok(None) => {}
        }
        Ok(())
    })?;

    tracing::info!("Found {} files, total characters: {}", files.len(), total_chars);

    // Generate output
    let output_result = output::format(config, &files)?;

    // Count tokens per file for statistics
    let mut file_stats: Vec<FileStats> = Vec::new();
    for (path, content) in &files {
        if let Ok(tokens) = metrics::count_tokens(content, &config.token_count.encoding) {
            file_stats.push(FileStats {
                path: path.clone(),
                token_count: tokens,
                char_count: content.len(),
            });
        }
    }
    
    // Sort by token count and take top 5
    file_stats.sort_by(|a, b| b.token_count.cmp(&a.token_count));
    let top_files: Vec<FileStats> = file_stats.into_iter().take(5).collect();

    // Count total tokens
    let token_count = metrics::count_tokens(&output_result.content, &config.token_count.encoding)?;
    tracing::info!("Generated output: {} tokens (encoding: {})", token_count, config.token_count.encoding);

    Ok(PackResult {
        output: output_result.content,
        token_count,
        total_files: files.len(),
        total_chars,
        top_files,
        has_secrets: false, // TODO: track actual security findings
    })
}
