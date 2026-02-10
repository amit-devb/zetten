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

*> Benchmarks run on macOS (Apple Silicon).*

## What Zetten Is Not

To understand Zetten, it is helpful to know what it is **not**:
- It is **not** a framework or a workflow engine.
- It is **not** a plugin system or a background service.
- It is **not** a runtime dependency of your application.

It is a small, fast, and predictable execution tool accessed via the `ztn` command.