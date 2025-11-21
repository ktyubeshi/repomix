// repomix-rust/src/config/global_directory.rs

use anyhow::{Result, bail};
use std::path::PathBuf;
use tracing::warn;

// Mimics Node.js getGlobalDirectory
pub fn get_global_directory() -> Result<PathBuf> {
    if let Some(mut home_dir) = dirs::home_dir() {
        home_dir.push(".repomix");
        Ok(home_dir)
    } else {
        bail!("Could not determine home directory to get global config path.");
    }
}

// TODO: Add tests for get_global_directory
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_global_directory() {
        let global_dir = get_global_directory().unwrap();
        // This test depends on the environment, so we can only assert basic properties
        assert!(global_dir.starts_with(dirs::home_dir().unwrap()));
        assert!(global_dir.ends_with(".repomix"));
    }
}
