use std::process::Command;
use std::path::Path;
use anyhow::{anyhow, Result};
use crate::config::Config;
use globset::Glob;
use walkdir::WalkDir;
use colored::*;

/// Validates the environment, commands, and input files/globs.
pub fn validate_execution_env(config: &Config, task_names: &[String]) -> Result<()> {
    let mut errors = Vec::new();

    for name in task_names {
        let task = config.tasks.get(name)
            .ok_or_else(|| anyhow!("Task {} not found", name))?;

        // 1. COMMAND VALIDATION
        let cmd_primary = task.cmd.split_whitespace().next().unwrap_or("");
        if !command_exists(cmd_primary) {
            let error_msg = format!(
                "{} Task '{}' requires binary '{}', but it was not found in PATH.",
                "✘".red(), name.bold(), cmd_primary.yellow()
            );
            
            if let Some(hint) = &task.hint {
                errors.push(format!("{}\n   {} {}", error_msg, "💡 Tip:".cyan(), hint));
            } else {
                errors.push(error_msg);
            }
        }

        // 2. INPUTS VALIDATION
        for input in &task.inputs {
            if input.contains('*') || input.contains('?') {
                // Compile the glob pattern
                let glob_matcher = Glob::new(input)
                    .map_err(|e| anyhow!("Invalid pattern '{}' in task '{}': {}", input, name, e))?
                    .compile_matcher();

                // Walk and check if AT LEAST ONE file matches the inputs pattern
                let mut found = false;
                for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    
                    // Standardize: strip "./" and check against the pattern
                    let stripped = path.strip_prefix("./").unwrap_or(path);
                    if stripped.as_os_str().is_empty() { continue; }

                    if glob_matcher.is_match(stripped) {
                        found = true;
                        break;
                    }
                }

                if !found {
                    errors.push(format!(
                        "{} Task '{}' defined inputs '{}', but no matching files were found.",
                        "✘".red(), name.bold(), input.yellow()
                    ));
                }
            } else {
                // Static file/folder check
                if !Path::new(input).exists() {
                    errors.push(format!(
                        "{} Task '{}' defined inputs '{}', but this path is missing.",
                        "✘".red(), name.bold(), input.yellow()
                    ));
                }
            }
        }
    }

    // 3. AGGREGATED ERROR REPORTING
    if !errors.is_empty() {
        println!("\n{}", "⚠️  Pre-flight validation failed:".bold().yellow());
        for err in errors {
            println!("  {}", err);
        }
        println!();
        return Err(anyhow!("Environment check failed. Please resolve the issues above."));
    }

    Ok(())
}

fn command_exists(cmd: &str) -> bool {
    if cmd.starts_with("./") || cmd.starts_with("../") {
        return Path::new(cmd).exists();
    }

    let check_cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
    Command::new(check_cmd)
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}