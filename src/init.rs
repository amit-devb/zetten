use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

use crate::templates;

/// Detect environment / tooling
fn detect_env() -> Vec<&'static str> {
    let mut env = Vec::new();

    if Path::new(".venv").exists() {
        env.push("venv");
    }
    if Path::new("requirements.txt").exists() {
        env.push("pip");
    }
    if Path::new("pyproject.toml").exists() {
        env.push("pyproject");
    }

    env
}

/// Auto-detect project template
fn detect_template() -> &'static str {
    if Path::new("manage.py").exists() {
        return "django";
    }

    if let Ok(content) = fs::read_to_string("pyproject.toml") {
        if content.contains("fastapi") {
            return "fastapi";
        }
        if content.contains("flask") {
            return "flask";
        }
    }

    "python"
}

pub fn init(template: &str) -> Result<()> {
    if Path::new("zetten.toml").exists() {
        bail!("Zetten is already initialized.\n\nFound existing configuration:\n- zetten.toml");
    }

    let chosen = if template == "auto" {
        detect_template()
    } else {
        template
    };

    let content = match chosen {
        "python" => templates::PYTHON,
        "fastapi" => templates::FASTAPI,
        "flask" => templates::FLASK,
        "django" => templates::DJANGO,
        _ => bail!("Unknown template: {}", template),
    };

    fs::write("zetten.toml", content.trim_start())?;

    let env = detect_env();

    println!("✔ Created zetten.toml");
    println!("  Template : {}", chosen);

    if !env.is_empty() {
        println!("  Detected : {}", env.join(", "));
    } else {
        println!("  Detected : (no python env files found)");
    }

    Ok(())
}
