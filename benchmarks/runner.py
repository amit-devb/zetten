
import subprocess
import json
import os
import sys
import argparse
import shutil

# --- Configuration ---
WARMUP_RUNS = 3
BENCHMARK_RUNS = 10

def get_project_root():
    # Assumes run from <root>/benchmarks/ or <root>/
    cwd = os.getcwd()
    if os.path.basename(cwd) == "benchmarks":
        return os.path.dirname(cwd)
    return cwd

def resolve_tools(root):
    tools = {}
    
    # 1. Zetten (Build Release)
    ztn_bin = os.path.join(root, "target", "release", "ztn")
    if not os.path.exists(ztn_bin):
        print(" Building Zetten release binary...", file=sys.stderr)
        subprocess.run(["cargo", "build", "--release", "--quiet"], check=True, cwd=root)
    tools["ztn"] = ztn_bin

    # 2. Make
    tools["make"] = shutil.which("make") or "make"

    # 3. Just
    tools["just"] = shutil.which("just") or "just"

    # 4. Poe (via venv)
    venv_python = os.path.join(root, "benchmarks", "venv", "bin", "python")
    if os.path.exists(venv_python):
        tools["poe"] = f"{venv_python} -m poethepoet"
    else:
        tools["poe"] = None # Disable if not found

    return tools

def run_hyperfine(name, commands, output_file, prepare_cmd=None):
    print(f"--- Benchmarking: {name} ---")
    cmd = [
        "hyperfine",
        "--warmup", str(WARMUP_RUNS),
        "--runs", str(BENCHMARK_RUNS),
        "--export-json", output_file,
    ]
    
    if prepare_cmd:
        cmd.extend(["--prepare", prepare_cmd])
        
    cmd.extend(commands)
    
    try:
        subprocess.run(cmd, check=True)
    except subprocess.CalledProcessError:
        print(f"Benchmark {name} failed.", file=sys.stderr)

def print_summary(results_file):
    if not os.path.exists(results_file):
        return

    with open(results_file) as f:
        data = json.load(f)
        
    print(f"\nSummary for {os.path.basename(results_file)}:")
    # Sort by mean time
    results = sorted(data["results"], key=lambda x: x["mean"])
    baseline = results[0]["mean"]
    
    print(f"{'Command':<10} | {'Time':<20} | {'Ratio':<6} | {'Peak Mem':<10}")
    print("-" * 55)

    for res in results:
        cmd_name = res["command"].split()[0].split("/")[-1] # Try to get short name
        if "python" in cmd_name: cmd_name = "poe" 
        
        mean = res["mean"] * 1000 # ms
        stddev = res["stddev"] * 1000 # ms
        ratio = res["mean"] / baseline
        
        # Hyperfine memory is in bytes (on macOS) or KB (Linux normalized by hyperfine? Let's assume bytes from JSON)
        mem_str = "N/A"
        # Check if memory metrics are available
        # Note: 'max' in JSON is max TIME. We need 'memory_usage_byte' list or similar if hyperfine provides it.
        # Hyperfine 1.18+ provides 'memory_usage_byte' as a list of peaks per run.
        if "memory_usage_byte" in res and res["memory_usage_byte"]:
            # We'll report the maximum peak observed across all runs
            peak_bytes = max(res["memory_usage_byte"])
            if peak_bytes > 0:
                 mem_mb = peak_bytes / (1024 * 1024)
                 mem_str = f"{mem_mb:.2f} MB"
        
        print(f"{cmd_name:<10} | {mean:.2f} ms ± {stddev:.2f} | {ratio:.2f}x | {mem_str:<10}")
    print("-" * 55)

def main():
    parser = argparse.ArgumentParser(description="Zetten Benchmark Runner")
    parser.add_argument("--scenario", choices=["startup", "noop", "warm_cache", "cold_build", "all"], default="all")
    parser.add_argument("--tools", nargs="+", default=["ztn", "just", "make", "poe"])
    args = parser.parse_args()

    root = get_project_root()
    tools_map = resolve_tools(root)
    
    # Paths to config files for tools
    tools_dir = os.path.join(root, "benchmarks", "tools")
    
    # Construct commands
    commands = {
        "startup": [],
        "noop": [],
        "warm_cache": [],
        "cold_build": []
    }

    selected_tools = [t for t in args.tools if t in tools_map and tools_map[t]]

    # Strategy: Change CWD to benchmarks/tools/ so all tools find their configs easily
    os.chdir(tools_dir)
    print(f"Changed CWD to {tools_dir}")
    
    # Prepare commands for cold build (clearing artifacts)
    # Note: 'prepare' runs before EACH run.
    prepare_cmds = {}

    if "ztn" in selected_tools:
        z = tools_map["ztn"]
        commands["startup"].append(f"{z} --version")
        commands["noop"].append(f"{z} run noop")
        commands["warm_cache"].append(f"{z} run build")
        commands["cold_build"].append(f"{z} run build")
        prepare_cmds["ztn"] = "rm -rf .zetten/cache" # Zetten cold build prep

    if "just" in selected_tools:
        j = tools_map["just"]
        commands["startup"].append(f"{j} --version")
        commands["noop"].append(f"{j} noop")
        commands["warm_cache"].append(f"{j} build")
        commands["cold_build"].append(f"{j} build")
        prepare_cmds["just"] = "rm -rf build_artifacts" # Simulated cleanup

        
    if "make" in selected_tools:
        m = tools_map["make"]
        commands["startup"].append(f"{m} --version")
        commands["noop"].append(f"{m} noop")
        commands["warm_cache"].append(f"{m} build")
        commands["cold_build"].append(f"{m} build")
        prepare_cmds["make"] = "rm -rf build_artifacts"

    if "poe" in selected_tools and tools_map["poe"]:
        p = tools_map["poe"]
        commands["startup"].append(f"{p} --version")
        commands["noop"].append(f"{p} noop")
        commands["warm_cache"].append(f"{p} build")
        # Poe doesn't have built-in caching like Zetten for this demo, usually
        commands["cold_build"].append(f"{p} build")

    # Run Scenarios
    scenarios = ["startup", "noop", "warm_cache", "cold_build"] if args.scenario == "all" else [args.scenario]
    
    for sc in scenarios:
        if not commands[sc]:
            continue
        output = os.path.join(root, "benchmarks", f"{sc}.json")
        
        # Determine prepare command if needed
        prep = None
        if sc == "cold_build":
             # Combine cleanups. In a strict comparison we might need per-command prepare,
             # but hyperfine --prepare runs one command. 
             # For simplicity, we'll run a unified clean command or just rely on Zetten's.
             # Actually, hyperfine supports --prepare 'cmd' which runs before EVERY run.
             # If we are comparing ztn vs make, we need to clean BOTH.
             prep = "rm -rf .zetten/cache build_artifacts"

        run_hyperfine(sc, commands[sc], output, prep)
        print_summary(output)

    # Metric: Binary Size
    print("\n--- Binary Size ---")
    for t in selected_tools:
        if t in ["ztn", "just", "make"]: # Compiled/Single binary tools
            path = tools_map[t]
            if os.path.exists(path):
                size_mb = os.path.getsize(path) / (1024 * 1024)
                print(f"{t:<10} | {size_mb:.2f} MB")
        elif t == "poe":
             print(f"{t:<10} | (Python Package - N/A)")

if __name__ == "__main__":
    main()
