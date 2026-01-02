use anyhow::{bail, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use crate::templates::{self, TEMPLATES};

fn detect_env() -> Vec<&'static str> {
    let mut env = Vec::new();
    if Path::new(".venv").exists() { env.push("venv"); }
    if Path::new("requirements.txt").exists() { env.push("pip"); }
    if Path::new("pyproject.toml").exists() { env.push("pyproject"); }
    env
}

fn detect_template() -> &'static str {
    if Path::new("manage.py").exists() { return "django"; }
    if let Ok(content) = fs::read_to_string("pyproject.toml") {
        let c = content.to_lowercase();
        if c.contains("fastapi") { return "fastapi"; }
        if c.contains("flask") { return "flask"; }
    }
    "python"
}

pub fn init(template: &str) -> Result<()> {
    if Path::new("zetten.toml").exists() {
        bail!("Zetten is already initialized (zetten.toml exists).");
    }

    // Determine template name
    let chosen_name = if template == "auto" {
        let detected = detect_template();
        println!("🚀 Auto-detected: {}", detected);
        detected.to_string()
    } else if template == "interactive" {
        crate::tui::select_template()?
    } else {
        template.to_string()
    };

    // Find content
    let content = TEMPLATES.iter()
        .find(|t| t.name == chosen_name)
        .map(|t| t.content)
        .unwrap_or(templates::DEFAULT);

    let py_path = Path::new("pyproject.toml");
    if py_path.exists() {
        let existing = fs::read_to_string(py_path)?;
        if existing.contains("[tool.zetten]") {
            bail!("Zetten is already initialized in pyproject.toml");
        }

        let formatted = templates::format_for_pyproject(content.trim_start());
        let mut file = fs::OpenOptions::new().append(true).open(py_path)?;
        writeln!(file, "\n{}", formatted)?;
        println!("✔ Updated pyproject.toml");
    } else {
        fs::write("zetten.toml", content.trim_start())?;
        println!("✔ Created zetten.toml");
    }

    let env = detect_env();
    if !env.is_empty() { println!("  Detected : {}", env.join(", ")); }
    Ok(())
}