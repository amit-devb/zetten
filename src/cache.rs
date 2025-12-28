use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub fn compute_hash(inputs: &[String]) -> Result<String> {
    let mut hasher = Sha256::new();

    for input in inputs {
        let path = Path::new(input);
        hash_path(path, &mut hasher)?;
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn hash_path(path: &Path, hasher: &mut Sha256) -> Result<()> {
    if path.is_file() {
        let bytes = fs::read(path)?;
        hasher.update(bytes);
    } else if path.is_dir() {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<_, _>>()?;

        entries.sort();

        for entry in entries {
            hash_path(&entry, hasher)?;
        }
    }

    Ok(())
}
