# Quickstart

# Quickstart

Getting started with Zetten takes less than a minute.

## 1. Configure a Task

Open your `pyproject.toml` and add a task (e.g., for your linter):
```toml
[tool.zetten.tasks.lint]
cmd = "ruff check src"
inputs = ["src/"]
```

---

## 2. Run the Task

Execute the task using the `run` command:
```bash
ztn run lint
```

