use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::cache::compute_hash;
use crate::config::Config;
use crate::log;
use crate::orchestrator::run_tasks;

/// Entry point for `zetten watch`
pub fn run(config: &Config, tasks: &[String]) -> Result<()> {
    log::info("Watching for changes...\n");

    // 👉 Run once immediately
    run_tasks(tasks)?;

    let inputs = collect_inputs(config, tasks)?;
    let mut last = hash_inputs(&inputs)?;

    loop {
        thread::sleep(Duration::from_millis(500));

        let current = hash_inputs(&inputs)?;
        if current != last {
            log::info("\n[change detected]\n");
            last = current;

            run_tasks(tasks)?;
        }
    }
}

/// Collect all input paths for the given tasks
fn collect_inputs(config: &Config, tasks: &[String]) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for task in tasks {
        let cfg = config.tasks.get(task).unwrap();
        for input in &cfg.inputs {
            paths.push(PathBuf::from(input));
        }
    }

    Ok(paths)
}

/// Hash all watched inputs
fn hash_inputs(inputs: &[PathBuf]) -> Result<HashMap<PathBuf, String>> {
    let mut hashes = HashMap::new();

    for path in inputs {
        if path.exists() {
            let hash = compute_hash(&[path.to_string_lossy().to_string()])?;
            hashes.insert(path.clone(), hash);
        }
    }

    Ok(hashes)
}
