use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;

/// Re-enter the existing `zetten run` execution pipeline.
/// This intentionally calls `run_main` to preserve:
/// - DAG scheduling
/// - caching
/// - exit codes
/// - logging
pub fn run_tasks(tasks: &[String]) -> Result<()> {
    let cli = Cli::parse_from(
        std::iter::once("zetten")
            .chain(std::iter::once("run"))
            .chain(tasks.iter().map(String::as_str)),
    );

    crate::run_main(cli)
}
