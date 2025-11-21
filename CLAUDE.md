# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`memwatch` is a cross-platform job-level memory profiler for macOS and Linux. It tracks total memory, per-process memory, and peak memory for any command and all its child processes. Unlike `top` or `htop`, it monitors entire process trees and provides clean summaries suitable for automation and benchmarking.

## Build and Development Commands

### Building
```bash
cargo build --release
```
Binary location: `target/release/memwatch`

### Running Tests
```bash
cargo test
```

### Development Build
```bash
cargo build
```

### Running the Tool (once implemented)
```bash
# Basic usage
cargo run -- run -- <command> [args...]

# Example with sampling interval
cargo run -- run -i 200 -- cargo test --release

# JSON output
cargo run -- run --json -- <command>
```

## Architecture

### Core Abstraction: ProcessInspector Trait

The project is built around a platform-agnostic abstraction for process inspection:

```rust
trait ProcessInspector {
    fn snapshot_all(&self) -> Result<Vec<ProcessSample>>;
}
```

This trait has OS-specific implementations:
- **Linux**: `/proc`-based implementation (direct file reading, no external commands)
- **macOS**: `ps`-based implementation using `ps -axo pid,ppid,rss,command`

### Module Structure

Expected structure (per SPEC.md):
```
src/
  cli.rs             # CLI argument parsing (clap or similar)
  sampler.rs         # Sampling loop and process tree logic
  inspector/
      mod.rs         # ProcessInspector trait definition
      linux.rs       # Linux /proc implementation
      macos.rs       # macOS ps implementation
  reporter.rs        # Summary formatting and JSON output
  types.rs           # Shared structs (ProcessSample, JobSnapshot, etc.)
```

### Key Data Types

**ProcessSample**: Single snapshot of one process
- `pid: i32`
- `ppid: i32`
- `rss_kib: u64` (always in KiB internally)
- `command: String`

**Job Metrics**:
- `max_total_rss_kib`: Peak sum of RSS across all job processes
- Per-PID peak RSS tracking
- Duration and sample count

### Process Tree Detection

Both platforms use the same algorithm:
1. Build a PID → (PPID, RSS, command) map
2. A process belongs to the job if:
   - `pid == root_pid`, OR
   - Following PPID chain reaches `root_pid`

### Platform-Specific Implementation Notes

#### Linux Backend
- Use `/proc/<pid>/status` for RSS (`VmRSS`) or `/proc/<pid>/statm`
- Parse `/proc/<pid>/stat` for PPID
- Read `/proc/<pid>/cmdline` for command
- No external commands required

#### macOS Backend
- Execute `ps -axo pid,ppid,rss,command` once per interval
- RSS from macOS `ps` is already in KiB
- Parse output into ProcessSample records
- v2 may use `libproc` APIs for better performance

### Cross-Platform Requirements

**CRITICAL**: CLI, output format, and semantics must be identical between Linux and macOS. Only internals for process discovery differ.

All memory units are stored as **KiB internally**. Human-readable conversion (MiB, GiB) happens only at presentation layer.

## Output Formats

### Human-Readable (default)
- Duration, sample count
- Max total RSS (formatted as KiB/MiB/GiB)
- Max per-process RSS
- Per-process peak table with PIDs and commands

### JSON (--json flag)
Structured output with:
- Command array
- Start/end timestamps (ISO 8601)
- Duration in seconds
- `max_total_rss_kib`
- `processes` array with per-PID metrics

## CLI Structure

```
memwatch run [OPTIONS] -- <command> [args...]

Options:
  -i, --interval <ms>    Sampling interval in milliseconds (default: 500)
      --json             Output JSON instead of human-readable text
      --quiet            Suppress output (useful with --json)
```

## Key Implementation Considerations

### Sampling Loop
- Configurable interval (default 500ms)
- Handle process churn (processes appearing/disappearing between samples)
- Update running maxima, don't store all samples in memory
- Continue while at least one job process is alive

### Error Handling
- Command fails to start: clear error message, non-zero exit
- Quick-exit commands: handle gracefully with minimal samples
- SIGINT forwarding: forward to root PID and children, then cleanup
- Permission errors: skip with warning

### Testing Strategy
- Unit tests for process-tree detection with mocked snapshots
- Unit tests for unit conversion (KiB → MiB/GiB)
- Integration tests with simple commands (e.g., `sleep`, small memory scripts)

## Platform Support

**Supported in v1**: macOS, Linux
**Not supported**: Windows (abstraction designed for future addition)

If built on Windows, should fail compilation with clear `cfg` error or exit immediately with unsupported message.

## Implementation Phases

1. **CLI parsing + subprocess spawning**: Parse args, spawn command, track timing
2. **Linux sampling**: Implement `/proc`-based inspector
3. **macOS sampling**: Implement `ps`-based inspector
4. **Output formatting**: Human-readable summary + JSON
5. **Polish**: Error handling, unit tests, integration tests

## Memory Units

Always use **KiB** as the base unit internally:
- 1 KiB = 1024 bytes
- 1 MiB = 1024 KiB
- 1 GiB = 1024 MiB

Convert to human-readable IEC units only for display.
