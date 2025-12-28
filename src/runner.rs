use anyhow::{anyhow, Result};
use std::process::Command;

use crate::venv::build_env_with_venv;

/// Result of executing a task command
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn run_command(cmd: &str, allow_exit_codes: &[i32]) -> Result<ExecutionResult> {
    // Split command string
    let mut parts = cmd.split_whitespace();
    let program = parts.next().ok_or_else(|| anyhow!("Empty command"))?;
    let args: Vec<&str> = parts.collect();

    // Build environment (with venv PATH if present)
    let envs = build_env_with_venv()?;

    // Run command AND CAPTURE output
    let output = Command::new(program).args(args).envs(envs).output()?; // 👈 key change

    let exit_code = output.status.code().unwrap_or(1);

    // Validate exit code
    let allowed = if allow_exit_codes.is_empty() {
        exit_code == 0
    } else {
        allow_exit_codes.contains(&exit_code)
    };

    if !allowed {
        return Err(anyhow!("Command failed with exit code {}", exit_code));
    }

    Ok(ExecutionResult {
        exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}
