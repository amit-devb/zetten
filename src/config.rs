use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub tasks: HashMap<String, TaskConfig>,
}

#[derive(Deserialize, Clone)]
pub struct TaskConfig {
    pub cmd: String,
    pub inputs: Vec<String>,

    #[serde(default)]
    pub allow_exit_codes: Vec<i32>,

    // 👇 REQUIRED FOR DAG
    #[serde(default)]
    pub depends_on: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let contents = fs::read_to_string("zetten.toml")?;
        Ok(toml::from_str(&contents)?)
    }

    /// Validate that all dependencies exist
    pub fn validate(&self) -> Result<()> {
        for (task_name, task) in &self.tasks {
            for dep in &task.depends_on {
                if !self.tasks.contains_key(dep) {
                    return Err(anyhow!(
                        "Task '{}' depends on unknown task '{}'",
                        task_name,
                        dep
                    ));
                }
            }
        }
        Ok(())
    }
}
