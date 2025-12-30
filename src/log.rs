// use crate::runner::ExecutionResult;

/// Print a user-facing info message
pub fn info(msg: &str) {
    println!("{}", msg);
}

/// Print a success message for a task
pub fn task_ok(task: &str, cached: bool) {
    if cached {
        println!("✓ {} (cached)", task);
    } else {
        println!("✓ {}", task);
    }
}

/// Print a failure message for a task
pub fn task_fail(task: &str, code: i32) {
    eprintln!("✗ {} (exit code {})", task, code);
}

/// Print a user-facing error message
pub fn user_error(msg: &str) {
    eprintln!("Error:\n{}", msg);
}

// Print command output (used later for CI / verbose mode)
// pub fn print_task_output(_task: &str, _result: &ExecutionResult) {
//     // Intentionally empty for now.
//     // Will be used when adding verbose / CI output.
// }
