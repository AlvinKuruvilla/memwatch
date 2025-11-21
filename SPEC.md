Good, this is the right way to use me: make me do the thinking, then throw the build at Claude.

Here’s a spec you can hand off directly.

---

## 1. Project Overview

**Name (working):** `memwatch`
**Purpose:** Run an arbitrary command, track **memory usage over time for that command and all its child processes**, and print a **concise summary of peak memory usage** + optional detailed metrics.

This is *not* a `top` clone. It’s a **job-level memory profiler** for things like:

* `cargo test …`
* `mpirun`, `prterun`, `pytest -n 8`
* Custom benchmarks or long-running pipelines

---

## 2. Core Goals

1. **Track a command and all its descendants** (process tree rooted at the spawned command).
2. Periodically **sample memory usage** (RSS) per process.
3. Compute and print:

   * Peak **total RSS** across all job processes
   * Peak **RSS per process**
   * Duration of the job
4. Provide a **human-readable summary** and optionally **machine-readable output** (JSON/CSV).
5. Work on **macOS** and **Linux** (best-effort parity).

---

## 3. Non-Goals (v1)

* No interactive TUI like `htop`.
* No CPU profiling, I/O stats, or perf counters (optional later).
* No kernel module or privileged monitoring.
* No remote monitoring; only local machine.

---

## 4. Primary Use Cases

1. **Capacity planning**

   > “If I run `prterun -n 8 cargo test …`, how much RAM does the whole job actually use at peak?”

2. **Scaling experiments**

   > “How does peak RAM change as I vary `-n` workers?”

3. **Regression detection**

   > “Did this branch increase peak memory by more than 20% vs main?”

---

## 5. High-Level Behavior

### 5.1 CLI shape

Base invocation:

```bash
memwatch run -- <command> [args...]
```

Examples:

```bash
memwatch run -- cargo test --release -p dis_cq bench_fold_prove_scalability
memwatch run -- prterun -n 8 ./target/release/my_binary
```

Behavior:

1. `memwatch` spawns `<command> [args...]` as a child.
2. It **tracks that process and all descendants** until they all exit.
3. While the job is alive, it periodically samples:

   * per-PID RSS
   * per-PID command name / short command line
4. After completion, it computes aggregate stats and prints a summary.

---

## 6. Detailed Requirements

### 6.1 Process Tracking

* Track a **root PID** (the command spawned by `memwatch`).

* Repeatedly query **current process tree**:

  * Linux:

    * Prefer `/proc/<pid>/stat` or `/proc/<pid>/status` for RSS.
    * Use `/proc/<pid>/task` or parse `/proc` tree to find descendants.
  * macOS:

    * Use `ps` initially (simpler v1).

      * Example: `ps -o pid,ppid,rss,command -ax`
    * Filter processes where:

      * The PID is the root, or
      * The process is a descendant (walk parent chain via PPID until root or no parent).
    * (Optional v2: use `libproc` APIs for better performance.)

* Sampling should handle **process churn**:

  * New processes appearing between samples.
  * Exited processes disappearing between samples.

### 6.2 Sampling Model

Configurable sampling interval:

* Default: `--interval 200ms` (or 500ms; pick a sane default).
* CLI flag: `--interval <ms>`

Pseudo-behavior:

* While at least one process in the job is alive:

  * Sleep `interval_ms`.
  * Collect snapshot:

    * Set of PIDs in the job.
    * For each PID:

      * RSS in KiB
      * Simple descriptor (e.g., command name)
  * Update metrics (see below).

### 6.3 Metrics & Aggregations

For the **whole job**:

* `start_time` (wall-clock)
* `end_time`
* `duration` (seconds)
* `max_total_rss_kib` – max over time of `sum(rss_kib for all PIDs at that sample)`
* `samples` – total number of polling samples

For **each PID** (process-level stats):

* `pid`
* `ppid`
* `command` (best-effort trimmed command line)
* `max_rss_kib`
* `first_seen` timestamp
* `last_seen` timestamp
* Optional: number of samples seen

No need to store all samples in memory for v1; only **update running maxima** and maybe keep a small window of recent data if needed.

### 6.4 Output Formats

#### 6.4.1 Human-readable (default)

After the job exits, print:

Example:

```text
Job: cargo test --release -p dis_cq bench_fold_prove_scalability
Duration:        00:03:21
Samples:         402

Max total RSS:   6.4 GiB (across 8 processes)
Max per process: 912 MiB (pid 8479)

Per-process peak RSS:
  pid 8473  534 MiB  /Users/alvinkuruvilla/.rustup/.../rustc
  pid 8474  612 MiB  /Users/alvinkurvilla/.rustup/.../rustc
  pid 8475  703 MiB  /Users/alvinkurvilla/.rustup/.../rustc
  pid 8476  546 MiB  /Users/alvinkurvilla/.rustup/.../rustc
  ...
```

Formatting rules:

* Convert KiB → human-readable IEC units (KiB, MiB, GiB).
* Align columns nicely.

#### 6.4.2 JSON output

CLI flag:

```bash
memwatch run --json -- cargo test ...
```

JSON structure (example):

```json
{
  "command": ["cargo", "test", "--release", "-p", "dis_cq", "bench_fold_prove_scalability"],
  "start_time": "2025-11-20T18:02:34Z",
  "end_time": "2025-11-20T18:05:55Z",
  "duration_seconds": 201.4,
  "interval_ms": 200,
  "max_total_rss_kib": 6624768,
  "max_total_rss_sample_index": 320,
  "processes": [
    {
      "pid": 8473,
      "ppid": 8472,
      "command": "/Users/.../rustc",
      "max_rss_kib": 54656,
      "first_seen": "2025-11-20T18:02:40Z",
      "last_seen": "2025-11-20T18:05:10Z"
    }
  ]
}
```

User can redirect to a file:

```bash
memwatch run --json -- cargo test ... > mem_profile.json
```

#### 6.4.3 CSV output (optional v1, okay to add v1.1)

Flag:

```bash
--csv <path>
```

Two possible tables:

1. **process_peaks.csv** – one row per PID with peak RSS.
2. (Optional later) **timeline.csv** – one row per sample with total RSS.

---

## 7. CLI Specification

Basic structure:

```bash
memwatch <subcommand> [flags] -- <command> [args...]
```

### 7.1 Subcommands

* `run` – main and only required subcommand in v1.

Example full spec:

```text
memwatch run [OPTIONS] -- <command> [args...]

Options:
  -i, --interval <ms>       Sampling interval in milliseconds (default: 500)
      --json                Output JSON instead of human-readable text
      --quiet               Suppress human-readable output (when using --json)
      --version             Show version
      --help                Show help
```

Later extensions (not required now):

* `memwatch compare <file1.json> <file2.json>` – compare two runs (v2 idea).
* `memwatch diff` / `report`, etc.

---

## 8. OS-Specific Implementation Notes

### 8.1 Linux

* Use `/proc` directly:

  * RSS: `/proc/<pid>/statm` or `/proc/<pid>/status` (`VmRSS`).
  * PPID & command:

    * `/proc/<pid>/stat`
    * `/proc/<pid>/cmdline`
* Detect children by:

  * Scanning `/proc`, reading `PPID` for each process, and building a PID→PPID map.
  * Determine “in job” if you can reach the root PID walking parent links.

### 8.2 macOS

**v1 (simpler, slower but acceptable):**

* Shell out to `ps` with a fixed format:

  ```bash
  ps -axo pid,ppid,rss,command
  ```

* Parse the output:

  * RSS is already in KiB.
  * Build PID→(PPID, RSS, command) map.
  * Same parent-chain logic to determine descendants.

This is not blazing fast, but with intervals like 500ms or 1s it’s fine for v1 and avoids fighting C APIs. You can refactor later.

---

## 9. Error Handling & Edge Cases

1. **Command fails to start** (e.g., not found):

   * Print clear error:

     ```text
     Failed to start command: No such file or directory
     ```
   * Exit non-zero.

2. **Command exits very quickly (< 1 sample interval)**:

   * Still record at least:

     * start_time
     * end_time
     * duration
   * A single “best effort” sample right after process exit is okay.

3. **No processes discovered for root PID**:

   * Likely a race / quick exit.
   * Handle gracefully (empty process list but still print duration, etc.).

4. **Long-running process cancellation**:

   * If user sends SIGINT (Ctrl+C) to `memwatch`, forward the signal to the root PID and children, then exit after cleanup.

5. **Permissions**:

   * If some processes can’t be inspected (rare for your use), skip them with a warning.

---

## 10. Implementation Phases (for Claude Code)

**Phase 1 – Skeleton + subprocess**

* Implement CLI parsing.
* Implement spawning `<command> [args...]` and waiting for exit.
* Add minimal timing (start/end/duration).

**Phase 2 – Linux-only sampling (if running on Linux)**

* Implement `/proc`-based sampler.
* Track root + children; log max total RSS + per-PID peaks.

**Phase 3 – macOS `ps` sampler**

* Implement `ps -axo pid,ppid,rss,command` polling.
* Integrate same logic.

**Phase 4 – Output formats**

* Human-readable summary.
* JSON output (`--json` flag).

**Phase 5 – Polish**

* Good error messages.
* Unit tests for:

  * Process-tree detection given a mocked `ps`/`proc` snapshot.
  * Unit conversion (KiB → MiB/GiB).
* A couple of integration tests using a trivial command like `sleep 1` and a small memory hog script.
Good catch to say that explicitly. Let’s bolt that into the spec so Claude doesn’t quietly go “Linux only”.

Here’s an **add-on section** you can append to the spec you already have and hand to Claude:

---

## 11. Cross-Platform Requirements (macOS + Linux)

### 11.1 Supported OSes in v1

* **Must support:**

  * `target_os = "linux"`
  * `target_os = "macos"`
* **Out of scope for v1:** Windows (may be added later as a separate backend).

The **CLI, output format, and semantics must be identical** between Linux and macOS. Only the *internals* for process discovery and RSS measurement should differ.

---

### 11.2 Process Info Abstraction

Define a small abstraction so OS-specific code is isolated:

```rust
/// One snapshot of a single process at a point in time.
struct ProcessSample {
    pid: i32,
    ppid: i32,
    rss_kib: u64,
    command: String,  // short command line
}

trait ProcessInspector {
    /// Return a map of pid -> ProcessSample for *all* processes on the system.
    fn snapshot_all(&self) -> Result<Vec<ProcessSample>, InspectorError>;
}
```

Then the sampling loop can be **OS-agnostic**:

```rust
fn sample_job_tree(
    inspector: &impl ProcessInspector,
    root_pid: i32,
) -> JobSnapshot { /* same on both OSes */ }
```

Platform-specific implementations:

* `LinuxProcessInspector`
* `MacProcessInspector`

Wired with `cfg`:

```rust
#[cfg(target_os = "linux")]
fn make_inspector() -> LinuxProcessInspector { ... }

#[cfg(target_os = "macos")]
fn make_inspector() -> MacProcessInspector { ... }
```

The rest of the code (CLI, aggregation, JSON/summary output) must **not** depend on the OS.

---

### 11.3 Linux Backend Requirements

Implementation freedom: `/proc` is preferred, no external commands.

**Mandatory behavior:**

* Use `/proc` to obtain:

  * `pid`, `ppid`
  * RSS in KiB
  * command / cmdline (trimmed to reasonable length)
* RSS sources:

  * Either `/proc/<pid>/status` (`VmRSS`) or `/proc/<pid>/statm` + page size.
* Process tree logic:

  * Build a PID → (PPID, RSS, command) map from `/proc`.
  * A process belongs to the job if:

    * `pid == root_pid`, or
    * following the PPID chain eventually reaches `root_pid`.

No shelling out on Linux unless absolutely necessary.

---

### 11.4 macOS Backend Requirements

macOS doesn’t have `/proc`, so v1 can use `ps` to keep development reasonable.

**v1 macOS approach (acceptable and required):**

* Call `ps` with a fixed format:

  ```bash
  ps -axo pid,ppid,rss,command
  ```

* Parse the output into `ProcessSample` records:

  * `pid` → PID
  * `ppid` → parent PID
  * `rss` → resident set size in **KiB** (macOS ps already reports in KiB)
  * `command` → full command line, trimmed

* Same process-tree logic as Linux:

  * Build PID map, walk PPID chain back to root.

**Implementation notes:**

* The `ps` invocation should be:

  * Hard-coded format (no locale-dependent output).
  * Executed once per sampling interval.
* Errors from `ps`:

  * If `ps` fails once, log a warning and skip that sample.
  * If `ps` fails repeatedly, exit with a clear error.

**v2 (future) macOS note (optional for now):**

* Backend can later be swapped to `libproc` or `sysctl` for speed, but the **trait and public behavior must not change**.

---

### 11.5 Behavior Parity Expectations

For the *same workload* and similar sampling interval:

* Both Linux and macOS versions must:

  * Track the same root PID semantics.
  * Include all descendants visible during sampling.
  * Produce structurally identical JSON.
* Numeric differences are okay (different kernel accounting), but:

  * RSS should be within reasonable ballpark (no obvious unit errors).
  * Units must always be KiB internally, with human-readable conversion only at the presentation layer.

---

### 11.6 Windows (Explicitly Deferred)

* Clearly mark in the README and code:

  * Windows builds are **not supported in v1**.
  * If someone attempts to build on Windows, the binary should:

    * Either fail compilation with a clear `cfg` error, **or**
    * Start and immediately print:
      `"memwatch: Windows support is not implemented yet"` and exit with non-zero.

* The abstraction via `ProcessInspector` is intentionally chosen so that a future `WindowsProcessInspector` can be added without touching the rest of the code.