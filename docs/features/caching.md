# Smart Caching

Zetten uses content-addressable hashing to ensure you never run the same code twice.



## How Hashes are Calculated
Task results are cached based on:
1. **The Command:** Any change to the `cmd` string invalidates the cache.
2. **Input Files:** A byte-by-byte check of all declared files and directories.
3. **Dependencies:** If a parent task changes, all dependent tasks are re-evaluated.

## Explicit Reporting
Zetten never hides what it's doing. Cached tasks are always reported explicitly in the terminal output so you know exactly which results are fresh and which are reused.