use crate::config::schema::RepomixConfig;
use crate::core::{compress, file, metrics, output, remote, security};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

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
    pub suspicious_files: Vec<PathBuf>,
}

pub fn pack(config: &RepomixConfig, paths: &[PathBuf]) -> Result<PackResult> {
    // Check for remote repositories and clone if necessary
    let mut temp_dirs = Vec::new();
    let mut target_paths = Vec::new();

    for path_buf in paths {
        let path_str = path_buf.to_string_lossy();
        if remote::is_remote_url(&path_str) {
            let temp_dir = remote::clone_repo(&path_str, config.remote_branch.as_deref())?;
            target_paths.push(temp_dir.path().to_path_buf());
            temp_dirs.push(temp_dir);
        } else {
            target_paths.push(path_buf.clone());
        }
    }

    // Collect files
    let walker = file::FileWalker::new(config.clone())?;
    let files = std::sync::Mutex::new(HashMap::new());
    let suspicious_files = std::sync::Mutex::new(Vec::new());
    let total_chars = std::sync::atomic::AtomicUsize::new(0);
    let total_bytes = std::sync::atomic::AtomicUsize::new(0);
    let file_stats = std::sync::Mutex::new(Vec::new());
    let enc = config.token_count.encoding.clone();

    // Use parallel walker for better performance (read -> check -> compress -> count tokens)
    walker.walk_parallel(&target_paths, |absolute_path, relative_path| {
        match file::read_file(&absolute_path, config) {
            Ok(Some(content)) => {
                tracing::debug!("Read file: {:?} (len: {})", absolute_path, content.len());

                // Security check
                if config.security.enable_security_check {
                    if let Ok(Some(result)) = security::scan_content(&absolute_path, &content) {
                        for secret in result.secrets {
                            tracing::warn!(
                                "Potential secret found in {:?}: {}",
                                absolute_path,
                                secret
                            );
                        }
                        suspicious_files.lock().unwrap().push(relative_path);
                        return Ok(());
                    }
                }

                // Compression
                let final_content = if config.output.compress {
                    let ext = absolute_path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    match compress::compress_content(&content, ext) {
                        Ok(c) => {
                            tracing::debug!(
                                "Compressed {:?} ({} -> {} chars)",
                                absolute_path,
                                content.len(),
                                c.len()
                            );
                            c
                        }
                        Err(e) => {
                            tracing::warn!("Failed to compress {:?}: {}", absolute_path, e);
                            content
                        }
                    }
                } else {
                    content
                };

                // Count tokens immediately (parallelized)
                let token_count = metrics::count_tokens(&final_content, &enc).unwrap_or(0);
                let char_count = final_content.chars().count();

                total_chars.fetch_add(char_count, std::sync::atomic::Ordering::Relaxed);
                total_bytes.fetch_add(final_content.len(), std::sync::atomic::Ordering::Relaxed);

                {
                    let mut stats = file_stats.lock().unwrap();
                    stats.push(FileStats {
                        path: relative_path.clone(),
                        token_count,
                        char_count,
                    });
                }

                {
                    let mut map = files.lock().unwrap();
                    map.insert(relative_path, final_content);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read file {:?}: {}", absolute_path, e);
            }
            Ok(None) => {}
        }
        Ok(())
    })?;

    // Extract results from synchronisation primitives
    let files = files.into_inner().unwrap();
    let suspicious_files = suspicious_files.into_inner().unwrap();
    let total_chars = total_chars.into_inner();
    let total_bytes = total_bytes.into_inner();
    let mut file_stats = file_stats.into_inner().unwrap();

    tracing::info!(
        "Found {} files, total characters: {}",
        files.len(),
        total_chars
    );

    // Generate output
    tracing::info!("Generating output file...");
    let output_result = output::format(config, &files)?;
    tracing::info!(
        "Output generation completed ({} bytes)",
        output_result.content.len()
    );

    // Calculate total tokens efficiently
    let files_token_count: usize = file_stats.iter().map(|f| f.token_count).sum();

    // Sort by token count and take top 5
    file_stats.sort_by(|a, b| b.token_count.cmp(&a.token_count));
    let top_files: Vec<FileStats> = file_stats.into_iter().take(5).collect();

    // Calculate tokens for metadata (header, directory structure, etc.)
    // This is much smaller than the entire output, so it's fast to calculate
    let metadata_size = output_result.content.len().saturating_sub(total_bytes);
    let metadata_tokens = if metadata_size > 0 {
        // Estimate: metadata is typically XML tags, headers, etc.
        // Use a rough estimate of 1 token per 3 characters for metadata
        metadata_size / 3
    } else {
        0
    };

    let token_count = files_token_count + metadata_tokens;
    tracing::info!(
        "Generated output: {} tokens (encoding: {}, {} from files + {} from metadata)",
        token_count,
        config.token_count.encoding,
        files_token_count,
        metadata_tokens
    );

    let output_total_chars = output_result.content.chars().count();

    Ok(PackResult {
        output: output_result.content,
        token_count,
        total_files: files.len(),
        total_chars: output_total_chars,
        top_files,
        has_secrets: !suspicious_files.is_empty(),
        suspicious_files,
    })
}
