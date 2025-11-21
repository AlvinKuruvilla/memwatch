# memwatch

### **Cross-platform job-level memory profiler (macOS + Linux)**

Track **total memory**, **per-process memory**, and **peak memory** for any command and all its child processes.

---

## 🚀 What is memwatch?

`memwatch` is a **job-level memory profiler** designed for developers and researchers running large or parallel workloads. Unlike `top`, `htop`, or `/usr/bin/time`, `memwatch` tracks **all descendant processes**, computes **peak total RAM**, and outputs a clean **summary** or **JSON** record suitable for automation and benchmarking.

This tool is ideal for:

* Rust builds (`cargo test`, `cargo bench`, `cargo run`)
* Proving systems and HPC jobs (`mpirun`, `prterun`, rayon-heavy workloads)
* Pipelines that spawn subprocesses
* Automated CI memory regression detection
* Capacity planning (e.g., “Can my machine handle N workers?”)

Windows support is planned, but **v1 focuses on macOS + Linux**.

---

## ✨ Features

### ✔ Track entire process trees

Monitors the root command **plus all children and grandchildren**.

### ✔ Peak total memory

Captures the real high-water RAM usage across the entire job.

### ✔ Per-process peak memory

See which worker or process consumed the most memory.

### ✔ Timeline-aware sampling

Samples memory at a user-defined interval (default: 500ms).

### ✔ Clean human-readable summary

Example:

```
Job: cargo build --release
Duration:        00:03:21
Samples:         402

Max total RSS:   6.4 GiB
Max per process: 912 MiB (pid 8479)

Per-process peak RSS:
  pid 8473  534 MiB  rustc
  pid 8474  612 MiB  rustc
  pid 8475  703 MiB  rustc
  ...

Process Groups (by command):
  rustc                ( 8 processes)  - Total peak: 4.2 GiB
  cargo                ( 1 process)   - Total peak: 256 MiB
  cc                   ( 2 processes)  - Total peak: 128 MiB
```

### ✔ Process grouping

Automatically aggregates memory by command name to show which programs consumed the most total memory.

### ✔ JSON export

Perfect for dashboards, plotting, or regression tracking.

### ✔ Exit code forwarding

Transparently passes through the child process's exit code for seamless CI/CD integration.

### ✔ macOS + Linux parity

Behavior is identical across both platforms.

---

## 🔧 Installation

Build locally:

```bash
git clone https://github.com/AlvinKuruvilla/memwatch
cd memwatch
cargo build --release
```

Binary will be in:

```
target/release/memwatch
```

### Man Page

A man page is automatically generated during build:

```bash
# View the man page
man target/debug/man/memwatch.1

# Or after release build
man target/release/man/memwatch.1
```

---

## 🕹 Usage

### Basic usage

```bash
memwatch run -- cargo test --release
```

### With a specific sampling interval

```bash
memwatch run -i 200 -- ./program --arg1 foo
```

Interval is in milliseconds.

### JSON output

```bash
memwatch run --json -- cargo test > mem.json
```

### Quiet mode (good for scripts)

```bash
memwatch run --json --quiet -- my_command
```

### CSV export

Export per-process peak memory to CSV:

```bash
memwatch run --csv processes.csv -- cargo build
```

### Timeline export

Export time-series memory data for plotting:

```bash
memwatch run --timeline timeline.csv -- ./benchmark
```

### Combined exports

```bash
memwatch run --csv procs.csv --timeline time.csv -- my_command
```

---

## 📦 Output Formats

### Human-readable summary (default)

Printed when job completes:

* Duration
* Peak total RSS
* Peak RSS per process
* Process list
* Sample count

### JSON output

Structured and stable:

```json
{
  "command": ["cargo", "test"],
  "start_time": "2025-11-20T18:02:34Z",
  "end_time": "2025-11-20T18:05:55Z",
  "duration_seconds": 201.4,
  "interval_ms": 500,
  "max_total_rss_kib": 6624768,
  "processes": [
    {
      "pid": 8473,
      "ppid": 8472,
      "command": "rustc",
      "max_rss_kib": 54656,
      "first_seen": "2025-11-20T18:02:40Z",
      "last_seen": "2025-11-20T18:05:10Z"
    }
  ]
}
```

### CSV output

#### Per-process CSV (`--csv`)

Exports peak memory usage for each process:

```csv
pid,ppid,command,max_rss_kib,max_rss_mib,first_seen,last_seen
1234,1233,"cargo test",102400,100.00,2025-11-20T18:02:34Z,2025-11-20T18:05:55Z
```

#### Timeline CSV (`--timeline`)

Exports memory usage over time for plotting:

```csv
timestamp,elapsed_seconds,total_rss_kib,total_rss_mib,process_count
2025-11-20T18:02:34Z,0.000,51200,50.00,4
2025-11-20T18:02:35Z,0.500,102400,100.00,8
```

Perfect for creating graphs in Python, R, Excel, or Grafana.

---

## 🧩 Architecture

```
src/
  cli.rs             # argument parsing
  sampler.rs         # sampling loop, process tree logic
  inspector/
      mod.rs         # ProcessInspector trait
      linux.rs       # /proc implementation
      macos.rs       # ps-based implementation
  reporter.rs        # summary + JSON output
  types.rs           # structs shared across modules
```

### ProcessInspector (core abstraction)

```rust
trait ProcessInspector {
    fn snapshot_all(&self) -> Result<Vec<ProcessSample>>;
}
```

Every OS implements it differently:

* **Linux** → `/proc`
* **macOS** → `ps -axo pid,ppid,rss,command`

This ensures:

* identical behavior across OSes
* simple future extension to Windows

---

## 🏗 How It Works

1. `memwatch` starts your command as a subprocess.
2. It records the root PID.
3. Every interval:

   * It inspects *all* system processes.
   * Builds PID→PPID mapping.
   * Determines which processes belong to the job.
   * Sums RSS across all job processes.
4. While sampling:

   * Tracks peak total RSS.
   * Tracks per-PID max RSS.
5. After job exit:

   * Computes summary.
   * Prints human-readable or JSON output.

Low overhead (<5%), no slowdown for real Rust or HPC workloads.

---

## 🧪 Example Workflows

### Benchmarking Rust compilations

```bash
memwatch run -- cargo build --release
```

### Parallel proving system memory scaling

```bash
memwatch run -- prterun -n 16 ./prove_large_circuit
```

### CI regression detection

```bash
memwatch run --json -- my_benchmark > results.json
```

Consume in Python or Grafana:

```python
import json
result = json.load(open("results.json"))
print(result["max_total_rss_kib"])
```

---

## ⚠️ Platform Support

### macOS — Fully supported

Uses `ps` backend (stable and good enough for v1).

### Linux — Fully supported

Uses `/proc` backend for efficient memory sampling.

### Windows — Not supported yet

Will require:

* Windows API process enumeration
* Memory counters via `PROCESS_MEMORY_COUNTERS_EX`

Planned as a separate backend later.

---

## 🔢 Exit Codes

`memwatch` uses specific exit codes to communicate results, particularly useful for CI/CD integration:

| Exit Code | Meaning |
|-----------|---------|
| `0` | Success - job completed, no limits exceeded |
| `1` | General error (command failed, invalid arguments, file I/O error) |
| `10` | Total RSS limit exceeded (future: `--max-total-rss`) |
| `11` | Per-process RSS limit exceeded (future: `--max-per-proc-rss`) |

Additional exit codes may be added for future features like leak detection thresholds.

---

## 🗺 Roadmap

### v1.0 ✅ Complete

- [x] macOS + Linux support
- [x] JSON output
- [x] Process tree memory tracking
- [x] Aggregated peak memory
- [x] Clean human-readable summary
- [x] Configurable sampling interval
- [x] Per-process peak RSS tracking

### v1.1 ✅ Complete

- [x] CSV export
- [x] Time-series export (timeline.csv)
- [x] Edge case handling (defunct processes, quick-exit commands)
- [x] Helpful error messages and suggestions
- [x] Man page documentation
- [x] Exit code forwarding (transparent pass-through for CI/CD)
- [x] Enhanced version info (build date and target platform)
- [x] Basic process grouping (automatic aggregation by command name)

### v1.2 (Packaging)

- [ ] Package for crates.io
- [ ] Homebrew formula

### v2.0 (Core Profiling Enhancements)

- [ ] **Threshold alerts** - Memory guardrails for CI/CD
  - `--max-total-rss <bytes>` and `--max-per-proc-rss <bytes>` flags
  - Enforce limits and exit with specific codes (10/11) when exceeded
  - Support human units (e.g., `512M`, `8G`)
- [ ] **Run comparison** - Regression detection between runs
  - `memwatch compare base.json candidate.json` command
  - Detect memory increases with configurable thresholds
  - Perfect for CI pipelines to catch regressions
- [ ] **Advanced process grouping** - Configurable grouping strategies
  - `--group-by command/exe/ppid` flag for custom grouping
  - Note: Basic process grouping (by command name) is already implemented in v1.1
  - Advanced features: custom grouping keys, filtering, hierarchical views
- [ ] **Improved short-lived process detection**
  - Better capture of rapid fork/exit patterns
  - Optional startup burst sampling mode
- [ ] libproc backend for faster macOS sampling
- [ ] Windows backend

### v3.0 (Advanced Features)

- [ ] **Leak detection heuristics** - Detect potential memory leaks
  - Analyze RSS growth trends over job lifetime
  - Configurable thresholds (`--leak-min-growth`, `--leak-min-duration`)
  - Conservative defaults to avoid false positives (50% growth, 60s minimum)
- [ ] **`memwatch view`** - Interactive HTML visualizer
  - Beautiful interactive HTML reports for completed runs
  - `memwatch view results.json --html report.html`
  - Grouped timeline view showing per-process memory over time (like Chrome DevTools)
  - Zoom, pan, hover for detailed metrics
  - Multiple chart types: timeline swimlanes, stacked area charts, summary tables
  - Single-file HTML output for easy sharing
- [ ] **Live progress mode** - Real-time job monitoring
  - `--live` flag for simple single-line progress updates
  - Shows current RSS, peak RSS, and top process during execution
  - Non-intrusive, works over SSH and in CI logs
- [ ] **CI integration helpers** - GitHub Actions and automation
  - Example workflow files for catching regressions
  - Shell/Python wrappers for common CI patterns
  - Optional `memwatch ci-check` convenience command
- [ ] **Plugin/hook API** - Extensibility for custom workflows
  - `--post-run-cmd` flag to execute custom commands after profiling
  - Receive JSON output path as argument
  - Integrate with Prometheus, Grafana, custom dashboards, etc.

### 🔬 Future Research

These features represent potential scope expansions beyond job-level memory profiling. They're listed here for visibility but are **not committed to any version** as they may be better served by separate tools or integrations.

- **CPU/I/O metrics** - Expand beyond memory profiling
  - `--with-cpu` and `--with-io` flags for additional performance data
  - Platform-dependent, requires different OS APIs
  - **Rationale for deferral**: Significant complexity; users needing this likely already use `perf`, `htop`, etc. Focus remains on being the best memory profiler.

- **GPU memory tracking** - ML/HPC workload support
  - `--with-gpu` flag to track VRAM usage (NVIDIA via nvidia-smi/NVML)
  - Useful for machine learning and HPC workloads
  - **Rationale for deferral**: Platform-specific (Linux-only initially), GPUs are shared resources (doesn't fit job-level model cleanly). May be better as a wrapper script or separate tool.

**Philosophy**: memwatch aims to be the **definitive job-level memory profiler** - simple, reliable, and cross-platform. Features that significantly expand scope or complexity are carefully evaluated to maintain this focus.

---

## 🤝 Contributing

Pull requests, issues, and benchmarks are welcome.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

