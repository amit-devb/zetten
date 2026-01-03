use std::process::{Command, Stdio};
use std::time::{Instant, Duration};
use std::io::Read;
use std::path::Path;
use anyhow::{Result, anyhow};
use crate::PROCESS_REGISTRY;

#[derive(Default, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub is_success: bool,
    pub duration: Duration,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn execute_task_command(
    cmd_str: &str, 
    allow_exit_codes: &[i32], 
    is_parallel: bool
) -> Result<ExecutionResult> {
    let start = Instant::now();

    // --- AUTO-VENV LOGIC ---
    // Prepend virtual env bins to PATH so the shell finds them first
    let mut path_env = std::env::var_os("PATH").unwrap_or_default();
    let venv_paths = if cfg!(target_os = "windows") {
        vec![".venv\\Scripts", "venv\\Scripts"]
    } else {
        vec![".venv/bin", "venv/bin"]
    };

    for venv_path in venv_paths {
        if Path::new(venv_path).exists() {
            let mut new_path = std::ffi::OsString::from(venv_path);
            new_path.push(if cfg!(target_os = "windows") { ";" } else { ":" });
            new_path.push(&path_env);
            path_env = new_path;
            break; // Use the first one found
        }
    }

    // --- COMMAND SETUP ---
    let mut command = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", cmd_str]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", cmd_str]);
        c
    };

    command.env("PATH", path_env);

    // PILLAR 3: Output Handling
    // In parallel mode, we pipe so we can buffer logs. 
    // In serial mode, we inherit for real-time interaction.
    if is_parallel {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
    } else {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    // --- PILLAR 2: SPAWN & REGISTER ---
    let child = command.spawn()
        .map_err(|e| anyhow!("Failed to spawn command: {}", e))?;

    let child_id = child.id();

    // Register the child handle immediately
    {
        let mut registry = PROCESS_REGISTRY.lock().unwrap();
        registry.push(child);
    }

    // --- MONITORING LOOP ---
    let (status, stdout_final, stderr_final) = loop {
        std::thread::sleep(Duration::from_millis(50));
        let mut registry = PROCESS_REGISTRY.lock().unwrap();
        
        if let Some(pos) = registry.iter_mut().position(|c| c.id() == child_id) {
            match registry[pos].try_wait()? {
                Some(status) => {
                    // Task finished - pull from registry to collect final output
                    let mut finished_child = registry.remove(pos);
                    let mut stdout_buf = Vec::new();
                    let mut stderr_buf = Vec::new();
                    
                    if is_parallel {
                        if let Some(mut out) = finished_child.stdout.take() {
                            let _ = out.read_to_end(&mut stdout_buf);
                        }
                        if let Some(mut err) = finished_child.stderr.take() {
                            let _ = err.read_to_end(&mut stderr_buf);
                        }
                    }
                    
                    break (status, stdout_buf, stderr_buf);
                }
                None => {
                    // Still running, drop lock to allow other threads to access registry
                    drop(registry);
                }
            }
        } else {
            // If the ID is gone, the Ctrl+C handler killed the process and cleared the registry
            return Ok(ExecutionResult {
                exit_code: 130,
                is_success: false,
                duration: start.elapsed(),
                ..Default::default()
            });
        }
    };

    let exit_code = status.code().unwrap_or(0);
    let is_success = exit_code == 0 || allow_exit_codes.contains(&exit_code);

    Ok(ExecutionResult {
        exit_code,
        is_success,
        duration: start.elapsed(),
        stdout: stdout_final,
        stderr: stderr_final,
    })
}