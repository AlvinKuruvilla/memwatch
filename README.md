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
Job: prterun -n 8 cargo test --release -p dis_cq bench_fold_prove_scalability
Duration:        00:03:21
Samples:         402

Max total RSS:   6.4 GiB
Max per process: 912 MiB (pid 8479)

Per-process peak RSS:
  pid 8473  534 MiB  rustc
  pid 8474  612 MiB  rustc
  pid 8475  703 MiB  rustc
  ...
```

### ✔ JSON export

Perfect for dashboards, plotting, or regression tracking.

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

## 🗺 Roadmap

### v1.0 ✅ Complete

- [x] macOS + Linux support
- [x] JSON output
- [x] Process tree memory tracking
- [x] Aggregated peak memory
- [x] Clean human-readable summary
- [x] Configurable sampling interval
- [x] Per-process peak RSS tracking

### v1.1 (In Progress)

- [x] CSV export
- [x] Time-series export (timeline.csv)
- [x] Edge case handling (defunct processes, quick-exit commands)
- [x] Helpful error messages and suggestions
- [ ] Optional RAM usage graph via CLI flag
- [ ] Package for crates.io
- [ ] Homebrew formula
- [ ] Man page documentation

### v2.0 (Future)

- [ ] libproc backend for faster macOS sampling
- [ ] Windows backend
- [ ] Improved detection of short-lived worker processes
- [ ] Memory usage flamegraph visualization

### v3.0 (Future)

- [ ] `compare` command for regression analysis
- [ ] CI assistant scripts and GitHub Actions
- [ ] Interactive TUI mode
- [ ] Memory leak detection

---

## 🤝 Contributing

Pull requests, issues, and benchmarks are welcome.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

