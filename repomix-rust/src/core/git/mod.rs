use anyhow::{Context, Result};
use std::process::Command;

pub fn get_git_diff(dir: &std::path::Path) -> Result<String> {
    let output = Command::new("git")
        .arg("diff")
        .current_dir(dir)
        .output()
        .context("Failed to execute git diff")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("git diff failed"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn get_git_log(dir: &std::path::Path, max_commits: usize) -> Result<String> {
    let output = Command::new("git")
        .arg("log")
        .arg(format!("-n{}", max_commits))
        .current_dir(dir)
        .output()
        .context("Failed to execute git log")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("git log failed"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
