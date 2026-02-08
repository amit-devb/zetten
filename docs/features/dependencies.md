# Enhanced Dependency Management

Zetten provides robust hooks for managing task lifecycles, ensuring your environment is always in a known state.

## Setup & Teardown

You can define `setup` and `teardown` tasks for any task.

```toml
[tasks.db_test]
cmd = "pytest tests/db"
setup = "db_init"
teardown = "db_clean"
```

### Behavior

1.  **Setup (`db_init`)**: Runs *before* `db_test`. If `setup` fails, `db_test` is skipped.
2.  **Main Task (`db_test`)**: Runs only if `setup` succeeds.
3.  **Teardown (`db_clean`)**: Runs *after* `db_test` completes, **regardless of success or failure**.

This is crucial for cleanup tasks like dropping test databases or removing temporary files.

## DAG Dependencies

Standard dependencies still apply via `depends_on`:

```toml
[tasks.test]
depends_on = ["lint", "build"]
```

Zetten guarantees `lint` and `build` finish successfully before `test` starts.
