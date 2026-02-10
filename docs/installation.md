# Installation

# Installation

Zetten is distributed as a single, self-contained binary written in Rust, but it is easily installable via common Python package managers.

---

## Install with pip

The easiest way to install Zetten is via `pip`:

```bash
pip install zetten
```

## Install with uv

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
ztn --help
```

---

## Requirements

- **Python-Aware:** Zetten detects Python environments automatically but does not depend on a specific Python version to run.
- **No Virtual Env Required:** You do not need to activate a virtual environment to use the `ztn` binary itself (though your tasks might run commands inside one).
- **OS Support:** Native support for Linux, macOS, and Windows.