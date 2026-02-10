# Introduction

# Introduction

**Zetten** is a high-performance, deterministic execution engine for Python backend projects. Built in Rust, it acts as a focused task runner designed to unify how you run tests, linters, and builds.

Zetten ensures that your workflow remains identical across local development environments and any CI platform—only faster.

## The Mental Model

Think of Zetten as an **execution engine**, not a framework. Unlike tools that rely on implicit behavior, Zetten requires explicit declarations:

* **Tasks are explicit:** You define exactly what to run.
* **Inputs are declared:** You tell Zetten which files matter.
* **Execution is deterministic:** The same configuration behaves the same everywhere.
* **Results are cached:** If inputs haven't changed, the task is skipped.
* **Output is CI-safe:** Logs and exit codes are designed for automation.

[Get Started](quickstart.md)

## What Zetten Is Not

To understand Zetten, it is helpful to know what it is **not**:
- It is **not** a framework or a workflow engine.
- It is **not** a plugin system or a background service.
- It is **not** a runtime dependency of your application.

It is a small, fast, and predictable execution tool accessed via the `ztn` command.