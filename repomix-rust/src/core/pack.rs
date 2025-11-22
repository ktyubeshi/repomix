use crate::config::schema::{RepomixConfig, TokenCountTreeConfig};
use crate::core::file::process;
use crate::core::metrics::token_tree::{build_token_tree, TokenTreeNode};
use crate::core::{file, metrics, output, remote, security};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone)]
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
    pub token_count_tree: Option<TokenTreeNode>,
    pub has_secrets: bool,
    pub suspicious_files: Vec<PathBuf>,
}

pub fn pack(config: &RepomixConfig, paths: &[PathBuf]) -> Result<PackResult> {
    // Check for remote repositories and clone if necessary
    let mut _temp_dirs = Vec::new();
    let mut target_paths = Vec::new();

    for path_buf in paths {
        let path_str = path_buf.to_string_lossy();
        if remote::is_remote_url(&path_str) {
            let temp_dir = remote::clone_repo(&path_str, config.remote_branch.as_deref())?;
            target_paths.push(temp_dir.path().to_path_buf());
            _temp_dirs.push(temp_dir);
        } else {
            target_paths.push(path_buf.clone());
        }
    }

    // Collect files
    let walker = file::FileWalker::new(config.clone())?;
    let files = std::sync::Mutex::new(HashMap::new());
    let suspicious_files = std::sync::Mutex::new(Vec::new());
    let total_chars = std::sync::atomic::AtomicUsize::new(0);
    let file_stats = std::sync::Mutex::new(Vec::new());
    let enc = config.token_count.encoding.clone();

    // Use parallel walker for better performance (read -> check -> process -> count tokens)
    walker.walk_parallel(&target_paths, |absolute_path, relative_path| {
        match file::read_file(&absolute_path, config) {
            Ok(Some(content)) => {
                tracing::debug!("Read file: {:?} (len: {})", absolute_path, content.len());

                // Security check
                if config.security.enable_security_check {
                    if let Ok(Some(result)) = security::scan_content(&absolute_path, &content) {
                        for secret in result.secrets {
                            tracing::debug!(
                                "Potential secret found in {:?}: {}",
                                absolute_path,
                                secret
                            );
                        }
                        suspicious_files.lock().unwrap().push(relative_path);
                        return Ok(());
                    }
                }

                let final_content = match process::process_content(&content, &absolute_path, config)
                {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Failed to process {:?}: {}", absolute_path, e);
                        content
                    }
                };

                // Count tokens immediately (parallelized)
                let token_count = metrics::count_tokens(&final_content, &enc).unwrap_or(0);
                let char_count = final_content.chars().count();

                total_chars.fetch_add(char_count, std::sync::atomic::Ordering::Relaxed);

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
    let files_total_chars = total_chars.into_inner();
    let mut file_stats = file_stats.into_inner().unwrap();

    tracing::info!(
        "Found {} files, total characters: {}",
        files.len(),
        files_total_chars
    );

    let mut sorted_paths: Vec<PathBuf> = files.keys().cloned().collect();
    let git_root = if config.cwd.as_os_str().is_empty() {
        std::path::PathBuf::from(".")
    } else {
        config.cwd.clone()
    };
    if config.output.git.sort_by_changes {
        match crate::core::git::get_file_change_counts(
            &git_root,
            config.output.git.sort_by_changes_max_commits as usize,
        ) {
            Ok(counts) => {
                sorted_paths.sort_by(|a, b| {
                    let ca = counts.get(a).copied().unwrap_or(0);
                    let cb = counts.get(b).copied().unwrap_or(0);
                    ca.cmp(&cb).then_with(|| a.cmp(b))
                });
            }
            Err(e) => {
                tracing::warn!("Failed to sort by git change count: {}", e);
                sorted_paths.sort();
            }
        }
    } else {
        sorted_paths.sort();
    }

    let files_token_count: usize = file_stats.iter().map(|f| f.token_count).sum();

    file_stats.sort_by(|a, b| b.token_count.cmp(&a.token_count));
    let top_len = config.output.top_files_length as usize;
    let top_files: Vec<FileStats> = if top_len == 0 {
        Vec::new()
    } else {
        file_stats.iter().take(top_len).cloned().collect()
    };

    let token_tree = match config.output.token_count_tree {
        TokenCountTreeConfig::Bool(false) => None,
        _ => Some(build_token_tree(
            &file_stats
                .iter()
                .map(|f| (f.path.clone(), f.token_count))
                .collect::<Vec<_>>(),
        )),
    };

    // Generate output
    tracing::info!("Generating output file...");
    let mut output_result = output::format(
        config,
        output::FormatContext {
            files: &files,
            sorted_paths: &sorted_paths,
            top_files: &top_files,
            token_count_tree: token_tree.as_ref(),
            token_count: files_token_count,
            total_chars: files_total_chars,
        },
    )?;
    tracing::info!(
        "Output generation completed ({} bytes)",
        output_result.content.len()
    );

    let output_total_chars = output_result.content.chars().count();
    let token_count =
        metrics::count_tokens(&output_result.content, &enc).unwrap_or(files_token_count);

    if matches!(
        config.output.style,
        crate::config::schema::RepomixOutputStyle::Json
    ) {
        output_result = output::format(
            config,
            output::FormatContext {
                files: &files,
                sorted_paths: &sorted_paths,
                top_files: &top_files,
                token_count_tree: token_tree.as_ref(),
                token_count,
                total_chars: output_total_chars,
            },
        )?;
    }

    Ok(PackResult {
        output: output_result.content,
        token_count,
        total_files: files.len(),
        total_chars: output_total_chars,
        top_files,
        token_count_tree: token_tree,
        has_secrets: !suspicious_files.is_empty(),
        suspicious_files,
    })
}
