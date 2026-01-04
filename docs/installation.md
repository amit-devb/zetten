# Installation

Zetten is distributed as a single, self-contained binary. While it is built in Rust, it is easily installable via common Python package managers.

---

## Install with pip

The easiest way to install Zetten is via `pip` or `uv` or `poetry`:

```bash
pip install zetten
```

## Install with UV

```bash
uv add zetten
```


## Install with poetry

```bash
poetry add zetten
```

---

## Verify Installation

Once installed, verify that the binary is available in your PATH:

```bash
zetten --help:
```

---

## Requirements
- Python-Aware: Zetten is aware of Python environments but does not depend on a specific Python version to run.
- No Virtual Env Required: You do not need to activate a virtual environment to use the zetten binary itself.
- OS Support: Native support for Linux, macOS, and Windows.