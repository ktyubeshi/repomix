use anyhow::{Context, Result};
use std::process::Command;
use tempfile::TempDir;

pub fn clone_repo(url: &str) -> Result<TempDir> {
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;

    tracing::info!(
        "Cloning {} to temporary directory {:?}",
        url,
        temp_dir.path()
    );

    let output = Command::new("git")
        .arg("clone")
        .arg("--depth=1")
        .arg(url)
        .arg(".")
        .current_dir(temp_dir.path())
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git clone failed: {}", stderr));
    }

    Ok(temp_dir)
}

pub fn is_remote_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("git@")
}
