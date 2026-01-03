use crate::root::ConfigSource;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;

lazy_static! {
    static ref RE_VAR_DEFAULT: Regex = Regex::new(r"\$\{([^:]+):-(.*)\}").unwrap();
    static ref RE_VAR_PLAIN: Regex = Regex::new(r"\$\{([^}]+)\}").unwrap();
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub tasks: HashMap<String, TaskConfig>,
    #[serde(default)]
    pub vars: HashMap<String, String>, // Global variables from TOML
}

#[derive(Deserialize, Clone)]
pub struct TaskConfig {
    pub cmd: String,
    #[serde(default = "default_description")]
    pub description: String,
    pub hint: Option<String>,

    #[serde(default)]
    pub inputs: Vec<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub allow_exit_codes: Vec<i32>,

    #[serde(default)]
    pub depends_on: Vec<String>,

    #[serde(default)]
    pub ignore_errors: bool,
}

fn default_description() -> String {
    "No description provided.".to_string()
}

impl TaskConfig {
    /// Resolves the command string using the hierarchy: CLI > TOML > ENV
    pub fn resolve_cmd(&self, extra_args: &[String], all_vars: &HashMap<String, String>) -> String {
        let mut resolved = self.cmd.clone();

        // 1. Resolve ${VAR:-default}
        resolved = RE_VAR_DEFAULT
            .replace_all(&resolved, |caps: &regex::Captures| {
                let var_name = &caps[1];
                let default_val = &caps[2];
                all_vars.get(var_name).cloned().unwrap_or_else(|| default_val.to_string())
            })
            .to_string();

        // 2. Resolve plain ${VAR}
        resolved = RE_VAR_PLAIN
            .replace_all(&resolved, |caps: &regex::Captures| {
                let var_name = &caps[1];
                all_vars.get(var_name).cloned().unwrap_or_else(|| format!("${{{}}}", var_name))
            })
            .to_string();

        // 3. Append forwarded positional arguments
        let positional_args: Vec<String> = extra_args
            .iter()
            .filter(|a| !a.contains('='))
            .cloned()
            .collect();

        if !positional_args.is_empty() {
            resolved.push(' ');
            resolved.push_str(&positional_args.join(" "));
        }

        resolved
    }
}

impl Config {
    pub fn load(source: &ConfigSource) -> Result<Self> {
        // 1. Start with Global Config (Base Layer) from ~/.config/zetten.toml
        let mut final_config = if let Some(global_path) = crate::root::get_global_config_path() {
            if let Ok(contents) = fs::read_to_string(global_path) {
                toml::from_str::<Config>(&contents).unwrap_or(Config {
                    tasks: HashMap::new(),
                    vars: HashMap::new(),
                })
            } else {
                Config { tasks: HashMap::new(), vars: HashMap::new() }
            }
        } else {
            Config { tasks: HashMap::new(), vars: HashMap::new() }
        };

        // 2. Load Local Config (Priority Layer)
        let local_config: Config = match source {
            ConfigSource::PyProjectToml(path) => {
                let contents = fs::read_to_string(path)?;
                let value: toml::Value = toml::from_str(&contents)?;
                let zetten = value.get("tool").and_then(|t| t.get("zetten")).ok_or_else(|| {
                    anyhow!("USER_ERROR: Missing [tool.zetten] section in pyproject.toml")
                })?;
                zetten.clone().try_into()?
            }
            ConfigSource::ZettenToml(path) => {
                let contents = fs::read_to_string(path)?;
                toml::from_str(&contents)?
            }
        };

        // 3. Merge: Local vars and tasks OVERWRITE global ones
        for (key, value) in local_config.vars {
            final_config.vars.insert(key, value);
        }
        for (name, task) in local_config.tasks {
            final_config.tasks.insert(name, task);
        }

        Ok(final_config)
    }

    pub fn validate(&self) -> Result<()> {
        for name in self.tasks.keys() {
            let task = &self.tasks[name];
            for dep in &task.depends_on {
                if !self.tasks.contains_key(dep) {
                    return Err(anyhow!("USER_ERROR: Task '{}' depends on unknown task '{}'", name, dep));
                }
            }
            self.check_cycles(name, &mut HashSet::new())?;
        }
        Ok(())
    }

    fn check_cycles(&self, name: &str, visited: &mut HashSet<String>) -> Result<()> {
        if visited.contains(name) {
            return Err(anyhow!("USER_ERROR: Circular dependency detected at task '{}'", name));
        }
        visited.insert(name.to_string());
        if let Some(task) = self.tasks.get(name) {
            for dep in &task.depends_on {
                self.check_cycles(dep, visited)?;
            }
        }
        visited.remove(name);
        Ok(())
    }
}