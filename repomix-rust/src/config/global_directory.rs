// repomix-rust/src/config/global_directory.rs

use anyhow::{bail, Result};
use std::env;
use std::path::PathBuf;

// Mimics Node.js getGlobalDirectory
// Windows: %LOCALAPPDATA%\Repomix
// Others: $XDG_CONFIG_HOME/repomix or ~/.config/repomix
pub fn get_global_directory() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // Try LOCALAPPDATA first, then fall back to home/AppData/Local
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data).join("Repomix"));
        }

        if let Some(home_dir) = dirs::home_dir() {
            return Ok(home_dir.join("AppData").join("Local").join("Repomix"));
        }

        bail!("Could not determine config directory on Windows (LOCALAPPDATA not set and home dir not found)");
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(xdg_config_home).join("repomix"));
        }

        if let Some(home_dir) = dirs::home_dir() {
            return Ok(home_dir.join(".config").join("repomix"));
        }

        bail!(
            "Could not determine config directory (XDG_CONFIG_HOME not set and home dir not found)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_global_directory() {
        let global_dir = get_global_directory().unwrap();

        #[cfg(target_os = "windows")]
        {
            // Basic check for Windows pattern
            assert!(global_dir.ends_with("Repomix"));
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Check for Unix pattern
            assert!(global_dir.ends_with("repomix"));
            // If XDG_CONFIG_HOME is not set (likely in test env), it should be in .config
            if std::env::var("XDG_CONFIG_HOME").is_err() {
                assert!(global_dir.to_string_lossy().contains(".config"));
            }
        }
    }
}
