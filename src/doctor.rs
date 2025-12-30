use anyhow::Result;
use std::fs;

use crate::config::Config;
use crate::root;
use crate::venv;

pub fn run() -> Result<()> {
    println!("Zetten Doctor\n");

    // 1. Config detection
    let (root, source) = match root::find_project_root() {
        Ok(v) => v,
        Err(_) => {
            println!("✘ Zetten is not initialized");
            println!("  → Run: zetten init");
            return Ok(());
        }
    };

    crate::log::info("✔ Zetten configuration found");
    std::env::set_current_dir(&root)?;

    // 2. Load config
    match Config::load(&source) {
        Ok(cfg) => {
            println!("✔ Configuration loaded ({} tasks)", cfg.tasks.len());
        }
        Err(e) => {
            println!("✘ Failed to load configuration");
            println!("  → {}", e);
            return Ok(());
        }
    }

    // 3. Cache directory
    if fs::create_dir_all(".zetten/cache").is_ok() {
        println!("✔ Cache directory writable (.zetten/cache)");
    } else {
        println!("✘ Cache directory not writable");
    }

    // 4. Python / venv detection
    match venv::detect_venv_bin() {
        Ok(Some(info)) => {
            println!("✔ Python environment detected ({})", info);
        }
        Ok(None) => {
            println!("✘ No Python environment detected");
        }
        Err(e) => {
            println!("✘ Failed to detect Python environment");
            println!("  → {}", e);
        }
    }

    println!("\nDoctor finished.");

    Ok(())
}
