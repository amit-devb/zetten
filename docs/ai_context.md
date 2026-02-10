# Zetten Context for AI Agents

> **Role:** You are an AI Assistant helping a developer work on this project.
> **Tool:** The project uses `zetten` (binary: `ztn`) as its primary task runner.

## ⚡ What is Zetten?
Zetten is a language-agnostic task runner that executes commands defined in `pyproject.toml` (or `zetten.toml`). It handles:
- **Dependency Management:** Tasks run in the correct order (DAG).
- **Parallelism:** Independent tasks run concurrently.
- **Caching:** Tasks are skipped if their inputs haven't changed.

## 🛠 How to Run Tasks
**Do NOT run raw commands** like `pytest`, `ruff`, or `python main.py` directly unless initialized. Instead, ask Zetten to run them.

### 1. Discovery
First, list the available tasks to understand the project's capabilities:
```bash
ztn tasks
```

### 2. Execution
Run a specific task:
```bash
ztn run <task_name>
```

Run multiple tasks (Zetten optimizes the order):
```bash
ztn run lint test build
```

### 3. Debugging
If a task fails, Zetten provides a specific error code.
- If you see `ztn::config::missing`, suggest running `ztn init`.
- If you see `ztn::graph::cycle`, check `depends_on` in the config.
- Use `ztn doctor` to check for environment issues.

## 🧠 AI "Skills" / Common Patterns

### 1. The "Safe Change" Pattern
When asked to refactor or fix code:
1. Run `ztn run lint` *before* starting to ensure a clean state.
2. Make your changes.
3. Run `ztn run test` to verify.

### 2. The "CI Simulation" Pattern
To guarantee your changes will pass CI:
```bash
ztn run --tag ci
```
This runs the exact subset of tasks defined for the CI pipeline.

### 3. Dependency Analysis
If you are unsure why a task is running (or not running), inspect the graph:
```bash
ztn run <task> --dry-run
```

**Example:**
```toml
[tool.zetten.tasks.test]
cmd = "pytest"
inputs = ["tests/", "src/"]  # Cache invalidation key
depends_on = ["setup"]       # Dependency
tags = ["ci"]                # Grouping
```

**Variables:**
Zetten resolves variables in this order:
1. CLI: `ztn run task -k KEY=val`
2. Config: `[tool.zetten.vars]`
3. Environment: `os.environ`

When modifying the project, prefer adding new tasks to `pyproject.toml` rather than creating shell scripts.
