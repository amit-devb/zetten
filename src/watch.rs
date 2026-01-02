use crate::config::Config;
use anyhow::Result;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher, Event};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub fn run(config: &Config, task_names: &[String]) -> Result<()> {
    let (root_path, _) = crate::root::find_project_root()?; // Get project root
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())?;
    
    for name in task_names {
        if let Some(task) = config.tasks.get(name) {
            for input in &task.inputs {
                if Path::new(input).exists() {
                    watcher.watch(Path::new(input), RecursiveMode::Recursive)?;
                }
            }
        }
    }

    crate::log::info("Precision Watch active. Waiting for changes...");
    let _ = crate::run_tasks(task_names.to_vec(), "auto".to_string(), false, vec![], None);

    let mut last_run = Instant::now();
    let debounce = Duration::from_millis(500);

    for res in rx {
        if let Ok(event) = res {
            if last_run.elapsed() < debounce || !is_relevant(&event) { continue; }

            // SAFETY: Only process paths inside our project root
            let project_paths: Vec<_> = event.paths.iter()
                .filter(|p| crate::root::is_path_in_root(p, &root_path))
                .cloned()
                .collect();

            if project_paths.is_empty() { continue; }

            let affected = identify_affected(config, task_names, &project_paths);
            if !affected.is_empty() {
                crate::log::info(&format!("Change in {:?}. Re-running affected tasks...", project_paths[0]));
                let _ = crate::run_tasks(affected, "auto".to_string(), false, vec![], None);
                last_run = Instant::now();
            }
        }
    }
    Ok(())
}

fn is_relevant(event: &Event) -> bool {
    event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove()
}

fn identify_affected(config: &Config, roots: &[String], paths: &[std::path::PathBuf]) -> Vec<String> {
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