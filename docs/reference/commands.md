# CLI Commands

The primary binary is `ztn`.

## `ztn run`

Execute tasks.

```bash
ztn run [TASKS]... [FLAGS]
```

### Flags
- `-w, --workers <NUM>`: Set number of parallel workers (default: auto).
- `--dry-run`: Show execution plan without running commands.
- `-t, --tag <EXPR>`: Filter tasks by tag expression (e.g., `ci+!slow`).
- `-k, --key-value <KEY>=<VAL>`: Override a configuration variable.

## `ztn watch`

Watch for file changes and re-run tasks.

```bash
ztn watch [TASKS]...
```

## `ztn init`

Initialize a new project with an interactive template selector.

## `ztn doctor`

Check environment health (Rust, Python, Config).

## `ztn completions`

Generate shell completion scripts

```bash
ztn completions <SHELL>
```
