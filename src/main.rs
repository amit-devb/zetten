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
mod validator;
mod watch;

use crate::progress::Progress;
use anyhow::{anyhow, Result};
use cache::compute_hash;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use cli::{Cli, Command};
use colored::*;

use config::Config;
use lazy_static::lazy_static;
use runner::{execute_task_command, ExecutionResult};
use std::process::Child;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    env, fs,
    path::Path,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use validator::validate_execution_env;

lazy_static! {
    static ref PROCESS_REGISTRY: Arc<Mutex<Vec<Child>>> = Arc::new(Mutex::new(Vec::new()));
}

fn main() -> anyhow::Result<()> {
    // 1. Pillar 2: Graceful Shutdown
    ctrlc::set_handler(move || {
        println!(
            "\n{}",
            "🛑 Shutdown signal received! Cleaning up processes..."
                .red()
                .bold()
        );
        if let Ok(mut registry) = PROCESS_REGISTRY.lock() {
            for mut child in registry.drain(..) {
                let _ = child.kill();
            }
        }
        println!("{}", "✔ Cleanup complete. Exiting.".yellow());
        std::process::exit(130);
    })?;

    // 2. CLI Parsing
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
                show_usage_guide(load_config_safe().as_ref());
                std::process::exit(1);
            }
            e.exit();
        }
    };

    if Path::new(".env").exists() && dotenvy::dotenv().is_ok() {
        crate::log::info("Environment variables loaded from .env");
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

    Ok(())
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
    if let Some(cfg) = config {
        println!("\n{}", "Detected Tasks:".bold());
        let mut tasks: Vec<_> = cfg.tasks.keys().collect();
        tasks.sort();
        for t in tasks.iter().take(5) {
            println!("  - {: <12} {}", t.yellow(), cfg.tasks[*t].description);
        }
    }
}

pub fn run_main(cli: Cli) -> Result<()> {
    let command = match cli.command {
        Some(cmd) => cmd,
        None => return tui::show_selector(),
    };

    match command {
        Command::Tasks => {
            let (root, source) = root::find_project_root()?;
            env::set_current_dir(&root)?;
            let config = Config::load(&source)?;
            let mut keys: Vec<_> = config.tasks.keys().collect();
            keys.sort();
            for name in keys {
                println!("  {:<15} {}", name, config.tasks[name].description);
            }
            Ok(())
        }
        Command::Run {
            tasks,
            workers,
            dry_run,
            kv,
            args,
            tag,
        } => {
            if tasks.is_empty() && tag.is_none() {
                tui::show_selector()
            } else {
                // Convert CLI Vec to HashMap for the merger
                let cli_vars: HashMap<String, String> = kv.into_iter().collect();
                run_tasks(tasks, workers, dry_run, args, tag, cli_vars)
            }
        }
        Command::Watch { tasks } => {
            let (root, source) = root::find_project_root()?;
            env::set_current_dir(&root)?;
            let config = Config::load(&source)?;
            if tasks.is_empty() {
                return Err(anyhow!("USER_ERROR: Please specify which tasks to watch"));
            }
            watch::run(&config, &tasks)
        }
        Command::Doctor => doctor::run(),
        Command::Graph => {
            let (root, source) = root::find_project_root()?;
            env::set_current_dir(&root)?;
            graph::run(&Config::load(&source)?)
        }
        Command::Init { template } => init::init(template.as_deref().unwrap_or("interactive")),
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
    cli_vars: HashMap<String, String>, // Added
) -> Result<()> {
    let (root, source) =
        root::find_project_root().map_err(|_| anyhow!("USER_ERROR: Not a project root."))?;
    env::set_current_dir(&root)?;

    let config = Arc::new(Config::load(&source)?);
    config.validate()?;

    // --- NEW: THREE-TIER VARIABLE MERGE ---
    let mut all_vars = HashMap::new();
    for (k, v) in env::vars() {
        all_vars.insert(k, v);
    } // Tier 3
    for (k, v) in &config.vars {
        all_vars.insert(k.clone(), v.clone());
    } // Tier 2
    for (k, v) in cli_vars {
        all_vars.insert(k, v);
    } // Tier 1 (Winner)
    let all_vars = Arc::new(all_vars);

    let mut root_tasks = tasks;
    if let Some(ref t) = tag_filter {
        let tagged: Vec<String> = config
            .tasks
            .iter()
            .filter(|(_, c)| c.tags.contains(t))
            .map(|(n, _)| n.clone())
            .collect();
        root_tasks.extend(tagged);
    }

    let task_names = collect_tasks(&config, &root_tasks)?;
    crate::log::info("🔍 Validating environment...");
    if let Err(e) = validate_execution_env(&config, &task_names) {
        crate::log::user_error(&format!("Validation failed: {}", e));
        std::process::exit(1);
    }

    if dry_run {
        crate::log::info("🌵 Dry Run Plan:");
        for n in &task_names {
            println!(
                "  [{}] {}",
                n,
                config.tasks[n].resolve_cmd(&args, &all_vars)
            );
        }
        return Ok(());
    }

    let workers_count = if workers == "auto" {
        num_cpus::get()
    } else {
        workers.parse()?
    };
    let is_parallel = task_names.len() > 1 && workers_count > 1;
    let mut summary = RunSummary::new();

    // --- Kahn's Algorithm Setup ---
    let mut indegree: HashMap<String, usize> = HashMap::new();
    let mut graph_map: HashMap<String, Vec<String>> = HashMap::new();
    for n in &task_names {
        indegree.insert(n.clone(), 0);
    }
    for n in &task_names {
        for dep in &config.tasks[n].depends_on {
            if indegree.contains_key(dep) {
                *indegree.get_mut(n).unwrap() += 1;
                graph_map.entry(dep.clone()).or_default().push(n.clone());
            }
        }
    }

    let mut ready = VecDeque::new();
    for (t, &deg) in &indegree {
        if deg == 0 {
            ready.push_back(t.clone());
        }
    }

    let progress = Arc::new(Progress::new(task_names.len()));
    let (tx, rx) = mpsc::channel::<Result<(String, ExecutionResult, bool)>>();
    let work_queue = Arc::new(Mutex::new(VecDeque::<String>::new()));

    // Spawn Workers
    for _ in 0..workers_count {
        let q = Arc::clone(&work_queue);
        let t_tx = tx.clone();
        let p = Arc::clone(&progress);
        let cfg = Arc::clone(&config);
        let f_args = args.clone();
        let vars = Arc::clone(&all_vars); // Clone the Arc for the thread

        thread::spawn(move || loop {
            let task_name = {
                let mut lock = q.lock().unwrap();
                match lock.pop_front() {
                    Some(name) => name,
                    None => {
                        drop(lock);
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                }
            };

            let task_cfg = cfg.tasks.get(&task_name).unwrap();
            let final_cmd = task_cfg.resolve_cmd(&f_args, &vars); // Resolved with hierarchy

            if is_parallel {
                p.start_task(&task_name);
            }

            let res: Result<(ExecutionResult, bool)> = (|| {
                let cache_path = format!(".zetten/cache/{}.hash", task_name);
                if !task_cfg.inputs.is_empty() && f_args.is_empty() {
                    let hash = compute_hash(&task_cfg.inputs)?;
                    if fs::read_to_string(&cache_path)
                        .map(|s| s == hash)
                        .unwrap_or(false)
                    {
                        return Ok((
                            ExecutionResult {
                                is_success: true,
                                ..Default::default()
                            },
                            true,
                        ));
                    }
                    let exec =
                        execute_task_command(&final_cmd, &task_cfg.allow_exit_codes, is_parallel)?;
                    if exec.is_success {
                        let _ = fs::create_dir_all(".zetten/cache");
                        let _ = fs::write(cache_path, hash);
                    }
                    Ok((exec, false))
                } else {
                    let exec =
                        execute_task_command(&final_cmd, &task_cfg.allow_exit_codes, is_parallel)?;
                    Ok((exec, false))
                }
            })();

            if is_parallel {
                p.finish_task();
            }
            let _ = t_tx.send(res.map(|(e, c)| (task_name, e, c)));
        });
    }

    let mut in_flight = 0;
    for t in ready {
        work_queue.lock().unwrap().push_back(t);
        in_flight += 1;
    }

    let mut exit_code = 0;
    while in_flight > 0 {
        let (finished, exec, cached) = rx.recv()??;
        in_flight -= 1;
        summary.task_metrics.insert(finished.clone(), exec.duration);
        let task_cfg = config.tasks.get(&finished).unwrap();

        let mut log_action = || {
            if exec.is_success || task_cfg.ignore_errors {
                if !exec.is_success {
                    summary.warned += 1;
                    crate::log::warn(&format!("Task '{}' failed (ignored).", finished));
                } else if cached {
                    summary.cached += 1;
                    crate::log::task_ok(&finished, true);
                } else {
                    summary.succeeded += 1;
                    crate::log::task_ok(&finished, false);
                }
            } else {
                summary.failed += 1;
                exit_code = exec.exit_code;
                crate::log::task_fail(&finished, exec.exit_code);
                if is_parallel {
                    if !exec.stdout.is_empty() {
                        println!("{}", String::from_utf8_lossy(&exec.stdout));
                    }
                    if !exec.stderr.is_empty() {
                        eprintln!("{}", String::from_utf8_lossy(&exec.stderr));
                    }
                }
                if let Some(h) = task_cfg.hint.as_ref() {
                    crate::log::suggestion(&finished, h);
                }
            }
        };

        if is_parallel {
            progress.pb.suspend(log_action);
        } else {
            log_action();
        }

        if !exec.is_success && !task_cfg.ignore_errors {
            break;
        }

        if let Some(children) = graph_map.get(&finished) {
            for child in children {
                let deg = indegree.get_mut(child).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    work_queue.lock().unwrap().push_back(child.clone());
                    in_flight += 1;
                }
            }
        }
    }

    if is_parallel {
        progress.pb.finish_and_clear();
    }
    print_summary(&summary, &config, &task_names);
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

// --- Summary & Logic Helpers (Preserved) ---
struct RunSummary {
    succeeded: usize,
    cached: usize,
    failed: usize,
    warned: usize,
    start_time: Instant,
    task_metrics: HashMap<String, Duration>,
}
impl RunSummary {
    fn new() -> Self {
        Self {
            succeeded: 0,
            cached: 0,
            failed: 0,
            warned: 0,
            start_time: Instant::now(),
            task_metrics: HashMap::new(),
        }
    }
}

fn find_critical_path(
    config: &Config,
    metrics: &HashMap<String, Duration>,
    task_names: &[String],
) -> (Vec<String>, Duration) {
    let mut cache: HashMap<String, (Vec<String>, Duration)> = HashMap::new();
    fn compute_path(
        node: &str,
        config: &Config,
        metrics: &HashMap<String, Duration>,
        cache: &mut HashMap<String, (Vec<String>, Duration)>,
    ) -> (Vec<String>, Duration) {
        if let Some(res) = cache.get(node) {
            return res.clone();
        }
        let current_dur = *metrics.get(node).unwrap_or(&Duration::ZERO);
        let mut best_path = vec![node.to_string()];
        let mut max_dep_dur = Duration::ZERO;
        if let Some(task_cfg) = config.tasks.get(node) {
            for dep in &task_cfg.depends_on {
                let (path, dur) = compute_path(dep, config, metrics, cache);
                if dur > max_dep_dur {
                    max_dep_dur = dur;
                    let mut new_path = path;
                    new_path.push(node.to_string());
                    best_path = new_path;
                }
            }
        }
        let res = (best_path, current_dur + max_dep_dur);
        cache.insert(node.to_string(), res.clone());
        res
    }
    let mut longest_path = Vec::new();
    let mut max_total = Duration::ZERO;
    for task in task_names {
        let (path, dur) = compute_path(task, config, metrics, &mut cache);
        if dur > max_total {
            max_total = dur;
            longest_path = path;
        }
    }
    (longest_path, max_total)
}

fn print_summary(s: &RunSummary, config: &Config, task_names: &[String]) {
    let total_wall = s.start_time.elapsed();
    let total_work: Duration = s.task_metrics.values().sum();
    let saved = if total_work > total_wall {
        total_work - total_wall
    } else {
        Duration::ZERO
    };
    println!("\n{}", "Summary:".bold());
    println!(
        "  {} succeeded, {} cached, {} warned, {} failed",
        s.succeeded.to_string().green(),
        s.cached.to_string().cyan(),
        s.warned.to_string().yellow(),
        s.failed.to_string().red()
    );
    println!(
        "  Total time: {:.2?} ({} saved via parallelism)",
        total_wall,
        format!("{:.2?}", saved).yellow().bold()
    );
    if !s.task_metrics.is_empty() {
        let (path, _) = find_critical_path(config, &s.task_metrics, task_names);
        if path.len() > 1 {
            println!("\n{}", "Critical Path (Bottleneck):".bold().dimmed());
            println!("  {}", path.join(" ➔ ").magenta());
            if let Some(bottleneck) = path.last() {
                println!(
                    "  Tip: Speeding up {} will reduce total run time.",
                    bottleneck.yellow()
                );
            }
        }
    }
}

fn collect_tasks(config: &Config, roots: &[String]) -> Result<Vec<String>> {
    let mut expanded = HashSet::new();
    let mut stack = roots.to_vec();
    while let Some(t) = stack.pop() {
        if expanded.insert(t.clone()) {
            if let Some(c) = config.tasks.get(&t) {
                for d in &c.depends_on {
                    stack.push(d.clone());
                }
            } else {
                return Err(anyhow!("USER_ERROR: Task '{}' not found.", t));
            }
        }
    }
    let mut sorted = Vec::new();
    let mut visited = HashSet::new();
    let mut visiting = HashSet::new();
    fn visit(
        n: &str,
        cfg: &Config,
        s: &mut Vec<String>,
        v: &mut HashSet<String>,
        vg: &mut HashSet<String>,
    ) -> Result<()> {
        if vg.contains(n) {
            return Err(anyhow!("USER_ERROR: Circular dependency at '{}'", n));
        }
        if !v.contains(n) {
            vg.insert(n.to_string());
            if let Some(t) = cfg.tasks.get(n) {
                for d in &t.depends_on {
                    visit(d, cfg, s, v, vg)?;
                }
            }
            vg.remove(n);
            v.insert(n.to_string());
            s.push(n.to_string());
        }
        Ok(())
    }
    for n in expanded {
        visit(&n, config, &mut sorted, &mut visited, &mut visiting)?;
    }
    Ok(sorted)
}

fn find_closest<'a>(i: &str, opts: Vec<&'a str>) -> Option<&'a str> {
    opts.into_iter()
        .map(|o| (o, strsim::levenshtein(i, o)))
        .filter(|(_, d)| *d <= 2)
        .min_by_key(|(_, d)| *d)
        .map(|(o, _)| o)
}
