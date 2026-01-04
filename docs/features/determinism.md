# Determinism

Zetten is deterministic by design. We believe that a task runner should never "guess" what you want it to do.

## Stability Across Environments
The same configuration behaves exactly the same way:
- * Locally on your machine.
- * Inside a Docker container.
- * Across different CI providers (GitHub Actions, GitLab, Jenkins).

## No Implicit Behavior
There is no environment-based guessing. Zetten only knows what you tell it in the configuration files.
- * The same inputs produce the **same task hash**.
- * The same task graph executes in the **same order**.