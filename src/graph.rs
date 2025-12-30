use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::config::Config;
use crate::log;

/// Entry point for `zetten graph`
pub fn run(config: &Config) -> Result<()> {
    let graph = build_graph(config)?;

    if let Some(cycle) = find_cycle(&graph) {
        return Err(anyhow!(
            "USER_ERROR: Dependency cycle detected:\n{}",
            cycle.join(" -> ")
        ));
    }

    print_graph(&graph);

    Ok(())
}

/// Build adjacency list: task -> dependents
fn build_graph(config: &Config) -> Result<HashMap<String, Vec<String>>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    // Ensure all tasks exist as keys
    for task in config.tasks.keys() {
        graph.entry(task.clone()).or_default();
    }

    for (task, cfg) in &config.tasks {
        for dep in &cfg.depends_on {
            graph
                .entry(dep.clone())
                .or_default()
                .push(task.clone());
        }
    }

    Ok(graph)
}

/// Print graph in human-readable form
fn print_graph(graph: &HashMap<String, Vec<String>>) {
    log::info("Task graph:\n");

    let mut tasks: Vec<_> = graph.keys().cloned().collect();
    tasks.sort();

    for task in tasks {
        match graph.get(&task) {
            Some(children) if !children.is_empty() => {
                let mut children = children.clone();
                children.sort();
                log::info(&format!("{} -> {}", task, children.join(", ")));
            }
            _ => {
                log::info(&task);
            }
        }
    }
}

/// Detect dependency cycles (DFS)
fn find_cycle(graph: &HashMap<String, Vec<String>>) -> Option<Vec<String>> {
    #[derive(Clone, Copy)]
    enum State {
        Visiting,
        Visited,
    }

    let mut state: HashMap<String, State> = HashMap::new();
    let mut stack: Vec<String> = Vec::new();

    fn dfs(
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        state: &mut HashMap<String, State>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        state.insert(node.to_string(), State::Visiting);
        stack.push(node.to_string());

        if let Some(children) = graph.get(node) {
            for child in children {
                match state.get(child) {
                    Some(State::Visiting) => {
                        let idx = stack.iter().position(|n| n == child).unwrap();
                        let mut cycle = stack[idx..].to_vec();
                        cycle.push(child.clone());
                        return Some(cycle);
                    }
                    Some(State::Visited) => {}
                    None => {
                        if let Some(cycle) = dfs(child, graph, state, stack) {
                            return Some(cycle);
                        }
                    }
                }
            }
        }

        stack.pop();
        state.insert(node.to_string(), State::Visited);
        None
    }

    for node in graph.keys() {
        if !state.contains_key(node) {
            if let Some(cycle) = dfs(node, graph, &mut state, &mut stack) {
                return Some(cycle);
            }
        }
    }

    None
}
