use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};
use colored::Colorize;

/// Defines the source of the Zetten configuration
#[derive(Debug, Clone)]
pub enum ConfigSource {
    ZettenToml(PathBuf),
    PyProjectToml(PathBuf),
}

/// Finds the project root by searching upwards for pyproject.toml or zetten.toml.
/// Priority is given to pyproject.toml as the primary configuration source.
pub fn find_project_root() -> Result<(PathBuf, ConfigSource)> {
    let current_dir = std::env::current_dir()?;
    let mut path = current_dir.as_path();

    loop {
        // 1. Check for pyproject.toml (Priority 1)
        let pyproject = path.join("pyproject.toml");
        if pyproject.exists() {
            // Only use it if it contains [tool.zetten]
            if let Ok(contents) = std::fs::read_to_string(&pyproject) {
                if let Ok(value) = toml::from_str::<toml::Value>(&contents) {
                     if value.get("tool").and_then(|t| t.get("zetten")).is_some() {
                         return Ok((path.to_path_buf(), ConfigSource::PyProjectToml(pyproject)));
                     }
                }
            }
        }

        // 2. Check for zetten.toml (Priority 2)
        let zetten = path.join("zetten.toml");
        if zetten.exists() {
            return Ok((path.to_path_buf(), ConfigSource::ZettenToml(zetten)));
        }

        // Move up the directory tree
        match path.parent() {
            Some(parent) => path = parent,
            None => break,
        }
    }

    Err(anyhow!(
        "USER_ERROR: No Zetten configuration found.\n\
         Run {} to create one, or add [tool.zetten] to your pyproject.toml.",
        "ztn init".bold().green()
    ))
}

/// Resolves the OS-specific global configuration path.
/// - Windows: C:\Users\Name\AppData\Roaming\zetten\zetten\config\zetten.toml
/// - macOS:   /Users/Name/Library/Application Support/com.zetten.zetten/zetten.toml
/// - Linux:   /home/name/.config/zetten/zetten.toml
pub fn get_global_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "zetten", "zetten").and_then(|dirs| {
        let path = dirs.config_dir().join("zetten.toml");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    })
}

/// Utility to check if a specific path is within the project root
pub fn is_path_in_root(target: &Path, root: &Path) -> bool {
    target.starts_with(root)
}