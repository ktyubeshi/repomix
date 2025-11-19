use ignore::{WalkBuilder, WalkState};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs;
use crate::config::RepomixConfig;
use anyhow::{Result, Context};

pub struct FileWalker {
    config: Arc<RepomixConfig>,
}

impl FileWalker {
    pub fn new(config: RepomixConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    pub fn walk<F>(&self, paths: &[PathBuf], mut callback: F) -> Result<()>
    where
        F: FnMut(PathBuf) -> Result<()>,
    {
        let mut builder = WalkBuilder::new(&paths[0]);
        
        for path in paths.iter().skip(1) {
            builder.add(path);
        }

        builder
            .git_ignore(self.config.ignore.use_gitignore)
            .git_global(self.config.ignore.use_gitignore)
            .git_exclude(self.config.ignore.use_gitignore)
            .hidden(!self.config.output.include_empty_directories);

        if !self.config.ignore.custom_patterns.is_empty() {
            let mut overrides = ignore::overrides::OverrideBuilder::new(".");
            for pattern in &self.config.ignore.custom_patterns {
                overrides.add(pattern)?;
            }
            builder.overrides(overrides.build()?);
        }

        let walker = builder.build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        callback(entry.path().to_path_buf())?;
                    }
                }
                Err(err) => {
                    tracing::warn!("Error walking path: {}", err);
                }
            }
        }

        Ok(())
    }
}

pub fn read_file(path: &Path, config: &RepomixConfig) -> Result<Option<String>> {
    let metadata = fs::metadata(path).with_context(|| format!("Failed to get metadata for {:?}", path))?;
    
    if metadata.len() > config.input.max_file_size {
        tracing::debug!("Skipping file {:?} (size: {} > max: {})", path, metadata.len(), config.input.max_file_size);
        return Ok(None);
    }

    // Try to read as UTF-8 string
    // TODO: Handle binary files more gracefully (e.g. using content_inspector)
    match fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            tracing::debug!("Skipping binary file {:?}", path);
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}
