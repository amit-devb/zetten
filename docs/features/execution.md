# Execution Model

Zetten utilizes a high-performance worker pool to maximize your hardware's potential.

## Parallel Execution
Zetten builds a **Directed Acyclic Graph (DAG)** of your tasks. 
- * Independent tasks (e.g., Linting and Type Checking) run in **parallel** across available CPU cores.
- * Dependent tasks (e.g., Testing) wait until their requirements are satisfied.

## Dependency Detection
If Zetten detects a cycle in your dependencies (e.g., Task A depends on B, and B depends on A), it will fail immediately with a clear error message rather than entering an infinite loop.