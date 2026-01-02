use anyhow::{Context, Result};
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use std::fs;
use sha2::{Sha256, Digest};

pub fn compute_hash(patterns: &[String]) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut builder = GlobSetBuilder::new();

    for pat in patterns {
        builder.add(Glob::new(pat)?);
    }
    let glob_set = builder.build()?;

    // Walk the project directory, respecting .gitignore but finding matches
    let walker = WalkBuilder::new("./")
        .hidden(false) 
        .build();

    for result in walker {
        let entry = result?;
        let path = entry.path();

        if path.is_file() {
            let relative_path = path.strip_prefix("./").unwrap_or(path);
            
            if glob_set.is_match(relative_path) {
                let content = fs::read(path)
                    .with_context(|| format!("Failed to read file: {:?}", path))?;
                // Hash both the path and the content to detect renames/moves
                hasher.update(relative_path.to_string_lossy().as_bytes());
                hasher.update(&content);
            }
        }
    }

    Ok(format!("{:x}", hasher.finalize()))
}