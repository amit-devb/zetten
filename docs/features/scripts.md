# Python Script Execution

Zetten allows you to run Python functions directly, bypassing the need for separate shell scripts or `python -m` commands.

## Configuration

Use the `script` key in your `[tasks]` definition.

```toml
[tasks.hello]
description = "Run a greeting function"
script = "my_module:greet"
```

This is equivalent to running:
```bash
python -c "import my_module; my_module.greet()"
```

## Inline Modules

You can also run modules directly:

```toml
[tasks.server]
script = "http.server"
```
Equivalent to: `python -m http.server`

## Why use `script` over `cmd`?

1.  **Cross-Platform**: no need to worry about `/` vs `\` or shell differences.
2.  **Performance**: slightly faster startup as it avoids shell process overhead.
3.  **Cleanliness**: keeps your config focused on Python logic.
