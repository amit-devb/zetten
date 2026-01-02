use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "zetten",
    version,
    about = "Fast, Python-aware task runner",
    long_about = "Zetten is a Rust-based task runner for Python backend projects with deterministic caching, parallel execution, and DAG-based scheduling."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Initialize a new Zetten project
    Init {
        /// Name of the template to use
        template: Option<String>,
    },

    /// Run tasks in parallel with caching
    Run {
        /// Names of the tasks to run
        tasks: Vec<String>,

        /// Number of parallel workers
        #[arg(short, long, default_value = "auto")]
        workers: String,

        /// Preview the execution plan without running commands
        #[arg(long)]
        dry_run: bool,

        /// Pass additional arguments to the task command (e.g. zetten run test -- -k login)
        #[arg(last = true)]
        args: Vec<String>,

        /// Filter tasks by a specific tag (e.g., --tag ci)
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// List all available tasks
    Tasks,

    /// Watch for changes and re-run tasks
    Watch { tasks: Vec<String> },

    /// Check project health
    Doctor,

    /// Visualize the task dependency graph
    Graph,

    /// Generate shell completions
    Completions { shell: Shell },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}
