# Zetten
Zetten is a fast, deterministic task runner for Python backend projects, written in Rust.
It provides a reliable way to run common backend tasks - tests, linters, type checks, builds - using a single execution engine that behaves the same locally and in CI.
Zetten replaces ad-hoc setups built from Makefiles, shell scripts, nox/tox, and CI YAML glue with a small, explicit, and predictable tool.
Zetten is inspired by tools like make, nox, just, and cargo, but is designed specifically for modern Python backend workflows.

---

## Why Zetten
Python backend projects often rely on a mix of:
- Makefiles
- shell scripts
- tox / nox
- CI YAML logic

These approaches are flexible, but they are often:
- non-deterministic
- slow as projects grow
- hard to reason about
- inconsistent between local runs and CI

Zetten focuses on a small set of guarantees:
- The same inputs always produce the same results
- Tasks only run when their inputs change
- Independent tasks run in parallel
- Local and CI execution behave identically

---

## Features
- Fast execution using a Rust-based core
- Deterministic task caching based on input hashing
- Python virtual environment awareness
- Automatic environment detection without manual activation
- Parallel task execution using a worker pool
- Task dependency execution using a DAG model
- Structured logging and progress reporting
- Clear and consistent exit code semantics

---

## What Zetten Is (and Is Not)
### Zetten is
- A task runner for Python backend projects
- A local development and CI execution tool
- Deterministic and cache-aware
- CLI-first
### Zetten is not
- A framework
- A workflow engine
- A job queue
- A replacement for linters or test frameworks
- A runtime dependency of your application

---

## Quick Start
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
[tool.zetten.tasks.test]
cmd = "pytest"
inputs = ["src/", "tests/"]

[tool.zetten.tasks.lint]
cmd = "ruff check src"
inputs = ["src/"]
```

Define tasks in zetten.toml:
```bash
[tasks.test]
cmd = "pytest"
inputs = ["src/", "tests/"]

[tasks.lint]
cmd = "ruff check src"
inputs = ["src/"]
```

Run tasks:
```bash
zetten run test
zetten run lint test
```
Zetten will only re-run tasks when their inputs change.

---

## Configuration Model
Configuration is explicit by design:
- No templating
- No conditionals
- No implicit behavior

Configuration lives in:
- pyproject.toml (preferred)
- zetten.toml (for legacy or minimal projects)

If no configuration is found, Zetten will explain how to fix it.

---


### Commands
- zetten run <tasks> — execute tasks deterministically
- zetten watch <tasks> — re-run tasks on input changes
- zetten graph — inspect the task dependency graph
- zetten doctor — diagnose configuration and environment issues
- All commands produce stable, CI-safe output with well-defined exit codes.

---

## Status
Zetten is in v0.1 and is feature-complete for its initial design.
Core semantics - execution, caching, exit codes, configuration - are stable.

---

## Documentation
Full documentation is available at: [Github Wiki](https://github.com/amit-devb/zetten/wiki)

---

## Contributing and Feedback
Zetten is intentionally minimal and opinionated.
Feedback is welcome, especially around:
- Developer experience
- CI usage
- Installation friction
- Missing but essential workflows.
  
Please open an issue or discussion on GitHub before proposing large changes.
