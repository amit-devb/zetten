# Quickstart

Getting started with Zetten takes less than a minute.

## 1. Initialize your project
Run the following command in your project root to create a base configuration:

```bash
zetten init
```

---


## 2. Define your first task
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
zetten run lint
```

