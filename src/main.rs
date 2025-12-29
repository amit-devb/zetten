mod cache;
mod cli;
mod config;
mod init;
mod log;
mod progress;
mod root;
mod runner;
mod templates;
mod venv;

use anyhow::{anyhow, Result};
use cache::compute_hash;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use cli::{Cli, Command, Shell};
use config::{Config, TaskConfig};
use runner::{run_command, ExecutionResult};

use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

struct WorkItem {
    name: String,
    task: TaskConfig,
}

//
// --------------------------------------------------
// Collect transitive dependencies
// --------------------------------------------------
//
fn collect_tasks(config: &Config, roots: &[String]) -> Result<Vec<String>> {
    let mut visited = HashMap::new();
    let mut stack = roots.to_vec();

    while let Some(task) = stack.pop() {
        if visited.contains_key(&task) {
            continue;
        }

        let cfg = config
            .tasks
            .get(&task)
            .ok_or_else(|| anyhow!("Unknown task '{}'", task))?;

        visited.insert(task.clone(), true);

        for dep in &cfg.depends_on {
            stack.push(dep.clone());
        }
    }

    Ok(visited.keys().cloned().collect())
}

//
// --------------------------------------------------
// Cycle detection (DFS with stack)
// --------------------------------------------------
//
fn find_cycle(graph: &HashMap<String, Vec<String>>, nodes: &[String]) -> Option<Vec<String>> {
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

    for node in nodes {
        if !state.contains_key(node) {
            if let Some(cycle) = dfs(node, graph, &mut state, &mut stack) {
                return Some(cycle);
            }
        }
    }

    None
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // --------------------------------------------------
    // Handle shell completions EARLY
    // --------------------------------------------------
    match cli.command {
        Command::Init { template } => {
            init::init(&template)?;
            return Ok(());
        }

        Command::Completions { shell } => {
            let mut cmd = Cli::command();

            match shell {
                Shell::Bash => generate(shells::Bash, &mut cmd, "zetten", &mut std::io::stdout()),
                Shell::Zsh => generate(shells::Zsh, &mut cmd, "zetten", &mut std::io::stdout()),
                Shell::Fish => generate(shells::Fish, &mut cmd, "zetten", &mut std::io::stdout()),
            }

            return Ok(());
        }

        Command::Run { .. } => {}
    }

    // --------------------------------------------------
    // Project root auto-detection
    // --------------------------------------------------
    let root = root::find_project_root()?;
    env::set_current_dir(&root)?;

    let config = Config::load()?;
    config.validate()?;

    fs::create_dir_all(".zetten/cache")?;

    // --------------------------------------------------
    // Run command
    // --------------------------------------------------
    if let Command::Run { tasks, workers } = cli.command {
        let workers = if workers == "auto" {
            num_cpus::get()
        } else {
            workers
                .parse::<usize>()
                .map_err(|_| anyhow!("Invalid --worker value '{}'", workers))?
        };

        let tasks = collect_tasks(&config, &tasks)?;
        let progress = Arc::new(progress::Progress::new(tasks.len()));

        // --------------------------------------------------
        // Build DAG
        // --------------------------------------------------
        let mut indegree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        for name in &tasks {
            indegree.insert(name.clone(), 0);
        }

        for name in &tasks {
            let task = config.tasks.get(name).unwrap();
            for dep in &task.depends_on {
                if indegree.contains_key(dep) {
                    *indegree.get_mut(name).unwrap() += 1;
                    graph.entry(dep.clone()).or_default().push(name.clone());
                }
            }
        }

        let mut ready = VecDeque::new();
        for (task, &deg) in &indegree {
            if deg == 0 {
                ready.push_back(task.clone());
            }
        }

        if ready.is_empty() {
            if let Some(cycle) = find_cycle(&graph, &tasks) {
                return Err(anyhow!(
                    "Dependency cycle detected:\n{}",
                    cycle.join(" -> ")
                ));
            } else {
                return Err(anyhow!("Dependency cycle detected"));
            }
        }

        // --------------------------------------------------
        // Channels
        // --------------------------------------------------
        let (work_tx, work_rx) = mpsc::channel::<WorkItem>();
        let (result_tx, result_rx) = mpsc::channel::<Result<(String, ExecutionResult)>>();

        let work_rx = Arc::new(Mutex::new(work_rx));

        // --------------------------------------------------
        // Worker pool
        // --------------------------------------------------
        for wid in 0..workers {
            let work_rx = Arc::clone(&work_rx);
            let result_tx = result_tx.clone();
            let progress = progress.clone();

            thread::spawn(move || loop {
                let work = {
                    let rx = work_rx.lock().unwrap();
                    rx.recv()
                };

                let work = match work {
                    Ok(w) => w,
                    Err(_) => break,
                };

                let name = work.name;
                let task = work.task;

                progress.start_task();
                crate::log::log(&name, &format!("started on worker {}", wid));

                let result: Result<ExecutionResult> = (|| {
                    let hash = compute_hash(&task.inputs)?;
                    let cache_file = format!(".zetten/cache/{}.hash", name);

                    if let Ok(prev) = fs::read_to_string(&cache_file) {
                        if prev == hash {
                            crate::log::log(&name, "skipped (cached)");
                            return Ok(ExecutionResult {
                                exit_code: 0,
                                stdout: Vec::new(),
                                stderr: Vec::new(),
                            });
                        }
                    }

                    crate::log::log(&name, "running");

                    let exec = run_command(&task.cmd, &task.allow_exit_codes)?;

                    fs::write(&cache_file, &hash)?;
                    Ok(exec)
                })();

                progress.finish_task();
                crate::log::log(&name, "done");

                let _ = result_tx.send(result.map(|e| (name, e)));
            });
        }

        // --------------------------------------------------
        // Scheduler
        // --------------------------------------------------
        let mut remaining = indegree.clone();
        let mut in_flight = 0usize;

        for t in ready.drain(..) {
            let task = config.tasks.get(&t).unwrap().clone();
            work_tx.send(WorkItem { name: t, task })?;
            in_flight += 1;
        }

        while in_flight > 0 {
            let (finished, exec) = result_rx.recv()??;
            in_flight -= 1;

            crate::log::print_task_output(&finished, &exec);

            let (running, done, total) = progress.snapshot();
            println!(
                "[progress] running={} done={} total={}",
                running, done, total
            );

            if let Some(children) = graph.get(&finished) {
                for child in children {
                    let deg = remaining.get_mut(child).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        let task = config.tasks.get(child).unwrap().clone();
                        work_tx.send(WorkItem {
                            name: child.clone(),
                            task,
                        })?;
                        in_flight += 1;
                    }
                }
            }
        }

        drop(work_tx);
    }

    Ok(())
}
