# ⚡ Zetten

**The High-Performance Task Runner for Python Backends.** *Parallel. Deterministic. Fast.*

[![PyPI - Version](https://img.shields.io/pypi/v/zetten?color=orange&label=pypi)](https://pypi.org/project/zetten/)
[![PyPI - License](https://img.shields.io/pypi/l/zetten?color=brightgreen&label=license)](https://github.com/amit-devb/zetten/blob/main/LICENSE)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/zetten?color=blue&label=python)](https://pypi.org/project/zetten/)
[![CI Status](https://img.shields.io/github/check-runs/amit-devb/zetten/main?label=CI&logo=github)](https://github.com/amit-devb/zetten/actions)

Zetten is a dependency-aware execution engine designed to unify how you run tests, linters, and builds. It ensures that your workflow remains identical across local development environments and any CI platform, only faster.

---

## 🚀 The Zetten Philosophy

Modern Python projects often require coordinating various tools (tests, type-checkers, formatters). Zetten eliminates "Glue Code Fatigue" by providing:

* **Parallel Execution:** Automatically identifies independent tasks and runs them concurrently across your CPU cores.
* **Three-Tier Variable:** System: Advanced command templating with a strict priority hierarchy: CLI Flags (-k) > Config Vars > Environment Variables.
* **Smart Caching:** Uses content-addressable hashing to skip tasks if their specific inputs haven't changed since the last run.
* **Platform Agnostic:** Behaves identically on macOS, Windows, Linux, or any CI/CD provider.
* **Dependency Awareness:** Define a Directed Acyclic Graph (DAG) of tasks to ensure correct execution order (e.g., `setup` always precedes `test`).

---

## ✨ Features

- **⚡ Worker Pool Concurrency:** Maximizes resource usage by running non-dependent tasks in parallel.
- **🏷️ CI Tagging:** Execute logical groups of tasks (e.g., `run --tag ci`) with a single command.
- **🛡️ Failure Propagation:** If a foundational task fails, Zetten halts downstream execution to prevent cascading errors.
- **🔍 Intelligent Diagnostics:** Includes `zetten doctor` to identify environment inconsistencies instantly.
- **⏱️ Performance Analytics:** (Coming Soon) Real-time insights into time saved via parallelism.

---

## 🛠️ Quick Start
Install Zetten:

```bash
pip install zetten
```

Initiate a project:

```bash
zetten init
```

Define tasks in pyproject.toml:
```bash
[tool.zetten.tasks.lint]
cmd = "ruff check src"
inputs = ["src/"]
tags = ["ci"]

[tool.zetten.tasks.test]
cmd = "pytest"
depends_on = ["lint"]
inputs = ["src/", "tests/"]
tags = ["ci"]

[tool.zetten.tasks.build]
description = "Build the project"
# Supports Fallback Syntax: ${VAR:-default}
cmd = "mkdir -p ${build_dir} && python -m build --outdir ${DEST:-dist}"
depends_on = ["lint"]
inputs = ["src/"]
```

Define tasks in zetten.toml:
```bash
[tasks.setup]
cmd = "pip install -r requirements.txt"

[tasks.lint]
cmd = "ruff check src"
inputs = ["src/"]
tags = ["ci"]

[tasks.test]
cmd = "pytest"
depends_on = ["setup"]
inputs = ["src/", "tests/"]
tags = ["ci"]
```

Run tasks:
```bash
zetten run test
zetten run lint test
```
Zetten will only re-run tasks when their inputs change.

---

## ⚙️ The Variable Hierarchy
Zetten uses a deterministic three-tier system to resolve variables:
- Tier 1 (CLI): zetten run build -k build_dir=output (Highest Priority)
- Tier 2 (Config): Values defined in [tool.zetten.vars]
- Tier 3 (Env): System environment variables (e.g., $USER, $PATH)

---

## 🚀 Running in CI
Zetten is designed for the modern CI/CD pipeline. By using Tags and Strict Mode, you can ensure your pipeline is both flexible and safe.
```bash
# Force a specific version and environment in CI
zetten run --tag ci -k VERSION=${GITHUB_SHA} -k ENV=prod
```

If a foundational task fails, Zetten halts downstream execution immediately to save CI minutes and prevent cascading failures.


## ⚙️ Configuration Model
Configuration is explicit by design:
- No templating
- No conditionals
- No implicit behavior

Configuration lives in:
- pyproject.toml (preferred)
- zetten.toml (for legacy or minimal projects)

If no configuration is found, Zetten will explain how to resolve the issue.

---


## 🛠 Commands
- zetten run <tasks> — Execute tasks with parallel dependency resolution.
- zetten run <task> -k KEY=VAL — Override any variable via the CLI.
- zetten watch <tasks> — Precision re-runs on input changes.
- zetten graph — Visualizes the Directed Acyclic Graph (DAG) of your tasks.
- zetten doctor — Diagnoses configuration and environmental health issues.
- zetten init — Interactive project setup and template generation.

---

## 🛡 Status
Zetten is currently in v0.1. If no configuration file is found, Zetten will provide clear instructions on how to initialize your project.

---

## Documentation
Full documentation is available at: [Github Wiki](https://github.com/amit-devb/zetten/wiki)

---

## 🤝 Contributing
We love Rust and Python! If you want to help make Zetten even faster:
- Fork the repo.
- Add your feature (don't forget the tests!).
- Open a Pull Request.

Built with ❤️ for the Python community using the speed of Rust.
  
Please open an issue or discussion on GitHub before proposing large changes.
