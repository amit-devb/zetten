# Benchmarks

We believe in transparency. Benchmarks are often biased, so we provide the exact methodology and scripts used to produce these numbers. You can run them yourself using the code in the `benchmarks/` directory of this repository.

## 📊 Summary Results

| Metric | Zetten | Just | Make | PoeThePoet |
| :--- | :--- | :--- | :--- | :--- |
| **Startup Time** | **2.08 ms** | 2.15 ms | 3.85 ms | 41.88 ms |
| **No-Op (Cached)** | **3.49 ms** | 4.26 ms | 5.63 ms | 64.68 ms |
| **Cold Build** | **60 ms*** | 4 ms | 6 ms | 66 ms |
| **Binary Size** | **2.9 MB** | 3.6 MB | 0.1 MB | N/A (Python) |
| **Peak Memory** | **~9 MB** | ~9 MB | ~9 MB | ~27 MB |

*> Note on Cold Build: Zetten performs content-hashing on all input files even during a cold build to populate the cache. Tools like `make` and `just` do not check content hashes by default, explaining their speed advantage in this specific "blind run" scenario.*

---

## 🔬 Methodology

Our benchmarks are automated using `hyperfine`, a command-line benchmarking tool.

### Environment
- **Hardware:** Apple Silicon (M1 Pro) / Ubuntu Latest (CI)
- **Tool Versions:** Latest stable releases as of Feb 2026.
- **Python:** 3.10+

### Tools Used
1. **Hyperfine**: For statistical precision (warmup runs, outlier detection).
2. **Time**: For measuring peak memory (RSS).
3. **wc/du**: For binary size analysis.

### Scenarios

#### 1. Startup Time (`--version`)
Measures the overhead of the CLI itself.
- **Command:** `tool --version`
- **Why it matters:** Developer experience. Laggy CLIs break flow state.

#### 2. No-Op (Cached Build)
Measures the time to determine "nothing needs to be done".
- **Command:** `tool run build` (where sources are unchanged)
- **Why it matters:** 90% of developer builds are incremental. Zetten uses content hashing; others often use file modification timestamps (mtime).

#### 3. Cold Build
Measures a fresh build from scratch.
- **Command:** `tool run build` (after `clean`)
- **Why it matters:** CI/CD pipelines often start fresh.

---

## 🏃 Run the Benchmarks Yourself

Clone the repository and run the included runner:

```bash
# 1. Install dependencies
pip install hyperfine

# 2. Run the benchmark suite
python3 benchmarks/runner.py --scenario all
```
