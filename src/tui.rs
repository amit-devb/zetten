use anyhow::{anyhow, Result};
use inquire::Select;
use crate::config::Config;
use crate::root;
use std::env;
use std::collections::HashMap;
use crate::templates::TEMPLATES;

pub fn select_template() -> Result<String> {
    let options: Vec<String> = TEMPLATES
        .iter()
        .map(|t| format!("{:<10} - {}", t.name, t.description))
        .collect();

    let ans = Select::new("Choose a project template:", options).prompt()?;
    let chosen_name = ans.split_whitespace().next().unwrap_or("python").to_string();
    Ok(chosen_name)
}

pub fn show_selector() -> Result<()> {
    // 1. Find project and load config
    let (root, source) = root::find_project_root()
        .map_err(|_| anyhow!("USER_ERROR: No project found. Run `ztn init`."))?;
    env::set_current_dir(&root)?;
    let config = Config::load(&source)?;

    if config.tasks.is_empty() {
        return Err(anyhow!("USER_ERROR: No tasks defined in zetten.toml"));
    }

    // 2. Prepare task names for selection
    let mut options: Vec<String> = config.tasks.keys().cloned().collect();
    options.sort();

    // 3. Launch the interactive prompt
    let ans = Select::new("Select a task to run:", options)
        .with_help_message("↑↓ to move, enter to select, esc to exit")
        .prompt();

    match ans {
        Ok(choice) => {
            println!("Selected: {}", choice);
            // Argument #6: HashMap::new() ensures the 3-tier system has a map to check
            crate::run_tasks(vec![choice], "auto".to_string(), false, vec![], None, HashMap::new())?; // interactive=false
            Ok(())
        }
        Err(_) => {
            println!("Exited.");
            Ok(())
        }
    }
}