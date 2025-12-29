# Zetten
Zetten is a fast, deterministic task runner for Python backend projects, written in Rust.
It provides a reliable way to run common backend tasks-tests, linters, type checks, builds-using a single execution engine that behaves the same locally and in CI.
Zetten is inspired by tools like make, nox, just, and cargo, but is designed specifically for modern Python workflows.

---

## Why Zetten
Python backend projects often rely on a mix of:
- Makefiles
- shell scripts
- tox / nox
- CI YAML logic

These approaches are flexible, but they are often slow, inconsistent, and hard to reason about at scale.

Zetten focuses on a small set of guarantees:
- The same inputs always produce the same results
- Tasks only run when they need to
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

## Example Usage
```bash
zetten run test
zetten run lint typecheck

## Zetten uses pyproject.toml for configuration.
[tool.zetten.tasks.test]
cmd = "pytest"
inputs = ["src/", "tests/"]

[tool.zetten.tasks.lint]
cmd = "ruff check src"
inputs = ["src/"]
```
---

## Configuration is explicit by design:
- No templating
- No conditionals
- No implicit behavior

---

## Installation

```bash
pip install zetten
```

---


### Contributing and Feedback
Feedback from Python backend developers is welcome, especially around:
- Developer experience
- CI usage
- Installation friction
- Missing but essential workflows
Please open issues or discussions on GitHub.
```bash

---

If you want next, I strongly recommend one of these (in order of impact):

1. **Add a 60-second Quickstart section** at the top
2. Add a **“Zetten vs Make / Nox / Just”** comparison
3. Create a **minimal FastAPI example repo**
4. Tighten this README for a **launch/announcement version**



