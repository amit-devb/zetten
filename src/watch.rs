use crate::config::Config;
use anyhow::Result;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher, Event};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use colored::*;

pub fn run(initial_config: &Config, task_names: &[String]) -> Result<()> {
    let (root_path, source) = crate::root::find_project_root()?;
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())?;
    
    let mut current_config: Config = initial_config.clone();

    // Initial setup
    setup_watcher(&mut watcher, &current_config, task_names)?;

    crate::log::info("Precision Watch active. Waiting for changes...");
    
    // Initial run - Added empty HashMap for Argument #6
    let _ = crate::run_tasks(task_names.to_vec(), "auto".to_string(), false, vec![], None, HashMap::new());

    let mut last_event_time = Instant::now();
    let debounce_duration = Duration::from_millis(300);
    let mut pending_paths: Vec<PathBuf> = Vec::new();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                if is_relevant(&event) {
                    pending_paths.extend(event.paths);
                    last_event_time = Instant::now();
                }
            }
            Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
            Err(_) => {
                if !pending_paths.is_empty() && last_event_time.elapsed() >= debounce_duration {
                    
                    let project_paths: Vec<PathBuf> = pending_paths.iter()
                        .filter(|p| crate::root::is_path_in_root(p, &root_path))
                        .cloned()
                        .collect();

                    if !project_paths.is_empty() {
                        let config_file_changed = project_paths.iter().any(|p| {
                            let s = p.to_string_lossy();
                            s.ends_with("pyproject.toml") || s.ends_with("zetten.toml")
                        });

                        if config_file_changed {
                            println!("\n{}", "⚙️ Configuration change detected. Reloading...".bold().magenta());
                            if let Ok(new_cfg) = Config::load(&source) {
                                current_config = new_cfg;
                                let _ = setup_watcher(&mut watcher, &current_config, task_names);
                                crate::log::info("Config reloaded successfully.");
                            }
                        }

                        let affected = identify_affected(&current_config, task_names, &project_paths);
                        
                        if !affected.is_empty() {
                            println!("\n{}", "🔄 Changes detected. Re-running affected tasks...".bold().cyan());
                            // Re-run call - Added empty HashMap for Argument #6
                            let _ = crate::run_tasks(affected, "auto".to_string(), false, vec![], None, HashMap::new());
                        }
                    }
                    pending_paths.clear();
                }
            }
        }
    }
}

fn setup_watcher(watcher: &mut RecommendedWatcher, config: &Config, task_names: &[String]) -> Result<()> {
    if Path::new("pyproject.toml").exists() { watcher.watch(Path::new("pyproject.toml"), RecursiveMode::NonRecursive)?; }
    if Path::new("zetten.toml").exists() { watcher.watch(Path::new("zetten.toml"), RecursiveMode::NonRecursive)?; }

    for name in task_names {
        if let Some(task) = config.tasks.get(name) {
            for input in &task.inputs {
                let p = Path::new(input);
                if p.exists() { watcher.watch(p, RecursiveMode::Recursive)?; }
            }
        }
    }
    Ok(())
}

fn is_relevant(event: &Event) -> bool {
    event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove()
}

fn identify_affected(config: &Config, roots: &[String], paths: &[PathBuf]) -> Vec<String> {
    let mut affected = Vec::new();
    for name in roots {
        if let Some(task) = config.tasks.get(name) {
            for input in &task.inputs {
                let input_p = Path::new(input);
                if paths.iter().any(|p| p.starts_with(input_p) || input_p == p) {
                    if !affected.contains(name) { affected.push(name.clone()); }
                }
            }
        }
    }
    affected
}