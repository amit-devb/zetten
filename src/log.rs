use std::time::{SystemTime, UNIX_EPOCH};

use crate::runner::ExecutionResult;

pub fn log(task: &str, msg: &str) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let thread_id = std::thread::current().id();

    println!("[{}][{:?}][{}] {}", now, thread_id, task, msg);
}

/// Print buffered output for a task
pub fn print_task_output(task: &str, result: &ExecutionResult) {
    println!("────────── task: {} ──────────", task);

    if !result.stdout.is_empty() {
        println!("[stdout]");
        print!("{}", String::from_utf8_lossy(&result.stdout));
    }

    if !result.stderr.is_empty() {
        println!("[stderr]");
        print!("{}", String::from_utf8_lossy(&result.stderr));
    }

    println!(
        "────────── end: {} (exit={}) ──────────",
        task, result.exit_code
    );
}
