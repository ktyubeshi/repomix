use anyhow::{Context, Result};
use std::process::Command;

pub struct GitCommit {
    pub date: String,
    pub message: String,
    pub files: Vec<String>,
}

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

pub fn get_git_log(dir: &std::path::Path, max_commits: usize) -> Result<Vec<GitCommit>> {
    let output = Command::new("git")
        .args([
            "log",
            "--pretty=format:%ad|||%s",
            "--date=iso",
            "--name-only",
            &format!("-n{}", max_commits),
        ])
        .current_dir(dir)
        .output()
        .context("Failed to execute git log")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("git log failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();
    let mut current_commit: Option<GitCommit> = None;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.contains("|||") {
            if let Some(commit) = current_commit.take() {
                commits.push(commit);
            }

            let parts: Vec<&str> = line.split("|||").collect();
            if parts.len() >= 2 {
                current_commit = Some(GitCommit {
                    date: parts[0].to_string(),
                    message: parts[1..].join("|||"), // Rejoin if message contained separator (unlikely but safe)
                    files: Vec::new(),
                });
            }
        } else if let Some(commit) = &mut current_commit {
            commit.files.push(line.to_string());
        }
    }

    if let Some(commit) = current_commit {
        commits.push(commit);
    }

    Ok(commits)
}
