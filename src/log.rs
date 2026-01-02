use colored::*;

pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

pub fn user_error(msg: &str) {
    println!("{} {}", "✗".red().bold(), msg.red());
}

pub fn task_ok(name: &str, cached: bool) {
    let status = if cached {
        "cached".dimmed()
    } else {
        "ok".green()
    };
    println!("{} {} ({})", "✔".green().bold(), name, status);
}

pub fn task_fail(name: &str, code: i32) {
    println!("{} {} (exit code {})", "✗".red().bold(), name, code);
}

pub fn suggestion(task: &str, msg: &str) {
    println!(
        "\n{} {} '{}' suggested fix:",
        "💡".yellow(),
        "Tip for".blue().bold(),
        task.yellow()
    );
    println!("   {}\n", msg.cyan().italic());
}

pub fn did_you_mean(target: &str, suggestion: &str) {
    println!(
        "\n{} '{}' is not a recognized command.",
        "❓ Unknown:".yellow().bold(),
        target.red()
    );
    println!(
        "{} Did you mean '{}'?\n",
        "💡 Suggestion:".cyan().bold(),
        suggestion.green().bold()
    );
}