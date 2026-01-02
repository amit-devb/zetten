use anyhow::Result;
use crate::config::Config;
use std::collections::HashSet;

pub fn run(config: &Config) -> Result<()> {
    crate::log::info("Task Dependency Graph:");
    
    let mut visited = HashSet::new();
    
    // Print a tree for each "root" task (tasks that nothing else depends on)
    let all_deps: HashSet<_> = config.tasks.values()
        .flat_map(|t| t.depends_on.iter())
        .collect();

    let mut roots: Vec<_> = config.tasks.keys()
        .filter(|name| !all_deps.contains(name))
        .collect();
    roots.sort();

    if roots.is_empty() && !config.tasks.is_empty() {
        println!("⚠️  Cycle detected or all tasks are inter-dependent!");
    }

    for root in roots {
        print_tree(config, root, "", true, &mut visited);
    }

    Ok(())
}

fn print_tree(
    config: &Config, 
    name: &str, 
    prefix: &str, 
    is_last: bool, 
    visited: &mut HashSet<String>
) {
    let connector = if is_last { "└── " } else { "├── " };
    println!("{}{}{}", prefix, connector, name);

    if visited.contains(name) {
        println!("  {} [RECURSION/CYCLE DETECTED]", prefix);
        return;
    }

    visited.insert(name.to_string());

    if let Some(task) = config.tasks.get(name) {
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        let count = task.depends_on.len();
        
        for (i, dep) in task.depends_on.iter().enumerate() {
            print_tree(config, dep, &new_prefix, i == count - 1, visited);
        }
    }

    visited.remove(name);
}