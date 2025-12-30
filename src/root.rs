use anyhow::{anyhow, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ConfigSource {
    ZettenToml(PathBuf),
    PyProjectToml(PathBuf),
}

/// Find the project root by searching for Zetten configuration upwards
pub fn find_project_root() -> Result<(PathBuf, ConfigSource)> {
    let mut current = env::current_dir()?;

    loop {
        // 1. pyproject.toml with any [tool.zetten.*] section
        let pyproject = current.join("pyproject.toml");
        if pyproject.exists() && pyproject_has_zetten(&pyproject)? {
            return Ok((
                current.clone(),
                ConfigSource::PyProjectToml(pyproject),
            ));
        }

        // 2. zetten.toml
        let zetten = current.join("zetten.toml");
        if zetten.exists() {
            return Ok((current.clone(), ConfigSource::ZettenToml(zetten)));
        }

        // Stop at filesystem root
        if !current.pop() {
            break;
        }
    }

    Err(anyhow!("NO_ZETTEN_CONFIG"))
}

fn pyproject_has_zetten(path: &PathBuf) -> Result<bool> {
    let contents = fs::read_to_string(path)?;
    let value: toml::Value = toml::from_str(&contents)?;

    Ok(
        value
            .get("tool")
            .and_then(|t| t.get("zetten"))
            .is_some(),
    )
}
