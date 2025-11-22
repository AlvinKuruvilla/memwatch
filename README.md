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

### ✔ Per-process peak memory with timestamps

See which worker or process consumed the most memory, and when each process peaked.

### ✔ Timeline-aware sampling

Samples memory at a user-defined interval (default: 500ms).

### ✔ Clean human-readable summary with colors

Compact, table-formatted output with ANSI colors for easy scanning:

```
Job: cargo build --release
Duration: 00:03:21  |  Samples: 402

MEMORY SUMMARY
  Total peak:    6.4 GiB
  Process peak:  912 MiB (pid 8479)

PER-PROCESS PEAKS
    PID      MEMORY     TIME     COMMAND
   8473     534 MiB  @  45.2s   rustc
   8474     612 MiB  @  67.8s   rustc
   8475     703 MiB  @ 102.3s   rustc
   ...

PROCESS GROUPS
  COMMAND    PROCESSES    TOTAL PEAK
  rustc              8      4.2 GiB
  cargo              1      256 MiB
  cc                 2      128 MiB
```

Colors automatically disable when piping to files.

### ✔ Process grouping

Automatically aggregates memory by command name to show which programs consumed the most total memory.

### ✔ Process filtering

Filter display to show only relevant processes while maintaining accurate total memory accounting. Supports regex patterns for flexible include/exclude rules.

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

### Silent mode (suppress command output)

Hide stdout/stderr from the profiled command (useful for noisy commands):

```bash
memwatch run --silent -- mpirun -n 8 ./verbose_app
```

### Process filtering

Filter processes from output while preserving total memory accounting:

```bash
# Exclude processes matching regex
memwatch run --exclude 'cargo|rustc' -- cargo test

# Only include processes matching regex
memwatch run --include 'dis_cq' -- mpirun -n 8 cargo test

# Combine both (include first, then exclude)
memwatch run --include 'test' --exclude 'cargo' -- cargo test
```

**How filtering works:**
- `--exclude <PATTERN>`: Hide processes matching regex pattern from output
- `--include <PATTERN>`: Only show processes matching regex pattern
- Both flags can be combined: include is applied first, then exclude
- **Total RSS always includes all processes** (filtering only affects display)
- Filter metadata shown in output: "2 processes filtered out, 2.1 GiB total"
- Invalid regex patterns produce clear error messages

**Use cases:**
- Hide build overhead: `--exclude 'cargo|rustc|cc|ld'`
- Focus on workers: `--include 'worker|benchmark'`
- Separate infrastructure from computation in MPI/distributed jobs

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
  ],
  "filter": {
    "exclude_pattern": "cargo"
  },
  "filtered_process_count": 8,
  "filtered_total_rss_kib": 400000
}
```

### CSV output

#### Per-process CSV (`--csv`)

Exports peak memory usage for each process:

```csv
# Filter: exclude='cargo' (8 processes filtered out, 400000 KiB total)
pid,ppid,command,max_rss_kib,max_rss_mib,first_seen,last_seen
1234,1233,"rustc",102400,100.00,2025-11-20T18:02:34Z,2025-11-20T18:05:55Z
```

When filters are applied, CSV includes header comments showing which processes were excluded.

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

## 📚 Examples

The `examples/` directory contains realistic demonstration programs showing memwatch's capabilities for different workload types.

### MPI Distributed Computation

**Location**: `examples/mpi_distributed_compute.rs`

A realistic MPI example inspired by distributed zero-knowledge proof systems (like [zeroasset2](https://github.com/privacy-scaling-explorations/zeroasset2)). This demonstrates memwatch's value for tracking memory across MPI process trees - a challenge where traditional profiling tools struggle.

**Quick start:**

```bash
# Run with 4 MPI processes
mpirun -n 4 cargo run --example mpi_distributed_compute

# Profile with memwatch
memwatch run -- mpirun -n 4 cargo run --release --example mpi_distributed_compute
```

**What it demonstrates:**

- **Process tree tracking**: All MPI ranks tracked as a single job
- **Memory scaling**: Per-process memory inversely proportional to process count
- **Short-lived execution**: Captures peak memory in ~5-10 second runs
- **Real workload patterns**: Simulates distributed polynomial computations with realistic memory phases

**Expected output:**

```
Duration: 5.234s
Samples: 11

Max total RSS: 2.25 MiB

Per-process peak RSS:
  PID 12345 (mpirun):                  1.23 MiB
  PID 12346 (mpi_distributed_compute): 384.0 KiB
  PID 12347 (mpi_distributed_compute): 384.0 KiB
  PID 12348 (mpi_distributed_compute): 384.0 KiB
  PID 12349 (mpi_distributed_compute): 384.0 KiB
```

**See** `examples/README.md` for detailed documentation, including:
- Prerequisites (OpenMPI installation)
- Memory scaling patterns
- Comparison with traditional tools
- Real-world applications in HPC and cryptography

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

