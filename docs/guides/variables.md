# Variable Hints & Interpolation

Zztn watchures a powerful, deterministic variable system. It allows you to write one command that adapts to different environments without changing the configuration file.

## Syntax
In your cmd strings, you can reference variables using the standard shell-like syntax:
- ${VAR}: Resolves to the value of VAR.
- ${VAR:-default}: Resolves to VAR, or uses default if VAR is not set.

Example in `pyproject.toml`:
```toml
[tool.zetten.tasks.build]
# If DEST is not provided, it defaults to 'dist'
cmd = "python -m build --outdir ${DEST:-dist}"
inputs = ["src/"]
```

---

## The Hierarchy (Resolution Order)
Zetten resolves variables using a "Strict Tier" system. If a variable is defined in multiple places, the higher tier always wins.

#### Tier 1: CLI Overrides (Highest):
Values passed directly via the -k (or --key) flag. This is used for "one-off" changes.
```bash
ztn run build -k DEST=build_output
```

#### Tier 2: Config File
Values defined globally in your configuration file under the vars table.
```toml
[tool.zetten.vars]
DEST = "local_cache"
```


#### Tier 3: Environment Variables (Lowest)
Standard system environment variables (e.g., $USER, $PATH, or variables exported in your shell).
```bash 
export DEST=system_dist
ztn run build
```

---
### Global Variables (vars)
Zetten allows you to define a global vars table. These variables act as "Defaults"—they will be injected into any task's cmd that references them, unless you override them via the CLI.

#### Example: `pyproject.toml`

Place your global variables under the `[tool.zetten.vars]` section.
```toml
[tool.zetten.vars]
BUILD_DIR = "dist"
PYTHON_BIN = "python3"
LOG_LEVEL = "info"

[tool.zetten.tasks.build]
# Uses the global BUILD_DIR and PYTHON_BIN defined above
cmd = "${PYTHON_BIN} -m build --outdir ${BUILD_DIR}"
inputs = ["src/"]
```

#### Example: `zetten.toml`
If using a standalone file, use the [vars] header.
```toml
[vars]
BUILD_DIR = "dist"

[tasks.build]
cmd = "mkdir -p ${BUILD_DIR} && tar -cvf ${BUILD_DIR}/pkg.tar src/"
inputs = ["src/"]
```
