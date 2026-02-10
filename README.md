# ⚡ Zetten

**The High-Performance Task Runner for Python Backends.** *Parallel. Deterministic. Fast.*

[![PyPI - Version](https://img.shields.io/pypi/v/zetten?color=orange&label=pypi)](https://pypi.org/project/zetten/)
[![PyPI - License](https://img.shields.io/pypi/l/zetten?color=brightgreen&label=license)](https://github.com/amit-devb/zetten/blob/main/LICENSE)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/zetten.svg)](https://pypi.org/project/zetten/)
[![Nightly Build](https://github.com/amit-devb/zetten/actions/workflows/nightly.yml/badge.svg)](https://github.com/amit-devb/zetten/actions/workflows/nightly.yml)

Zetten is a dependency-aware execution engine designed to unify how you run tests, linters, and builds. It ensures that your workflow remains identical across local development environments and any CI platform, only faster.

---

## 🚀 The Zetten Philosophy

Modern Python projects often require coordinating various tools (tests, type-checkers, formatters). Zetten eliminates "Glue Code Fatigue" by providing:

* **Parallel Execution:** Automatically identifies independent tasks and runs them concurrently across your CPU cores.
* **Three-Tier Variable System:** Advanced command templating with a strict priority hierarchy: CLI Flags (-k) > Config Vars > Environment Variables.
* **Smart Caching:** Uses content-addressable hashing to skip tasks if their specific inputs haven't changed since the last run.
* **Platform Agnostic:** Behaves identically on macOS, Windows, Linux, or any CI/CD provider.
* **Dependency Awareness:** Define a Directed Acyclic Graph (DAG) of tasks to ensure correct execution order (e.g., `setup` always precedes `test`).

---

## ✨ Features

- **⚡ Worker Pool Concurrency:** Maximizes resource usage by running non-dependent tasks in parallel.
- **🏷️ CI Tagging:** Execute logical groups of tasks (e.g., `run --tag ci`) with a single command.
- **🛡️ Failure Propagation:** If a foundational task fails, Zetten halts downstream execution to prevent cascading errors.
- **🔍 Intelligent Diagnostics:** Includes `ztn doctor` to identify environment inconsistencies instantly.
- **⏱️ Performance Analytics:** (Coming Soon) Real-time insights into time saved via parallelism.

---

---

## 🏎️ Performance

Zetten is built for speed. Benchmarks against popular task runners show it provides the fastest developer experience for incremental builds.

| Metric | Tool | Time | vs Zetten |
| :--- | :--- | :--- | :--- |
| **Startup** | **`ztn`** | **2.08 ms** | **1.00x** |
| *(CLI overhead)* | `just` | 2.15 ms | 1.04x |
| | `make` | 3.85 ms | 1.85x |
| | `poe` | 41.88 ms | 20.15x |
| | | | |
| **Smart Caching** | **`ztn`** | **3.49 ms** | **1.00x** |
| *(No-op re-run)* | `just` | 4.26 ms | 1.22x |
| | `make` | 5.63 ms | 1.61x |
| | `poe` | 64.68 ms | 18.52x |

*> Benchmarks run on macOS (Apple Silicon). Startup measures `tool --version`. Smart Caching measures time to detect no input changes and skip execution.*

---

## 🛠️ Quick Start
Install Zetten:

```bash
pip install zetten
```

Initiate a project:

```bash
ztn init
```

For Python script execution:
```toml
[tool.zetten.tasks.hello]
script = "my_module:main"  # Runs: python -c "import my_module; my_module.main()"
description = "Run a python function"
```

Define tasks in pyproject.toml:
```toml
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
```toml
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
ztn run test
ztn run lint test
```
Zetten will only re-run tasks when their inputs change.

---

## ⚙️ The Variable Hierarchy
Zetten uses a deterministic three-tier system to resolve variables:
- Tier 1 (CLI): `ztn run build -k build_dir=output` (Highest Priority)
- Tier 2 (Config): Values defined in `[tool.zetten.vars]`
- Tier 3 (Env): System environment variables (e.g., `$USER`, `$PATH`)

---

## 🚀 Running in CI
Zetten is designed for the modern CI/CD pipeline. By using Tags and Strict Mode, you can ensure your pipeline is both flexible and safe.
```bash
# Force a specific version and environment in CI
ztn run --tag ci -k VERSION=${GITHUB_SHA} -k ENV=prod
```

If a foundational task fails, Zetten halts downstream execution immediately to save CI minutes and prevent cascading failures.


## ⚙️ Configuration Model
Configuration is explicit by design:
- No templating
- No conditionals
- No implicit behavior

Configuration lives in:
- `pyproject.toml` (preferred)
- `zetten.toml` (for legacy or minimal projects)

If no configuration is found, Zetten will explain how to resolve the issue.

---


## 🛠 Commands
- `ztn run <tasks>` — Execute tasks with parallel dependency resolution.
- `ztn run <task> -k KEY=VAL` — Override any variable via the CLI.
- `ztn watch <tasks>` — Precision re-runs on input changes.
- `ztn graph` — Visualizes the Directed Acyclic Graph (DAG) of your tasks.
- `ztn doctor` — Diagnoses configuration and environmental health issues.
- `ztn init` — Interactive project setup and template generation.

---

## 🛡 Status
Zetten is currently in **v1.3.2**. If no configuration file is found, Zetten will provide clear instructions on how to initialize your project.

---

## Documentation
Full documentation is available at: [docs.zetten.in](https://docs.zetten.in)

---

## 🤝 Contributing
We love Rust and Python! If you want to help make Zetten even faster:
- Fork the repo.
- Add your feature (don't forget the tests!).
- Open a Pull Request.

Built with ❤️ for the Python community using the speed of Rust.
  
Please open an issue or discussion on GitHub before proposing large changes.
