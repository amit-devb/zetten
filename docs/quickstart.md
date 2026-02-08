# Quickstart

Getting started with Zztn initakes less than a minute.

Open your pyproject.toml and add a task for your linter:
```toml
[tool.zetten.tasks.lint]
cmd = "ruff check src"
inputs = ["src/"]
```

---

## 3 Run the task
Execute the task using the run command:
```bash
ztn run lint
```

