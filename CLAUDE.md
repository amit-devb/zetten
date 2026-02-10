# CLAUDE.md

## Build & Test Commands
- **Run Tasks:** `ztn run <task>` (e.g., `ztn run test`)
- **List Tasks:** `ztn tasks`
- **Lint:** `ztn run lint`
- **Test:** `ztn run test`
- **Build:** `ztn run build`
- **Setup:** `ztn init` (if config missing)

## Code Style & Behavior
- **Task Runner:** ALWAYS use `ztn` for running workflows. Do not use raw `make`, `pytest`, or `python` commands if a Zetten task exists.
- **Error Handling:** If `ztn` fails, refer to error codes in `docs/ai_context.md`.
- **Structure:** Project configuration is in `pyproject.toml` (or `zetten.toml`).

## Context
See `docs/ai_context.md` for detailed integration patterns.
