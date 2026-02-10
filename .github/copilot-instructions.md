# GitHub Copilot Instructions for Zetten Projects

This project uses **Zetten (`ztn`)** as its primary task runner.

## Core Rules
1. **Always use `ztn run <task>`** instead of direct commands like `pytest`, `ruff`, or `python main.py`.
2. **Never execute shell scripts** if a corresponding task exists in `pyproject.toml`.
3. **If a task fails**, refer to `docs/ai_context.md` for error code meanings.

## Context
Please read `docs/ai_context.md` to understand the project's specific task graph and available commands.

## Key Commands
- `ztn tasks` - List all available tasks
- `ztn run <task>` - Execute a task (e.g., `ztn run test`)
- `ztn init` - Initialize configuration
