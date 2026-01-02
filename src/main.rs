mod cache;
mod cli;
mod config;
mod doctor;
mod graph;
mod init;
mod log;
mod progress;
mod root;
mod runner;
mod templates;
mod tui;
mod watch;

use crate::progress::Progress;
use anyhow::{anyhow, Result};
use cache::compute_hash;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use cli::{Cli, Command};
use colored::*;
use config::{Config, TaskConfig};
use runner::{run_command, ExecutionResult};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    env, fs,
    path::Path,
    sync::{mpsc, Arc, Mutex},
    thread,
};

fn main() {
    // 1. Intercept CLI errors to show custom help & suggestions
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == clap::error::ErrorKind::InvalidSubcommand
                || e.kind() == clap::error::ErrorKind::UnknownArgument
            {
                let cmd_factory = Cli::command();
                let subcommands: Vec<&str> = cmd_factory
                    .get_subcommands()
                    .map(|s| s.get_name())
                    .collect();
                let args: Vec<String> = std::env::args().collect();
                if args.len() > 1 {
                    if let Some(suggest) = find_closest(&args[1], subcommands) {
                        crate::log::did_you_mean(&args[1], suggest);
                    }
                }
                let config = load_config_safe();
                show_usage_guide(config.as_ref());
                std::process::exit(1);
            }
            e.exit();
        }
    };

    // 2. FEATURE: Smart .env Auto-Loader
    if Path::new(".env").exists() {
        if dotenvy::dotenv().is_ok() {
            crate::log::info("Environment variables loaded from .env");
        }
    }

    if let Err(e) = run_main(cli) {
        let msg = e.to_string();
        if msg.starts_with("USER_ERROR:") {
            crate::log::user_error(msg.trim_start_matches("USER_ERROR:").trim());
        } else {
            eprintln!("{} {}", "Error:".red().bold(), e);
        }
        std::process::exit(1);
    }
}

fn load_config_safe() -> Option<Config> {
    if let Ok((root, source)) = root::find_project_root() {
        let _ = env::set_current_dir(&root);
        return Config::load(&source).ok();
    }
    None
}

fn show_usage_guide(config: Option<&Config>) {
    println!("{}", "\n--- Zetten Usage Guide ---".bold().blue());
    println!("Usage: {} {}", "zetten".green(), "[COMMAND]".cyan());

    println!("\n{}", "Standard Commands:".bold());
    println!("  {: <12} Open task selector (TUI)", "(none)".dimmed());
    println!("  {: <12} Run project tasks", "run".cyan());
    println!("  {: <12} Watch files for changes", "watch".cyan());
    println!("  {: <12} Diagnose project setup", "doctor".cyan());

    if let Some(cfg) = config {
        println!("\n{}", "Detected Tasks:".bold());
        let mut tasks: Vec<_> = cfg.tasks.keys().collect();
        tasks.sort();
        for task in tasks.iter().take(5) {
            println!(
                "  - {: <12} {}",
                task.yellow(),
                cfg.tasks[*task].description
            );
        }
    }
    println!("");
}

pub fn run_main(cli: Cli) -> Result<()> {
    let command = match cli.command {
        Some(cmd) => cmd,
        None => return tui::show_selector(), // Default TUI
    };

    match command {
        Command::Tasks => {
            let (root, source) =
                root::find_project_root().map_err(|_| anyhow!("USER_ERROR: No project found."))?;
            env::set_current_dir(&root)?;
            let config = Config::load(&source)?;
            crate::log::info("Available tasks:");
            let mut keys: Vec<_> = config.tasks.keys().collect();
            keys.sort();
            for name in keys {
                println!("  {:<15} {}", name, config.tasks[name].description);
            }
            Ok(())
        }
        Command::Init { template } => init::init(template.as_deref().unwrap_or("interactive")),
        Command::Run {
            tasks,
            workers,
            dry_run,
            args,
            tag,
        } => {
            // If no tasks and no tag, open TUI
            if tasks.is_empty() && tag.is_none() {
                tui::show_selector()
            } else {
                run_tasks(tasks, workers, dry_run, args, tag)
            }
        }
        Command::Watch { tasks } => {
            let (root, source) = root::find_project_root()?;
            env::set_current_dir(&root)?;
            let config = Config::load(&source)?;
            watch::run(&config, &tasks)
        }
        Command::Doctor => doctor::run(),
        Command::Graph => {
            let (root, source) = root::find_project_root()?;
            env::set_current_dir(&root)?;
            let config = Config::load(&source)?;
            graph::run(&config)
        }
        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            let bin = "zetten";
            match shell {
                cli::Shell::Bash => generate(shells::Bash, &mut cmd, bin, &mut std::io::stdout()),
                cli::Shell::Zsh => generate(shells::Zsh, &mut cmd, bin, &mut std::io::stdout()),
                cli::Shell::Fish => generate(shells::Fish, &mut cmd, bin, &mut std::io::stdout()),
            }
            Ok(())
        }
    }
}

pub(crate) fn run_tasks(
    tasks: Vec<String>,
    workers: String,
    dry_run: bool,
    args: Vec<String>,
    tag_filter: Option<String>,
) -> Result<()> {
    let (root, source) =
        root::find_project_root().map_err(|_| anyhow!("USER_ERROR: Not a project root."))?;
    env::set_current_dir(&root)?;
    let config = Config::load(&source)?;

    // 1. Validation Layer (Circular check and existence)
    config.validate()?;

    for arg in &args {
        if let Some((k, v)) = arg.split_once('=') {
            env::set_var(k, v);
        }
    }

    // 2. CI Tagging Feature
    let mut root_tasks = tasks;
    if let Some(ref t) = tag_filter {
        let tagged: Vec<String> = config
            .tasks
            .iter()
            .filter(|(_, cfg)| cfg.tags.contains(t))
            .map(|(name, _)| name.clone())
            .collect();

        if tagged.is_empty() {
            return Err(anyhow!("USER_ERROR: No tasks found with tag '{}'", t));
        }
        root_tasks.extend(tagged);
    }

    // 3. Topological Ordering (Ensures depends_on is respected)
    let task_names = collect_tasks(&config, &root_tasks)?;
    let total_tasks_count = task_names.len();

    if dry_run {
        crate::log::info("🌵 Dry Run - Execution Plan:");
        for name in &task_names {
            println!(
                "  [{}] {}",
                name,
                config.tasks.get(name).unwrap().resolve_cmd(&args)
            );
        }
        return Ok(());
    }

    let workers_count = if workers == "auto" {
        num_cpus::get()
    } else {
        workers.parse()?
    };
    let is_parallel = total_tasks_count > 1 && workers_count > 1;

    let mut summary = RunSummary::new();

    // 4. Execution Strategy: Kahn's Algorithm for Concurrency
    let mut indegree: HashMap<String, usize> = HashMap::new();
    let mut graph_map: HashMap<String, Vec<String>> = HashMap::new();

    for name in &task_names {
        indegree.insert(name.clone(), 0);
    }
    for name in &task_names {
        for dep in &config.tasks[name].depends_on {
            if indegree.contains_key(dep) {
                *indegree.get_mut(name).unwrap() += 1;
                graph_map.entry(dep.clone()).or_default().push(name.clone());
            }
        }
    }

    let mut ready_tasks = VecDeque::new();
    for (t, &deg) in &indegree {
        if deg == 0 {
            ready_tasks.push_back(t.clone());
        }
    }

    let progress = Arc::new(Progress::new(total_tasks_count));
    let (tx, rx) = mpsc::channel::<Result<(String, ExecutionResult, bool)>>();
    let work_queue = Arc::new(Mutex::new(VecDeque::<WorkItem>::new()));

    // 5. Concurrency: Parallel Workers
    for _ in 0..workers_count {
        let q = Arc::clone(&work_queue);
        let t_tx = tx.clone();
        let p = Arc::clone(&progress);

        thread::spawn(move || loop {
            let work = {
                let mut lock = q.lock().unwrap();
                match lock.pop_front() {
                    Some(w) => w,
                    None => {
                        drop(lock);
                        thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                }
            };

            if is_parallel {
                p.start_task(&work.name);
            }
            let final_cmd = work.task.resolve_cmd(&work.extra_args);

            let res: Result<(ExecutionResult, bool)> = (|| {
                let cache_path = format!(".zetten/cache/{}.hash", work.name);
                let mut was_cached = false;

                if !work.task.inputs.is_empty() && work.extra_args.is_empty() {
                    let hash = compute_hash(&work.task.inputs)?;
                    if let Ok(prev) = fs::read_to_string(&cache_path) {
                        if prev == hash {
                            was_cached = true;
                            return Ok((
                                ExecutionResult {
                                    is_success: true,
                                    ..Default::default()
                                },
                                was_cached,
                            ));
                        }
                    }
                    let exec = run_command(&final_cmd, &work.task.allow_exit_codes, is_parallel)?;
                    if exec.is_success {
                        let _ = fs::create_dir_all(".zetten/cache");
                        let _ = fs::write(cache_path, hash);
                    }
                    Ok((exec, was_cached))
                } else {
                    let exec = run_command(&final_cmd, &work.task.allow_exit_codes, is_parallel)?;
                    Ok((exec, false))
                }
            })();

            if is_parallel {
                p.finish_task();
            }
            let _ = t_tx.send(res.map(|(e, cached)| (work.name, e, cached)));
        });
    }

    let mut in_flight = 0;
    let push_work = |q: &Arc<Mutex<VecDeque<WorkItem>>>,
                     name: String,
                     config: &Config,
                     extra_args: Vec<String>| {
        let task = config.tasks.get(&name).unwrap().clone();
        q.lock().unwrap().push_back(WorkItem {
            name,
            task,
            extra_args,
        });
    };

    // Kick off initial ready tasks
    for t in ready_tasks {
        push_work(&work_queue, t, &config, args.clone());
        in_flight += 1;
    }

    // 6. Failure Propagation and Orchestration
    let mut exit_code = 0;
    while in_flight > 0 {
        let (finished, exec, was_cached) = rx.recv()??;
        in_flight -= 1;

        if exec.is_success {
            if was_cached {
                summary.cached += 1;
            } else {
                summary.succeeded += 1;
            }

            if is_parallel {
                progress
                    .pb
                    .suspend(|| crate::log::task_ok(&finished, was_cached));
            } else {
                crate::log::task_ok(&finished, was_cached);
            }

            // Unlock dependents
            if let Some(children) = graph_map.get(&finished) {
                for child in children {
                    let deg = indegree.get_mut(child).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        push_work(&work_queue, child.clone(), &config, vec![]);
                        in_flight += 1;
                    }
                }
            }
        } else {
            // FAILURE PROPAGATION: Stop launching new tasks

            summary.failed += 1;
            exit_code = exec.exit_code;


            if is_parallel {
                // In parallel mode, we dump the captured output ONLY on failure
                // so the user knows WHY it failed.
                if !exec.stdout.is_empty() {
                    println!("{}", String::from_utf8_lossy(&exec.stdout));
                }
                if !exec.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&exec.stderr));
                }
            }

            crate::log::task_fail(&finished, exec.exit_code);
            if let Some(task_cfg) = config.tasks.get(&finished) {
                if let Some(h) = &task_cfg.hint {
                    crate::log::suggestion(&finished, h);
                }
            }
            break;
        }
    }

    if is_parallel {
        progress.pb.finish_and_clear();
    }
    print_summary(&summary);
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

// --- Support Structures & Functions ---

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

struct WorkItem {
    name: String,
    task: TaskConfig,
    extra_args: Vec<String>,
}

fn print_summary(s: &RunSummary) {
    println!("\nSummary:");
    println!(
        "- {} succeeded, {} cached, {} failed",
        s.succeeded, s.cached, s.failed
    );
}

fn collect_tasks(config: &Config, roots: &[String]) -> Result<Vec<String>> {
    let mut expanded = HashSet::new();
    let mut stack = roots.to_vec();

    while let Some(task_name) = stack.pop() {
        if expanded.insert(task_name.clone()) {
            if let Some(task_cfg) = config.tasks.get(&task_name) {
                for dep in &task_cfg.depends_on {
                    stack.push(dep.clone());
                }
            } else {
                return Err(anyhow!("USER_ERROR: Task '{}' not found.", task_name));
            }
        }
    }

    let mut sorted = Vec::new();
    let mut visited = HashSet::new();
    let mut visiting = HashSet::new();

    fn visit(
        name: &str,
        config: &Config,
        sorted: &mut Vec<String>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
    ) -> Result<()> {
        if visiting.contains(name) {
            return Err(anyhow!(
                "USER_ERROR: Circular dependency detected at '{}'",
                name
            ));
        }
        if !visited.contains(name) {
            visiting.insert(name.to_string());
            if let Some(task) = config.tasks.get(name) {
                for dep in &task.depends_on {
                    visit(dep, config, sorted, visited, visiting)?;
                }
            }
            visiting.remove(name);
            visited.insert(name.to_string());
            sorted.push(name.to_string());
        }
        Ok(())
    }

    for name in expanded {
        visit(&name, config, &mut sorted, &mut visited, &mut visiting)?;
    }
    Ok(sorted)
}

fn find_closest<'a>(input: &str, options: Vec<&'a str>) -> Option<&'a str> {
    options
        .into_iter()
        .map(|opt| (opt, strsim::levenshtein(input, opt)))
        .filter(|(_, dist)| *dist <= 2)
        .min_by_key(|(_, dist)| *dist)
        .map(|(opt, _)| opt)
}
