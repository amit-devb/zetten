mod cache;
mod cli;
mod config;
mod doctor;
mod graph;
mod init;
mod log;
mod root;
mod runner;
mod templates;
mod venv;
mod watch;
mod orchestrator;

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

/* --------------------------------------------------
 * Exit helpers
 * -------------------------------------------------- */

fn exit_user_error(msg: &str) -> ! {
    crate::log::user_error(msg);
    std::process::exit(2);
}

fn exit_task_failure(code: i32) -> ! {
    std::process::exit(code);
}

fn exit_internal_error(err: anyhow::Error) -> ! {
    eprintln!("Internal error:\n{}", err);
    std::process::exit(3);
}

/* --------------------------------------------------
 * Work item
 * -------------------------------------------------- */

struct WorkItem {
    name: String,
    task: TaskConfig,
}

/* --------------------------------------------------
 * CI summary
 * -------------------------------------------------- */

struct RunSummary {
    succeeded: usize,
    cached: usize,
    failed: usize,
}

impl RunSummary {
    fn new() -> Self {
        Self {
            succeeded: 0,
            cached: 0,
            failed: 0,
        }
    }
}

/* --------------------------------------------------
 * Dependency resolution
 * -------------------------------------------------- */

fn collect_tasks(config: &Config, roots: &[String]) -> Result<Vec<String>> {
    let mut visited = HashMap::new();
    let mut stack = roots.to_vec();

    while let Some(task) = stack.pop() {
        if visited.contains_key(&task) {
            continue;
        }

        let cfg = config.tasks.get(&task).ok_or_else(|| {
            let available = config
                .tasks
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join("\n- ");

            anyhow!(
                "USER_ERROR: Unknown task '{}'\n\nAvailable tasks:\n- {}",
                task,
                available
            )
        })?;

        visited.insert(task.clone(), true);

        for dep in &cfg.depends_on {
            stack.push(dep.clone());
        }
    }

    Ok(visited.keys().cloned().collect())
}

/* --------------------------------------------------
 * Cycle detection
 * -------------------------------------------------- */

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

/* --------------------------------------------------
 * main
 * -------------------------------------------------- */

fn main() {
    let cli = Cli::parse();

    match run_main(cli) {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            let msg = e.to_string();

            if msg.starts_with("USER_ERROR:") {
                exit_user_error(msg.trim_start_matches("USER_ERROR:").trim());
            }

            exit_internal_error(e);
        }
    }
}

/* --------------------------------------------------
 * run_main
 * -------------------------------------------------- */

fn run_main(cli: Cli) -> Result<()> {
    match &cli.command {
        Command::Init { template } => {
            init::init(template)?;
            return Ok(());
        }

        Command::Watch { tasks } => {
            let (root, source) = root::find_project_root().map_err(|_| {
                anyhow!(
                    "USER_ERROR: Zetten is not initialized.\n\n\
                        Run `zetten init` to get started."
                )
            })?;

            std::env::set_current_dir(&root)?;
            let config = Config::load(&source)?;
            config.validate()?;

            watch::run(&config, &tasks)?;
            return Ok(());
        }

        Command::Doctor => {
            doctor::run()?;
            return Ok(());
        }

        Command::Graph => {
            let (root, source) = root::find_project_root()
                .map_err(|_| anyhow!("USER_ERROR: Zetten is not initialized"))?;

            env::set_current_dir(&root)?;
            let config = Config::load(&source)?;
            config.validate()?;
            graph::run(&config)?;
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

    let (root, source) = root::find_project_root().map_err(|_| {
        anyhow!(
            "USER_ERROR: Zetten is not initialized.\n\n\
            No Zetten configuration was found:\n\
            - pyproject.toml with [tool.zetten]\n\
            - zetten.toml\n\n\
            To get started, run:\n\n\
                zetten init"
        )
    })?;

    env::set_current_dir(&root)?;

    let config = Config::load(&source)?;
    config.validate()?;

    fs::create_dir_all(".zetten/cache")?;

    if let Command::Run { tasks, workers } = cli.command {
        let workers = if workers == "auto" {
            num_cpus::get()
        } else {
            workers
                .parse::<usize>()
                .map_err(|_| anyhow!("USER_ERROR: Invalid --worker value '{}'", workers))?
        };

        let tasks = collect_tasks(&config, &tasks)?;
        let mut summary = RunSummary::new();

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
                    "USER_ERROR: Dependency cycle detected:\n{}",
                    cycle.join(" -> ")
                ));
            }
        }

        let (work_tx, work_rx) = mpsc::channel::<WorkItem>();
        let (result_tx, result_rx) = mpsc::channel::<Result<(String, ExecutionResult)>>();

        let work_rx = Arc::new(Mutex::new(work_rx));

        for _wid in 0..workers {
            let work_rx = Arc::clone(&work_rx);
            let result_tx = result_tx.clone();

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

                let result: Result<ExecutionResult> = (|| {
                    let hash = compute_hash(&task.inputs)?;
                    let cache_file = format!(".zetten/cache/{}.hash", name);

                    if let Ok(prev) = fs::read_to_string(&cache_file) {
                        if prev == hash {
                            return Ok(ExecutionResult {
                                exit_code: 0,
                                stdout: Vec::new(),
                                stderr: Vec::new(),
                            });
                        }
                    }

                    let exec = run_command(&task.cmd, &task.allow_exit_codes)?;
                    fs::write(&cache_file, &hash)?;
                    Ok(exec)
                })();

                let _ = result_tx.send(result.map(|e| (name, e)));
            });
        }

        let mut remaining: HashMap<String, usize> = indegree.clone();
        let mut in_flight = 0;

        for t in ready.drain(..) {
            let task = config.tasks.get(&t).unwrap().clone();
            work_tx.send(WorkItem { name: t, task })?;
            in_flight += 1;
        }

        while in_flight > 0 {
            let (finished, exec) = result_rx.recv()??;
            in_flight -= 1;

            if exec.exit_code == 0 {
                let cached = exec.stdout.is_empty() && exec.stderr.is_empty();

                if cached {
                    summary.cached += 1;
                } else {
                    summary.succeeded += 1;
                }

                crate::log::task_ok(&finished, cached);
            } else {
                summary.failed += 1;
                crate::log::task_fail(&finished, exec.exit_code);
                exit_task_failure(exec.exit_code);
            }

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

        crate::log::info("\nSummary:");
        crate::log::info(&format!("- {} tasks succeeded", summary.succeeded));
        crate::log::info(&format!("- {} tasks cached", summary.cached));
        crate::log::info(&format!("- {} tasks failed", summary.failed));
    }

    Ok(())
}
