use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

use crate::root::ConfigSource;

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

    #[serde(default)]
    pub depends_on: Vec<String>,
}

impl Config {
    pub fn load(source: &ConfigSource) -> Result<Self> {
        match source {
            ConfigSource::ZettenToml(path) => {
                let contents = fs::read_to_string(path)
                    .with_context(|| format!("USER_ERROR: Failed to read {}", path.display()))?;

                toml::from_str(&contents)
                    .map_err(|e| anyhow!("USER_ERROR: Invalid zetten.toml:\n{}", e))
            }

            ConfigSource::PyProjectToml(path) => {
                let contents = fs::read_to_string(path)
                    .with_context(|| format!("USER_ERROR: Failed to read {}", path.display()))?;

                let value: toml::Value =
                    toml::from_str(&contents).map_err(|e| {
                        anyhow!("USER_ERROR: Invalid pyproject.toml:\n{}", e)
                    })?;

                let zetten = value
                    .get("tool")
                    .and_then(|t| t.get("zetten"))
                    .ok_or_else(|| {
                        anyhow!(
                            "USER_ERROR: Missing [tool.zetten] section in pyproject.toml"
                        )
                    })?;

                zetten
                    .clone()
                    .try_into()
                    .map_err(|e| anyhow!("USER_ERROR: Invalid [tool.zetten] config:\n{}", e))
            }
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.tasks.is_empty() {
            return Err(anyhow!(
                "USER_ERROR: No tasks defined.\n\nAdd tasks under [tasks] or [tool.zetten.tasks]"
            ));
        }

        for (task_name, task) in &self.tasks {
            if task.cmd.trim().is_empty() {
                return Err(anyhow!(
                    "USER_ERROR: Task '{}' has an empty command",
                    task_name
                ));
            }

            if task.inputs.is_empty() {
                return Err(anyhow!(
                    "USER_ERROR: Task '{}' must define at least one input",
                    task_name
                ));
            }

            for dep in &task.depends_on {
                if !self.tasks.contains_key(dep) {
                    return Err(anyhow!(
                        "USER_ERROR: Task '{}' depends on unknown task '{}'",
                        task_name,
                        dep
                    ));
                }
            }
        }

        Ok(())
    }
}
