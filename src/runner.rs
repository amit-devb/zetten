use anyhow::Result;
use std::process::{Command, Stdio};
use std::env;
use std::path::Path;
use std::fs;
use std::io::{self, Write};

#[derive(Debug, Clone, Default)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub is_success: bool,
}

pub fn run_command(cmd_str: &str, allow_exit_codes: &[i32], is_parallel: bool) -> Result<ExecutionResult> {
    let mut actual_cmd = cmd_str.trim();
    
    // 1. Quiet Mode Logic
    let is_quiet = actual_cmd.starts_with('@');
    if is_quiet {
        actual_cmd = actual_cmd[1..].trim();
    } else if !is_parallel {
        // Only print the command string if we aren't in a progress-bar (parallel) mode
        println!("  $ {}", actual_cmd);
    }

    // 2. Shell Configuration
    let shell = if cfg!(windows) { "cmd" } else { "sh" };
    let shell_flag = if cfg!(windows) { "/C" } else { "-c" };
    
    let mut command = Command::new(shell);
    command.arg(shell_flag).arg(actual_cmd);

    // 3. Integrated Auto-Venv Logic (Preserved)
    let venv_path = if cfg!(windows) { ".venv/Scripts" } else { ".venv/bin" };
    if Path::new(venv_path).exists() {
        if let Ok(current_path) = env::var("PATH") {
            if let Ok(abs_venv) = fs::canonicalize(venv_path) {
                let sep = if cfg!(windows) { ";" } else { ":" };
                let new_path = format!("{}{}{}", abs_venv.display(), sep, current_path);
                command.env("PATH", new_path);
                command.env("VIRTUAL_ENV", ".venv");
            }
        }
    }

    // 4. Output Logic (The Visibility Fix)
    if !is_parallel {
        // SINGLE TASK: Stream directly to Ubuntu terminal (shows errors/colors live)
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        
        let status = command.status()?;
        let exit_code = status.code().unwrap_or(1);
        let is_success = exit_code == 0 || allow_exit_codes.contains(&exit_code);
        
        return Ok(ExecutionResult {
            exit_code,
            stdout: Vec::new(), // Not needed for single stream
            stderr: Vec::new(),
            is_success,
        });
    }

    // PARALLEL MODE: Capture to buffer to keep progress bar clean
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let output = command.output()?;
    let exit_code = output.status.code().unwrap_or(1);
    let is_success = exit_code == 0 || allow_exit_codes.contains(&exit_code);

    // If parallel task fails, dump its output immediately so the user sees the error
    if !is_success {
        if !output.stdout.is_empty() {
            let _ = io::stdout().write_all(&output.stdout);
        }
        if !output.stderr.is_empty() {
            let _ = io::stderr().write_all(&output.stderr);
        }
    }

    Ok(ExecutionResult {
        exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
        is_success,
    })
}