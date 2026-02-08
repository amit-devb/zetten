# Configuration

Configuration in Zetten is explicit by design. There is no templating and no conditionals.

## Supported Files

### `pyproject.toml` (Preferred)
```toml
[tool.zetten.tasks.test]
cmd = "pytest"
inputs = ["src/", "tests/"]
```

### `zetten.toml` (Option)
```toml
[tasks.test]
cmd = "pytest"
inputs = ["src/", "tests/"]
```
---

## Resolution Rules
Zetten follows a strict logic to find your settings. If the configuration is missing or ambiguous, Zetten will fail with an explanation rather than trying to guess.

- Check pyproject.toml: If it exists and contains a [tool.zetten] header, Zztn uses it.
- Check zetten.toml: If pyproject.toml doesn't have Zetten settings, it looks for this file.
- Initialization: If neither is found, the project is considered uninitialized.

**To fix a missing configuration, run:**
```bash
ztn init
```

### Example: Defining a Task
A task consists of three main parts:
- `cmd`: The actual shell command to run (e.g., pytest or ruff).
- `inputs`: A list of files or directories Zetten should "watch." If these don't change, Zetten skips the run.
- `depends_on` (Optional): Other tasks that must finish before this one starts.
- `tags` (Optional): Optional labels to group tasks (e.g., `["ci"]`).
- `description` (Optional): Optional field to describe a task
- `hints` (Options): A field to help developer set some hints when the user encouters any issues.

Full Example in pyproject.toml:
```toml
[tool.zetten.tasks.lint]
cmd = "ruff check src"
inputs = ["src/"]

[tool.zetten.tasks.test]
cmd = "pytest"
inputs = ["src/", "tests/"]
depends_on = ["lint"]
```

---


### Pro-Tip: Debugging Variables
If you aren't sure what value a variable is taking, use the Doctor command:
```bash
zetten doctor
```