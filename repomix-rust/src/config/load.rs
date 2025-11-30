// repomix-rust/src/config/load.rs
// This file will contain the logic for loading repomix configuration.

use super::global_directory;
use crate::config::schema::RepomixConfig;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command; // Added for subprocess execution
use tracing::{debug, trace, warn}; // Added for Stdio

const DEFAULT_CONFIG_PATHS: [&str; 9] = [
    "repomix.config.ts",
    "repomix.config.mts",
    "repomix.config.cts",
    "repomix.config.js",
    "repomix.config.mjs",
    "repomix.config.cjs",
    "repomix.config.json5",
    "repomix.config.jsonc",
    "repomix.config.json",
];

fn check_file_exists(file_path: &Path) -> bool {
    file_path.is_file()
}

async fn find_config_file(config_paths: &[PathBuf], log_prefix: &str) -> Option<PathBuf> {
    for config_path in config_paths {
        trace!("Checking for {} config at: {:?}", log_prefix, config_path);
        if check_file_exists(config_path) {
            trace!("Found {} config at: {:?}", log_prefix, config_path);
            return Some(config_path.clone());
        }
    }
    None
}

fn get_file_extension(file_path: &Path) -> Option<&str> {
    file_path.extension().and_then(|s| s.to_str())
}

fn is_file_path_explicit(raw_config: &Value) -> bool {
    raw_config
        .get("output")
        .and_then(|output| output.get("filePath"))
        .and_then(|path| path.as_str())
        .map(|path| !path.is_empty())
        .unwrap_or(false)
}

async fn load_js_ts_config_with_subprocess(file_path: PathBuf) -> Result<Value> {
    debug!(
        "Attempting to load JS/TS config file {:?} via subprocess...",
        file_path
    );

    // Get the path to the jiti_config_loader.js script
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let loader_script_path = current_dir.join("repomix-rust/src/config/jiti_config_loader.js");

    if !loader_script_path.is_file() {
        bail!(
            "Jiti config loader script not found at {:?}",
            loader_script_path
        );
    }

    // Execute Node.js as a subprocess
    let command = Command::new("node")
        .arg(&loader_script_path)
        .arg(&file_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn Node.js subprocess. Is Node.js installed and in PATH?")?;

    let output = command
        .wait_with_output()
        .await
        .context("Failed to wait for Node.js subprocess")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Node.js subprocess failed to load config from {:?}:\n{}",
            file_path,
            stderr
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        bail!(
            "Node.js subprocess returned empty output for config file {:?}",
            file_path
        );
    }

    serde_json::from_str(&stdout).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse JSON output from Node.js subprocess for {:?}: {}",
            file_path,
            e
        )
    })
}

async fn load_and_validate_config(file_path: PathBuf) -> Result<RepomixConfig> {
    let ext = get_file_extension(&file_path);

    let raw_config: Value = match ext {
        Some("json") | Some("json5") | Some("jsonc") => {
            let file_content = fs::read_to_string(&file_path).map_err(|e| {
                anyhow::anyhow!("Failed to read config file {:?}: {}", file_path, e)
            })?;
            json5::from_str(&file_content).map_err(|e| {
                anyhow::anyhow!("Failed to parse config file {:?}: {}", file_path, e)
            })?
        }
        Some("ts") | Some("mts") | Some("cts") | Some("js") | Some("mjs") | Some("cjs") => {
            load_js_ts_config_with_subprocess(file_path.clone()).await?
        }
        _ => bail!("Unsupported config file format: {:?}", file_path),
    };

    let file_path_explicit = is_file_path_explicit(&raw_config);

    let mut config: RepomixConfig = serde_json::from_value(raw_config).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse config file {:?} into RepomixConfig: {}",
            file_path,
            e
        )
    })?;

    config.output.file_path_explicit = file_path_explicit;

    // TODO: Implement actual schema validation (similar to Zod). For now, serde will handle basic type checks.
    Ok(config)
}

pub async fn load_file_config(
    root_dir: &Path,
    arg_config_path: Option<&Path>,
) -> Result<RepomixConfig> {
    // If a specific config path is provided, use it directly
    if let Some(arg_path) = arg_config_path {
        let full_path = root_dir.join(arg_path);
        debug!("Loading local config from: {:?}", full_path);

        if check_file_exists(&full_path) {
            return load_and_validate_config(full_path).await;
        }
        bail!("Config file not found at {:?}", arg_path);
    }

    // Try to find a local config file using the priority order
    let local_config_paths: Vec<PathBuf> = DEFAULT_CONFIG_PATHS
        .iter()
        .map(|p| root_dir.join(p))
        .collect();
    if let Some(local_config_path) = find_config_file(&local_config_paths, "local").await {
        return load_and_validate_config(local_config_path).await;
    }

    // Try to find a global config file using the priority order
    let global_dir = match global_directory::get_global_directory() {
        Ok(dir) => dir,
        Err(e) => {
            warn!("Failed to get global directory: {}", e);
            PathBuf::new() // Use empty path if global directory cannot be determined
        }
    };
    let global_config_paths: Vec<PathBuf> = DEFAULT_CONFIG_PATHS
        .iter()
        .map(|p| global_dir.join(p))
        .collect();
    if let Some(global_config_path) = find_config_file(&global_config_paths, "global").await {
        return load_and_validate_config(global_config_path).await;
    }

    debug!("No custom config found. Using default config.");
    Ok(RepomixConfig::default())
}
