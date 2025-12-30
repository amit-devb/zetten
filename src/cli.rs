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
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run one or more tasks
    Run {
        /// Names of tasks to run
        tasks: Vec<String>,

        /// Number of worker threads (or \"auto\")
        #[arg(long = "worker", default_value = "auto")]
        workers: String,
    },

    /// Watch tasks and re-run on input changes
    Watch {
        /// Names of tasks to watch
        tasks: Vec<String>,
    },

    /// Show task dependency graph
    Graph,

    /// Check project and environment health
    Doctor,

    /// Generate shell completion scripts
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },

    Init {
        /// Project template (python, fastapi, django)
        #[arg(default_value = "python")]
        template: String,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}
