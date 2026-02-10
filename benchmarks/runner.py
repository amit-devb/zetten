
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

def run_hyperfine(name, commands, output_file):
    print(f"--- Benchmarking: {name} ---")
    cmd = [
        "hyperfine",
        "--warmup", str(WARMUP_RUNS),
        "--runs", str(BENCHMARK_RUNS),
        "--export-json", output_file,
    ] + commands
    
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
    
    for res in results:
        cmd_name = res["command"].split()[0].split("/")[-1] # Try to get short name
        if "python" in cmd_name: cmd_name = "poe" 
        
        mean = res["mean"] * 1000 # ms
        stddev = res["stddev"] * 1000 # ms
        ratio = res["mean"] / baseline
        print(f"{cmd_name:<10} | {mean:.2f} ms ± {stddev:.2f} | {ratio:.2f}x")
    print("-" * 50)

def main():
    parser = argparse.ArgumentParser(description="Zetten Benchmark Runner")
    parser.add_argument("--scenario", choices=["startup", "noop", "warm_cache", "all"], default="all")
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
        "warm_cache": []
    }

    selected_tools = [t for t in args.tools if t in tools_map and tools_map[t]]

    for t in selected_tools:
        bin_path = tools_map[t]
        
        if t == "ztn":
            # Point to zetten.toml in tools dir
            base_cmd = f"{bin_path}" # usage: ztn -C benchmarks/tools ... (if supported) or just cd
            # Zetten currently needs to be run FROM the dir or using set_current_dir logic?
            # Actually Main.rs:131 finds project root. 
            # Benchmarks rely on CWD being benchmarks/tools for config resolution usually.
            pass
            
        if t == "make":
             # make -f benchmarks/tools/Makefile
             pass

    # Strategy: Change CWD to benchmarks/tools/ so all tools find their configs easily
    # But pass absolute paths to binaries
    os.chdir(tools_dir)
    print(f"Changed CWD to {tools_dir}")

    # Re-map commands relative to new CWD
    
    cmds_list = []
    
    if "ztn" in selected_tools:
        z = tools_map["ztn"]
        commands["startup"].append(f"{z} --version")
        commands["noop"].append(f"{z} run noop")
        commands["warm_cache"].append(f"{z} run build")

    if "just" in selected_tools:
        j = tools_map["just"]
        commands["startup"].append(f"{j} --version")
        commands["noop"].append(f"{j} noop")
        commands["warm_cache"].append(f"{j} build")
        
    if "make" in selected_tools:
        m = tools_map["make"]
        commands["startup"].append(f"{m} --version")
        commands["noop"].append(f"{m} noop")
        commands["warm_cache"].append(f"{m} build")

    if "poe" in selected_tools and tools_map["poe"]:
        p = tools_map["poe"]
        commands["startup"].append(f"{p} --version")
        commands["noop"].append(f"{p} noop")
        commands["warm_cache"].append(f"{p} build")

    # Run Scenarios
    scenarios = ["startup", "noop", "warm_cache"] if args.scenario == "all" else [args.scenario]
    
    for sc in scenarios:
        if not commands[sc]:
            continue
        output = os.path.join(root, "benchmarks", f"{sc}.json")
        run_hyperfine(sc, commands[sc], output)
        print_summary(output)

if __name__ == "__main__":
    main()
