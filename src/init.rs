use miette::Result; // Use miette
use std::fs;
use std::io::Write;
use std::path::Path;
use crate::templates::{self, TEMPLATES};
use crate::errors::ZettenError;
use colored::*;

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
        return Err(ZettenError::AlreadyInitialized.into());
    }

    // Determine template name
    let chosen_name = if template == "auto" {
        let detected = detect_template();
        println!("🚀 Auto-detected environment: {}", detected.cyan());
        detected.to_string()
    } else if template == "interactive" {
        crate::tui::select_template().map_err(|e| ZettenError::Anyhow(anyhow::anyhow!(e)))?
    } else {
        template.to_string()
    };

    // Find content
    let content = TEMPLATES.iter()
        .find(|t| t.name == chosen_name)
        .map(|t| t.content)
        .unwrap_or(templates::DEFAULT);

    let py_path = Path::new("pyproject.toml");
    let target_file;
    
    if py_path.exists() {
        let existing = fs::read_to_string(py_path).map_err(ZettenError::IoError)?;
        if existing.contains("[tool.zetten]") {
            return Err(ZettenError::AlreadyInitialized.into());
        }

        let formatted = templates::format_for_pyproject(content.trim_start());
        let mut file = fs::OpenOptions::new().append(true).open(py_path).map_err(ZettenError::IoError)?;
        writeln!(file, "\n{}", formatted).map_err(ZettenError::IoError)?;
        target_file = "pyproject.toml";
    } else {
        fs::write("zetten.toml", content.trim_start()).map_err(ZettenError::IoError)?;
        target_file = "zetten.toml";
    }

    println!("\n{}:", "🎉 Zetten Initialized Successfully".green().bold());
    println!("   Added configuration to {}", target_file.yellow());
    
    println!("\n{}:", "🚀 Try running your first task".bold());
    println!("   {}", "ztn run hello".cyan());
    
    Ok(())
}