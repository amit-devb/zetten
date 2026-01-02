use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::root;
use colored::*;

pub fn run() -> Result<()> {
    println!("🩺 {} \n", "Zetten Doctor".bold().blue());

    // 1. Config detection
    let (root_path, source) = match root::find_project_root() {
        Ok(v) => v,
        Err(_) => {
            crate::log::user_error("Zetten is not initialized in this directory.");
            println!("   💡 Suggestion: Run 'zetten init' to create a configuration file.");
            return Ok(());
        }
    };

    crate::log::info("Config found.");
    std::env::set_current_dir(&root_path)?;

    // 2. Load and validate config
    match Config::load(&source) {
        Ok(cfg) => {
            match cfg.validate() {
                Ok(_) => {
                    println!("{} Configuration valid ({} tasks found)", "✔".green(), cfg.tasks.len());
                    
                    // Verify task inputs
                    for (name, task) in &cfg.tasks {
                        for input in &task.inputs {
                            let p = Path::new(input);
                            if p.is_absolute() && !crate::root::is_path_in_root(p, &root_path) {
                                println!("   {} Task '{}' uses external input '{}'", "⚠".yellow(), name, input);
                            }
                        }
                    }
                },
                Err(e) => {
                    crate::log::user_error(&format!("Configuration has errors: {}", e));
                }
            }
        }
        Err(e) => {
            crate::log::user_error("Failed to load configuration.");
            println!("   → {}", e);
            return Ok(());
        }
    }

    // 3. Cache directory
    if fs::create_dir_all(".zetten/cache").is_ok() {
        println!("{} Cache directory writable (.zetten/cache)", "✔".green());
    } else {
        println!("{} Cache directory not writable", "✘".red());
    }

    // 4. Python Virtual Env Detection
    let venv_path = if cfg!(windows) { ".venv/Scripts" } else { ".venv/bin" };
    if Path::new(venv_path).exists() {
        if let Ok(abs) = fs::canonicalize(venv_path) {
            println!("{} Environment: Python .venv detected ({})", "✔".green(), abs.display());
        } else {
            println!("{} Environment: Python .venv detected", "✔".green());
        }
    } else {
        println!("{} Environment: No local .venv found (using system path)", "!".yellow());
        println!("   💡 Tip: Running 'uv venv' or 'python -m venv .venv' can help isolate dependencies.");
    }

    println!("\n{} finished.", "Doctor".blue());

    Ok(())
}