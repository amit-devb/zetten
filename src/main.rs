mod cache;
mod cli;
mod config;
mod doctor;
mod errors; // New module
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
// Remove global Result imports to avoid ALL confusion
// use anyhow::{anyhow}; // Use full paths for anyhow types
use cache::compute_hash;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use cli::{Cli, Command};
use colored::*;
use config::Config;
use errors::ZettenError; // Import
use lazy_static::lazy_static;
use miette::IntoDiagnostic; // Import
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

fn main() -> miette::Result<()> {
    // 1. CLI Parsing (Fast Path)
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == clap::error::ErrorKind::InvalidSubcommand {
                let cmd_factory = Cli::command();
                let subcommands: Vec<&str> = cmd_factory
                    .get_subcommands()
                    .map(|s| s.get_name())
                    .collect();
                let args: Vec<String> = std::env::args().collect();
                if args.len() > 1 {
                    if let Some(suggest) = find_closest(&args[1], subcommands) {
                         crate::log::did_you_mean(&args[1], suggest);
                         // If we found a suggestion for the subcommand, exit with usage.
                         show_usage_guide(load_config_safe().as_ref());
                         std::process::exit(1);
                    }
                }
            }
            // For UnknownArgument or if no suggestion found, let Clap print its error
            e.exit();
        }
    };

    // 2. Pillar 2: Graceful Shutdown (Only for actual execution)
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
    }).into_diagnostic()?;

    if Path::new(".env").exists() && dotenvy::dotenv().is_ok() {
        crate::log::info("Environment variables loaded from .env");
    }

    if let Err(e) = run_main(cli) {
        // Miette will handle the printing nicely
        return Err(e);
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
    println!("{}", "\n--- Zetten (ztn) Usage Guide ---".bold().blue());
    println!("Usage: {} {}", "ztn".green(), "[COMMAND]".cyan());
    if let Some(cfg) = config {
        println!("\n{}", "Detected Tasks:".bold());
        let mut tasks: Vec<_> = cfg.tasks.keys().collect();
        tasks.sort();
        for t in tasks.iter().take(5) {
            println!("  - {: <12} {}", t.yellow(), cfg.tasks[*t].description);
        }
    }
}

pub fn run_main(cli: Cli) -> miette::Result<()> {
    let command = match cli.command {
        Some(cmd) => cmd,
        None => return tui::show_selector().map_err(|e| miette::Report::new(ZettenError::Anyhow(e))),
    };

    match command {
        Command::Tasks => {
            let (root, source) = root::find_project_root().map_err(|_| ZettenError::ConfigMissing)?;
            env::set_current_dir(&root).into_diagnostic()?;
            let config = Config::load(&source).map_err(|e| ZettenError::Anyhow(e))?;
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
                tui::show_selector().map_err(|e| miette::Report::new(ZettenError::Anyhow(e)))
            } else {
                // Convert CLI Vec to HashMap for the merger
                let cli_vars: HashMap<String, String> = kv.into_iter().collect();
                let exit_code = run_tasks(tasks, workers, dry_run, args, tag, cli_vars)?;
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
                Ok(())
            }
        }
        Command::Watch { tasks } => {
            let (root, source) = root::find_project_root().map_err(|_| ZettenError::ConfigMissing)?;
            env::set_current_dir(&root).into_diagnostic()?;
            let config = Config::load(&source).map_err(|e| ZettenError::Anyhow(e))?;
            if tasks.is_empty() {
                return Err(ZettenError::TaskNotFound("No tasks specified".to_string()).into());
            }
            watch::run(&config, &tasks).map_err(|e| miette::Report::new(ZettenError::Anyhow(e)))?;
            Ok(())
        }
        Command::Doctor => doctor::run().map_err(|e| miette::Report::new(ZettenError::Anyhow(e))),
        Command::Graph => {
            let (root, source) = root::find_project_root().map_err(|_| ZettenError::ConfigMissing)?;
            env::set_current_dir(&root).into_diagnostic()?;
            graph::run(&Config::load(&source).map_err(|e| miette::Report::new(ZettenError::Anyhow(e)))?).map_err(|e| miette::Report::new(ZettenError::Anyhow(e)))
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
) -> Result<i32, ZettenError> { // Return explicit ZettenError Result
    let (root, source) =
        root::find_project_root().map_err(|_| ZettenError::ConfigMissing)?;
    env::set_current_dir(&root).map_err(ZettenError::IoError)?; // std::io::Error -> ZettenError

    let config = Arc::new(Config::load(&source).map_err(|e| ZettenError::Anyhow(e))?);
    config.validate().map_err(|e| ZettenError::Anyhow(e))?;

    // --- NEW: THREE-TIER VARIABLE MERGE ---
    let mut all_vars: HashMap<String, String> = HashMap::new();
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
            .filter(|(_, c)| matches_tag_expression(t, &c.tags))
            .map(|(n, _): (&String, _)| n.clone())
            .collect();
        root_tasks.extend(tagged);
    }

    let task_names = collect_tasks(&config, &root_tasks)?;
    crate::log::info("🔍 Validating environment...");
    if let Err(e) = validate_execution_env(&config, &task_names) {
        crate::log::user_error(&format!("Validation failed: {}", e));
        return Ok(1);
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
        return Ok(0);
    }

    let workers_count = if workers == "auto" {
        num_cpus::get()
    } else {
        workers.parse().map_err(|_| ZettenError::TaskFailed("Invalid worker count".to_string(), 1))? // Reuse existing error or just create one. Actually ParseIntError. 
        // Simplest: .map_err(|e| ZettenError::Anyhow(anyhow::anyhow!(e)))?
    };
    let is_parallel = task_names.len() > 1 && workers_count > 1;
    let mut summary = RunSummary::new();


    // --- Kahn's Algorithm Setup ---
    let mut indegree: HashMap<String, usize> = HashMap::new();
    let mut graph_map: HashMap<String, Vec<String>> = HashMap::new();
    for n in &task_names {
        indegree.insert(n.clone(), 0usize);
    }
    for n in &task_names {
        for dep in &config.tasks[n].depends_on {
            if indegree.contains_key(dep.as_str()) {
                *indegree.get_mut(n.as_str()).unwrap() += 1;
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
    let (tx, rx) = mpsc::channel::<anyhow::Result<(String, ExecutionResult, bool)>>(); // explicit anyhow::Result
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

            // SETUP PHASE
            if let Some(setup_task) = &task_cfg.setup {
                 match execute_task_command(&cfg.tasks[setup_task].resolve_cmd(&f_args, &vars), &[], false, false) {
                    Ok(r) if !r.is_success => {
                        let _ = t_tx.send(Ok((task_name.clone(), r, false)));
                        continue; // Fail early
                    }
                    Err(_) => {
                         // internal error
                         continue;
                    }
                    _ => {}
                 }
            }

            let interactive = task_cfg.interactive.unwrap_or(false);
            let res: anyhow::Result<(ExecutionResult, bool)> = (|| { // explicit anyhow
                // ... cache logic ...
                let cache_path = format!(".zetten/cache/{}.hash", task_name);
                if !task_cfg.inputs.is_empty() && f_args.is_empty() && !interactive {
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
                }
                
                let exec = execute_task_command(&final_cmd, &task_cfg.allow_exit_codes, is_parallel, interactive)?;
                
                if exec.is_success && !interactive && !task_cfg.inputs.is_empty() {
                     let _ = fs::create_dir_all(".zetten/cache");
                     let _ = fs::write(cache_path, compute_hash(&task_cfg.inputs)?);
                }
                Ok((exec, false))
            })();

            // TEARDOWN PHASE
            if let Some(teardown_task) = &task_cfg.teardown {
                // Run teardown even if main task failed
                let _ = execute_task_command(&cfg.tasks[teardown_task].resolve_cmd(&f_args, &vars), &[], false, false);
            }


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
        let (finished, exec, cached) = rx.recv()
            .map_err(|e| ZettenError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            .map_err(|e| ZettenError::Anyhow(e))?;
            
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
    print_summary(&summary, &config, &task_names);
    Ok(exit_code)
}

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

fn collect_tasks(config: &Config, roots: &[String]) -> Result<Vec<String>, ZettenError> { // Return explicit ZettenError result
    let mut expanded = HashSet::new();
    let mut stack = roots.to_vec();
    
    // First pass: Expand dependencies
    while let Some(t) = stack.pop() {
        if expanded.insert(t.clone()) {
            if let Some(c) = config.tasks.get(&t) {
                for d in &c.depends_on {
                    stack.push(d.clone());
                }
            } else {
                // Fuzzy search
                let keys: Vec<&str> = config.tasks.keys().map(|s| s.as_str()).collect();
                if let Some(closest) = find_closest(&t, keys) {
                     return Err(ZettenError::TaskNotFoundFuzzy(t, closest.to_string()));
                }
                return Err(ZettenError::TaskNotFound(t));
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
    ) -> Result<(), ZettenError> {
        if vg.contains(n) {
            return Err(ZettenError::CircularDependency(n.to_string()));
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

fn matches_tag_expression(expr: &str, tags: &[String]) -> bool {
    // supported: "ci", "ci+slow" (AND), "ci,!slow" (OR/NOT mixed is tricky without parser, let's do simple)
    // Simple logic: 
    // ',' = OR
    // '+' = AND
    // '!' = NOT
    // split by comma (OR groups)
    for group in expr.split(',') {
        // inside group, all must match (AND)
        let mut group_match = true;
        for part in group.split('+') {
            let part = part.trim();
            if let Some(negated) = part.strip_prefix('!') {
                if tags.contains(&negated.to_string()) {
                    group_match = false;
                    break;
                }
            } else {
                if !tags.contains(&part.to_string()) {
                     group_match = false;
                     break;
                }
            }
        }
        if group_match {
            return true;
        }
    }
    false
}
