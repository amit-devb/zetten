# Zetten ⚡

Zetten is a fast, deterministic task runner for Python backend projects,
written in Rust.

It is inspired by tools like `make`, `nox`, `just`, and `cargo`,
but designed specifically for modern Python workflows.

---

## Features

- 🚀 Fast execution (Rust)
- 🔁 Deterministic caching (input hashing)
- 🐍 Python virtualenv awareness
- ⚙️ Parallel execution with worker pool
- 🧠 Task dependencies (DAG)
- 📊 Structured logging and progress tracking
- 🧪 Custom exit-code semantics

---

## Installation

### From source (recommended for now)

```bash
cargo install --path .
