use crate::config::{default_ignore::DEFAULT_IGNORE_PATTERNS, RepomixConfig};
use anyhow::{bail, Context, Result};
mod binary_extensions;
use self::binary_extensions::BINARY_EXTENSIONS;
use content_inspector::{inspect, ContentType};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use lazy_static::lazy_static;
use path_clean::PathClean;
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, IsTerminal};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

const MATCH_ALL_PATTERN: &str = "**/*";

lazy_static! {
    static ref BINARY_EXTENSION_SET: HashSet<String> = {
        let mut set = HashSet::with_capacity(BINARY_EXTENSIONS.len());
        for ext in BINARY_EXTENSIONS {
            set.insert(ext.to_ascii_lowercase());
        }
        set
    };
}

pub struct FileWalker {
    config: Arc<RepomixConfig>,
    custom_ignore: Option<GlobSet>,
    default_ignore: Option<GlobSet>,
    output_path: Option<PathBuf>,
}

impl FileWalker {
    pub fn new(config: RepomixConfig) -> Result<Self> {
        let custom_patterns = config
            .ignore
            .custom_patterns
            .iter()
            .map(|pattern| normalize_ignore_pattern(pattern))
            .collect::<Vec<_>>();
        let custom_ignore = build_globset(&custom_patterns)?;

        let default_ignore = if config.ignore.use_default_patterns {
            let patterns = DEFAULT_IGNORE_PATTERNS
                .iter()
                .map(|pattern| normalize_ignore_pattern(pattern))
                .collect::<Vec<_>>();
            build_globset(&patterns)?
        } else {
            None
        };

        let output_path = config.output.file_path.as_ref().map(|path| {
            if path.is_absolute() {
                path.clone()
            } else {
                config.cwd.join(path)
            }
        });

        Ok(Self {
            config: Arc::new(config),
            custom_ignore,
            default_ignore,
            output_path,
        })
    }

    pub fn walk<F>(&self, paths: &[PathBuf], mut callback: F) -> Result<()>
    where
        F: FnMut(PathBuf, PathBuf) -> Result<()>,
    {
        if paths.is_empty() {
            bail!("No target directories supplied");
        }

        for raw_path in paths {
            let root = dunce::canonicalize(raw_path)
                .with_context(|| format!("Failed to resolve path {:?}", raw_path))?;
            let metadata = fs::metadata(&root)
                .with_context(|| format!("Failed to read metadata for {:?}", root))?;

            if !metadata.is_dir() {
                bail!("Target path is not a directory: {:?}", root);
            }

            self.walk_root(&root, &mut callback)?;
        }

        Ok(())
    }

    fn walk_root<F>(&self, root: &Path, callback: &mut F) -> Result<()>
    where
        F: FnMut(PathBuf, PathBuf) -> Result<()>,
    {
        let include_patterns = self.include_patterns_for_root(root);
        let include_matcher = IncludeMatcher::new(include_patterns)?;
        let builder = self.build_walk_builder(root);
        let walker = builder.build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                        continue;
                    }

                    let path = entry.path();
                    let relative = relative_path_str(path, root);

                    if relative.is_empty() {
                        continue;
                    }

                    if !include_matcher.matches(&relative) {
                        continue;
                    }

                    if self.should_skip(&relative, path) {
                        tracing::trace!("Skipping {:?} (matched ignore)", path);
                        continue;
                    }

                    callback(path.to_path_buf(), PathBuf::from(relative))?;
                }
                Err(err) => {
                    tracing::warn!("Error walking path: {}", err);
                }
            }
        }

        Ok(())
    }

    fn include_patterns_for_root(&self, root: &Path) -> Vec<String> {
        let mut patterns: Vec<String> = self
            .config
            .include
            .iter()
            .map(|pattern| escape_glob_pattern(&to_unix_separators(pattern)))
            .collect();

        for path in &self.config.stdin_file_paths {
            match path.strip_prefix(root) {
                Ok(relative) => {
                    let rel = path_to_unix_string(relative);
                    if !rel.is_empty() {
                        patterns.push(escape_glob_pattern(&rel));
                    }
                }
                Err(_) => {
                    tracing::debug!(
                        "Ignoring stdin path {:?} because it is outside root {:?}",
                        path,
                        root
                    );
                }
            }
        }

        if patterns.is_empty() {
            patterns.push(MATCH_ALL_PATTERN.to_string());
        }

        patterns
    }

    fn build_walk_builder(&self, root: &Path) -> WalkBuilder {
        let mut builder = WalkBuilder::new(root);
        builder
            .git_ignore(self.config.ignore.use_gitignore)
            .git_global(self.config.ignore.use_gitignore)
            .git_exclude(self.config.ignore.use_gitignore)
            .ignore(self.config.ignore.use_dot_ignore)
            .hidden(false);
        builder.add_custom_ignore_filename(".repomixignore");
        builder
    }

    fn should_skip(&self, relative: &str, absolute: &Path) -> bool {
        if let Some(output_path) = &self.output_path {
            if absolute == output_path {
                return true;
            }
        }

        if let Some(custom) = &self.custom_ignore {
            if custom.is_match(relative) {
                return true;
            }
        }

        if let Some(default) = &self.default_ignore {
            if default.is_match(relative) {
                return true;
            }
        }

        false
    }
}

struct IncludeMatcher {
    matcher: GlobSet,
}

impl IncludeMatcher {
    fn new(patterns: Vec<String>) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            builder.add(Glob::new(&pattern)?);
        }
        let matcher = builder.build()?;
        Ok(Self { matcher })
    }

    fn matches(&self, path: &str) -> bool {
        self.matcher.is_match(path)
    }
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }

    Ok(Some(builder.build()?))
}

fn normalize_ignore_pattern(pattern: &str) -> String {
    let unix = to_unix_separators(pattern);

    if unix.ends_with('/') && !unix.ends_with("**/") {
        return unix.trim_end_matches('/').to_string();
    }

    // Only append /** if it doesn't look like a file pattern (has no dot)
    // This is a heuristic to avoid breaking **/*.xml -> **/*.xml/**
    if unix.starts_with("**/") && !unix.contains("/**") && !unix.contains('.') {
        return format!("{unix}/**");
    }

    unix
}

fn escape_glob_pattern(pattern: &str) -> String {
    let mut escaped = String::with_capacity(pattern.len());
    for ch in pattern.chars() {
        match ch {
            '(' | ')' | '[' | ']' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            '\\' => {
                escaped.push_str("\\\\");
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn to_unix_separators(value: &str) -> String {
    value.replace('\\', "/")
}

fn relative_path_str(path: &Path, root: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(relative) => path_to_unix_string(relative),
        Err(_) => String::new(),
    }
}

fn path_to_unix_string(path: &Path) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(os) => parts.push(os.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => parts.push("..".to_string()),
            Component::RootDir | Component::Prefix(_) => {}
        }
    }
    parts.join("/")
}

fn is_probably_binary(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| BINARY_EXTENSION_SET.contains(&ext.to_ascii_lowercase()))
        .unwrap_or(false)
}

pub fn read_file(path: &Path, config: &RepomixConfig) -> Result<Option<String>> {
    let metadata =
        fs::metadata(path).with_context(|| format!("Failed to get metadata for {:?}", path))?;

    if metadata.len() > config.input.max_file_size {
        tracing::debug!(
            "Skipping file {:?} (size: {} > max: {})",
            path,
            metadata.len(),
            config.input.max_file_size
        );
        return Ok(None);
    }

    if is_probably_binary(path) {
        tracing::debug!("Skipping binary file {:?} (extension match)", path);
        return Ok(None);
    }

    let bytes = fs::read(path).with_context(|| format!("Failed to read file {:?}", path))?;

    if inspect(&bytes) == ContentType::BINARY {
        tracing::debug!("Skipping binary file {:?} (content detection)", path);
        return Ok(None);
    }

    match String::from_utf8(bytes) {
        Ok(content) => Ok(Some(content)),
        Err(err) => {
            tracing::debug!("Skipping binary file {:?} (utf8 error: {})", path, err);
            Ok(None)
        }
    }
}

pub fn read_stdin_file_paths(cwd: &Path) -> Result<Vec<PathBuf>> {
    if io::stdin().is_terminal() {
        bail!("No data provided via stdin. Please pipe file paths when using --stdin.");
    }

    let stdin = io::stdin();
    let mut filtered_lines = Vec::new();

    for line in stdin.lock().lines() {
        let line = line.with_context(|| "Failed to read line from stdin")?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        filtered_lines.push(trimmed.to_owned());
    }

    if filtered_lines.is_empty() {
        bail!("No valid file paths found in stdin input.");
    }

    let mut seen = HashSet::new();
    let mut resolved_paths = Vec::new();

    for entry in filtered_lines {
        let mut full_path = if Path::new(&entry).is_absolute() {
            PathBuf::from(&entry)
        } else {
            cwd.join(&entry)
        };

        full_path = full_path.clean();

        if seen.insert(full_path.clone()) {
            tracing::trace!("Resolved stdin path: {:?}", full_path);
            resolved_paths.push(full_path);
        }
    }

    Ok(resolved_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn default_ignore_matches_ts_source() {
        let ts_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/config/defaultIgnore.ts");
        let content = fs::read_to_string(&ts_path)
            .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", ts_path, e));

        let mut ts_patterns = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            if let Some(start) = trimmed.find('\'') {
                let rest = &trimmed[start + 1..];
                if let Some(end) = rest.find('\'') {
                    ts_patterns.push(rest[..end].to_string());
                }
            }
        }

        let ours: Vec<String> = DEFAULT_IGNORE_PATTERNS
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(ts_patterns, ours);
    }

    #[test]
    fn walker_skips_default_patterns() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("node_modules")).unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("node_modules/skip.js"),
            "console.log('skip');",
        )
        .unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();

        let mut config = RepomixConfig::default();
        config.cwd = dir.path().to_path_buf();
        config.stdin_file_paths.clear();

        let walker = FileWalker::new(config).unwrap();
        let files = collect_relative_files(&walker, dir.path());

        assert_eq!(files, vec!["src/main.rs".to_string()]);
    }

    #[test]
    fn custom_ignore_overrides_include() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/keep.txt"), "data").unwrap();

        let mut config = RepomixConfig::default();
        config.cwd = dir.path().to_path_buf();
        config.include = vec!["src/**/*.txt".to_string()];
        config.ignore.custom_patterns = vec!["src/**/*.txt".to_string()];

        let walker = FileWalker::new(config).unwrap();
        let files = collect_relative_files(&walker, dir.path());

        assert!(files.is_empty());
    }

    #[test]
    fn stdin_paths_still_respect_ignores() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("logs")).unwrap();
        let log_path = dir.path().join("logs/app.log");
        fs::write(&log_path, "warn").unwrap();

        let mut config = RepomixConfig::default();
        config.cwd = dir.path().to_path_buf();
        config.stdin_file_paths = vec![log_path];

        let walker = FileWalker::new(config).unwrap();
        let files = collect_relative_files(&walker, dir.path());

        assert!(files.is_empty());
    }

    fn collect_relative_files(walker: &FileWalker, root: &Path) -> Vec<String> {
        let mut files = Vec::new();
        walker
            .walk(&[root.to_path_buf()], |_absolute, relative| {
                files.push(relative.to_string_lossy().replace('\\', "/"));
                Ok(())
            })
            .unwrap();
        files.sort();
        files
    }
}
