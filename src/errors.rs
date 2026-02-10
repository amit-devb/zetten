use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum ZettenError {
    #[error("Configuration file not found")]
    #[diagnostic(
        code(ztn::config::missing),
        help("Run `ztn init` to create a new project configuration.")
    )]
    ConfigMissing,

    #[error("Task '{0}' not found")]
    #[diagnostic(
        code(ztn::task::not_found),
        help("Check your configuration file for typos or run `ztn tasks` to see available tasks.")
    )]
    TaskNotFound(String),

    #[error("Task '{0}' not found. Did you mean '{1}'?")]
    #[diagnostic(
        code(ztn::task::typo),
        help("You consistently typed '{0}', but the closest match is '{1}'.")
    )]
    TaskNotFoundFuzzy(String, String),

    #[error("Circular dependency detected")]
    #[diagnostic(
        code(ztn::graph::cycle),
        help("The following tasks form a cycle: {0}. Review 'depends_on' fields to break the loop.")
    )]
    CircularDependency(String),

    #[error("Task execution failed")]
    #[diagnostic(
        code(ztn::exec::failed),
        help("The command exited with code {1}. See output above for details.")
    )]
    TaskFailed(String, i32),
    
    #[error("Project already initialized")]
    #[diagnostic(
        code(ztn::init::exists),
        help("A configuration file already exists. Delete it if you want to re-initialize.")
    )]
    AlreadyInitialized,

    #[error(transparent)]
    #[diagnostic(code(ztn::system::io))]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(code(ztn::system::other))]
    Anyhow(#[from] anyhow::Error),
}
