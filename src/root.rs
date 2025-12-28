use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

/// Find the project root by searching for zetten.toml upwards
pub fn find_project_root() -> Result<PathBuf> {
    let mut current = env::current_dir()?;

    loop {
        let candidate = current.join("zetten.toml");
        if candidate.exists() {
            return Ok(current);
        }

        // Stop at filesystem root
        if !current.pop() {
            break;
        }
    }

    Err(anyhow!(
        "Could not find zetten.toml in this directory or any parent"
    ))
}
