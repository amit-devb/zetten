# Advanced Tagging

Tags allow you to filter which tasks to run. Zetten supports boolean logic for powerful selection.

## defining Tags

```toml
[tasks.lint]
tags = ["ci", "fast"]

[tasks.test_unit]
tags = ["ci", "fast"]

[tasks.test_e2e]
tags = ["ci", "slow"]
```

## Filtering via CLI

Use the `--tag` (or `-t`) flag.

### Basic Selection
Run all tasks with the `ci` tag:
```bash
ztn run --tag ci
```

### AND Logic (`+`)
Run tasks that have **BOTH** `ci` and `fast`:
```bash
ztn run --tag "ci+fast"
```

### NOT Logic (`!`)
Run tasks that have `ci` but **NOT** `slow`:
```bash
ztn run --tag "ci+!slow"
```

### OR Logic (Comma)
Run tasks that match `fast` **OR** `slow`:
```bash
ztn run --tag "fast,slow"
```

### Complex Combinations
Run (CI and Fast) OR (Manual):
```bash
ztn run --tag "ci+fast,manual"
```
