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
    is_parallel: bool,
    interactive: bool,
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
    if is_parallel && !interactive {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
    } else {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        if interactive {    
            command.stdin(Stdio::inherit());
        }
    }

    // --- PILLAR 2: SPAWN & REGISTER ---
    let mut child = command.spawn()
        .map_err(|e| anyhow!("Failed to spawn command: {}", e))?;

    let child_id = child.id();

    // --- NEW: OUTPUT CONSUMPTION THREADS ---
    // We must consume the pipes in background threads to avoid deadlocks
    // if the output exceeds the OS pipe buffer size.
    let (stdout_thread, stderr_thread) = if is_parallel && !interactive {
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let t_out = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut out) = stdout {
                let _ = out.read_to_end(&mut buf);
            }
            buf
        });

        let t_err = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut err) = stderr {
                let _ = err.read_to_end(&mut buf);
            }
            buf
        });
        (Some(t_out), Some(t_err))
    } else {
        (None, None)
    };

    // Register the child handle immediately
    {
        let mut registry = PROCESS_REGISTRY.lock().unwrap();
        registry.push(child);
    }

    // --- MONITORING LOOP ---
    let status = loop {
        std::thread::sleep(Duration::from_millis(50));
        let mut registry = PROCESS_REGISTRY.lock().unwrap();
        
        if let Some(pos) = registry.iter_mut().position(|c| c.id() == child_id) {
            match registry[pos].try_wait()? {
                Some(status) => {
                    // Task finished - remove from registry
                    let _ = registry.remove(pos);
                    break Some(status);
                }
                None => {
                    // Still running, drop lock to allow other threads to access registry
                    drop(registry);
                }
            }
        } else {
            // If the ID is gone, the Ctrl+C handler killed the process and cleared the registry
            break None;
        }
    };

    let (exit_code, stdout_final, stderr_final) = match status {
        Some(s) => {
            let stdout = if let Some(t) = stdout_thread {
                t.join().unwrap_or_default()
            } else {
                Vec::new()
            };
            let stderr = if let Some(t) = stderr_thread {
                t.join().unwrap_or_default()
            } else {
                Vec::new()
            };
            (s.code().unwrap_or(0), stdout, stderr)
        },
        None => (130, Vec::new(), Vec::new()),
    };

    let is_success = exit_code == 0 || allow_exit_codes.contains(&exit_code);

    Ok(ExecutionResult {
        exit_code,
        is_success,
        duration: start.elapsed(),
        stdout: stdout_final,
        stderr: stderr_final,
    })
}