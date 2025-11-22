# Build Pipeline Example

A comprehensive multi-program example that demonstrates `memwatch`'s ability to track complex process trees spanning multiple programming languages and tools.

## Overview

This example simulates a realistic CI/CD build pipeline with six distinct phases:

1. **Code Generation** (Python) - Generates Rust source files
2. **C Library Compilation** (C/cc) - Compiles C helper library
3. **Rust Compilation** (Rust/cargo) - Compiles code, spawning rustc → cc → ld
4. **Testing** (Node.js) - Runs tests with memory-intensive operations
5. **Data Validation** (Perl) - Validates data integrity and generates reports
6. **Packaging** (Python) - Creates distribution packages

Each phase spawns different programs, creating a complex process tree that showcases memwatch's cross-language tracking capabilities spanning **five programming languages** (Python, C, Rust, JavaScript/Node.js, and Perl).

## Why This Example?

This example demonstrates several key memwatch features:

- **Multi-language tracking**: Python, C, Rust compiler, Node.js, Perl all in one job
- **Complex process trees**: Real compilation spawns cargo → rustc → cc → ld (4 levels deep)
- **Realistic workload**: Actual compilation and real tool invocations
- **Memory phases**: Six distinct phases with different memory characteristics
- **Process filtering**: Many helper processes that can be filtered
- **Timeline visualization**: Clear phases that show up in timeline exports
- **Configurable size**: Small/medium/large problem sizes
- **Parallel vs sequential**: Compare concurrent vs sequential memory usage

## Prerequisites

Required tools (all standard on macOS/Linux):
- `python3` - For code generation and packaging
- `cc` - C compiler (usually pre-installed on macOS/Linux)
- `cargo` / `rustc` - Already present (this is a Rust project)
- `node` - For test runner (install via: `brew install node` or `apt install nodejs`)
- `perl` - For data validation (pre-installed on macOS/Linux)
- `make` - Standard build tool

Check if required tools are installed:
```bash
python3 --version  # Should show Python 3.7+
cc --version       # Should show clang or gcc
node --version     # Should show v14+ or later
perl --version     # Should show Perl 5.x
```

## Quick Start

### Basic Usage

```bash
# Navigate to the example directory
cd examples/build_pipeline

# Run the full pipeline (sequential, small size)
memwatch run -- make

# Clean up generated files
make clean
```

### Different Problem Sizes

```bash
# Small: Quick demo (~10 MB source, ~100 MB peak memory, ~10 seconds)
memwatch run -- make SIZE=small

# Medium: Moderate workload (~50 MB source, ~500 MB peak, ~30 seconds)
memwatch run -- make SIZE=medium

# Large: Stress test (~200 MB source, ~2 GB peak, ~60+ seconds)
memwatch run -- make SIZE=large
```

### Parallel vs Sequential

```bash
# Sequential: Easier to see distinct phases
memwatch run -- make

# Parallel: Shows concurrent memory usage
memwatch run -- make all-parallel JOBS=4
```

## Understanding the Pipeline

The pipeline now includes 6 phases spanning 5 programming languages, demonstrating memwatch's comprehensive multi-language tracking capabilities.

### Phase 1: Code Generation (Python)

**What happens:**
- Python script `codegen.py` generates Rust source files
- Creates multiple modules with large const arrays
- Configurable based on SIZE parameter

**Memory characteristics:**
- Small: ~50 MB
- Medium: ~100 MB
- Large: ~200 MB

**Process tree:**
```
make
└── python3 scripts/codegen.py
```

### Phase 2: C Library Compilation (C)

**What happens:**
- Compiles C helper library using `cc` (clang or gcc)
- Creates static library with `ar`
- Demonstrates C compilation in the build chain

**Memory characteristics:**
- Small: ~10-20 MB
- Medium: ~10-20 MB
- Large: ~10-20 MB (C compilation is lightweight)

**Process tree:**
```
make
├── cc src/lib_helper.c
└── ar rcs libhelper.a
```

### Phase 3: Rust Compilation (Rust)

**What happens:**
- Cargo compiles the generated Rust code
- rustc spawns multiple processes for parallel compilation
- C linker (cc) and system linker (ld) are invoked

**Memory characteristics:**
- Small: ~200-500 MB
- Medium: ~500 MB - 1 GB
- Large: ~1-2 GB

**Process tree:**
```
make
└── cargo build
    ├── rustc (main compilation)
    │   ├── rustc (codegen unit 1)
    │   ├── rustc (codegen unit 2)
    │   └── ... (more parallel codegen units)
    ├── cc (C linker wrapper)
    └── ld (system linker)
```

This is the most memory-intensive phase and demonstrates complex multi-level process trees.

### Phase 4: Testing (Node.js)

**What happens:**
- Node.js script `run_tests.js` runs test suite
- Validates the compiled binary
- Performs memory-intensive operations (data processing, string ops, arrays)

**Memory characteristics:**
- Small: ~50-100 MB
- Medium: ~200-300 MB
- Large: ~500 MB

**Process tree:**
```
make
└── node scripts/run_tests.js
    └── build_example (spawned to test binary)
```

### Phase 5: Data Validation (Perl)

**What happens:**
- Perl script `validate_data.pl` validates data integrity
- Generates test data and computes checksums
- Creates validation reports

**Memory characteristics:**
- Small: ~20-50 MB
- Medium: ~100-200 MB
- Large: ~500 MB+

**Process tree:**
```
make
└── perl scripts/validate_data.pl
```

### Phase 6: Packaging (Python)

**What happens:**
- Python script `package.py` creates distribution package
- Generates dummy files to simulate package contents
- Computes checksums (memory intensive)
- Creates compressed tarball

**Memory characteristics:**
- Small: ~50-100 MB
- Medium: ~100-200 MB
- Large: ~200-400 MB

**Process tree:**
```
make
└── python3 scripts/package.py
```

## Demonstrating Memwatch Features

### 1. Process Filtering

Filter out shell overhead to focus on high-level processes:

```bash
# Exclude shell and make overhead
memwatch run --exclude 'sh|make' -- make SIZE=small

# Show only compilation-related processes
memwatch run --include 'rustc|cc|ld' -- make SIZE=small

# Show only high-level language processes
memwatch run --exclude 'cc|ld|as' -- make SIZE=small
```

### 2. Timeline Visualization

Export timeline data to see how memory usage evolves across phases:

```bash
# Export timeline and per-process data
memwatch run --timeline timeline.csv --csv processes.csv -- make SIZE=medium

# The timeline will show four distinct phases:
# 1. Codegen spike (Python)
# 2. Compilation plateau (rustc + linkers)
# 3. Testing spike (Node.js)
# 4. Packaging spike (Python)
```

You can plot the timeline with any spreadsheet tool or plotting library to visualize the phases.

### 3. Problem Size Scaling

Compare memory usage across different problem sizes:

```bash
# Small
memwatch run --json -- make SIZE=small > small.json

# Medium
memwatch run --json -- make SIZE=medium > medium.json

# Large
memwatch run --json -- make SIZE=large > large.json

# Compare max_total_rss_kib across all three
```

### 4. Parallel vs Sequential Memory Patterns

```bash
# Sequential: Peak memory is highest single phase
memwatch run --timeline sequential.csv -- make SIZE=medium

# Parallel: Peak memory is sum of concurrent phases
memwatch run --timeline parallel.csv -- make all-parallel SIZE=medium JOBS=4

# Compare the two timelines to see the difference
```

## Customization

### Environment Variables

All configurable via environment variables or make arguments:

- `SIZE`: Problem size (`small`, `medium`, `large`)
- `PROFILE`: Rust build profile (`debug`, `release`)
- `JOBS`: Parallel compilation jobs (default: 4)

Examples:
```bash
# Release build (more optimization, more memory, slower)
memwatch run -- make PROFILE=release SIZE=medium

# Many parallel jobs
memwatch run -- make all-parallel JOBS=8 SIZE=large
```

### Running Individual Phases

You can run phases independently:

```bash
# Just code generation
make codegen SIZE=large

# Just compilation (requires codegen first)
make build PROFILE=release

# Just tests (requires build first)
make test SIZE=medium

# Just packaging
make package SIZE=small
```

## Use Cases

### 1. Build Optimization

Find which phase of your build uses the most memory:

```bash
memwatch run --csv processes.csv -- make SIZE=large PROFILE=release

# Examine processes.csv to see peak memory per process
# Focus optimization efforts on the highest consumers
```

### 2. CI/CD Memory Planning

Determine how much RAM your CI runners need:

```bash
memwatch run --json -- make SIZE=medium PROFILE=release

# Check max_total_rss_kib to set runner memory limits
```

### 3. Parallel Build Tuning

Find the optimal number of parallel jobs:

```bash
# Try different JOBS values
memwatch run -- make all-parallel JOBS=2 SIZE=large
memwatch run -- make all-parallel JOBS=4 SIZE=large
memwatch run -- make all-parallel JOBS=8 SIZE=large

# Compare peak memory vs build time
```

### 4. Compiler Memory Profiling

Focus on just the compilation phase:

```bash
memwatch run --include 'rustc|cc|ld' -- make SIZE=large PROFILE=release

# See how rustc memory scales with codegen-units
```

## Expected Output

### Human-Readable Summary

```
Job: make
Duration: 45.2s  |  Samples: 91

=== MEMORY SUMMARY ===

  Total peak:    1.8 GiB

  Process peak:  892.4 MiB (pid 12345)

=== PER-PROCESS PEAKS ===

    PID     MEMORY      TIME  COMMAND
  12345   892.4 MiB    12.4s  rustc
  12346   234.1 MiB    15.2s  node scripts/run_tests.js
  12347   156.7 MiB     3.1s  python3 scripts/codegen.py
  12348   123.4 MiB    18.9s  python3 scripts/package.py
  ...
```

### Timeline CSV

The timeline shows four distinct phases:

| timestamp | elapsed_seconds | total_rss_kib | process_count |
|-----------|-----------------|---------------|---------------|
| 2025-... | 0.0 | 0 | 1 |
| 2025-... | 1.5 | 153600 | 2 |  ← Codegen
| 2025-... | 12.4 | 912384 | 8 |  ← Compilation (peak)
| 2025-... | 28.1 | 239616 | 3 |  ← Testing
| 2025-... | 35.7 | 126976 | 2 |  ← Packaging

## Troubleshooting

### Node.js Not Found

If you see "node: command not found":

```bash
# macOS
brew install node

# Ubuntu/Debian
sudo apt install nodejs npm

# Verify
node --version
```

### Binary Not Found in Tests

If tests fail with "binary not found", ensure compilation succeeded:

```bash
# Check if binary exists
ls -lh target/debug/build_example

# If missing, run compilation manually
make build SIZE=small
```

### Out of Memory

If the system runs out of memory with `SIZE=large`:

```bash
# Try medium size
memwatch run -- make SIZE=medium

# Or reduce parallel jobs
memwatch run -- make JOBS=2 SIZE=large
```

## Comparison with MPI Example

| Feature | MPI Example | Build Pipeline Example |
|---------|-------------|------------------------|
| **Languages** | Rust only | Python, C, Rust, Node.js, Perl (5 languages) |
| **Process Tree** | Flat (mpirun → ranks) | Deep (make → cargo → rustc → cc) |
| **Workload** | Distributed computation | Build pipeline |
| **Realism** | Scientific computing | CI/CD workflows |
| **Prerequisites** | OpenMPI | Python3, C compiler, Node.js, Perl |
| **Filtering Demo** | Rank filtering | Tool filtering |
| **Timeline** | Synchronized phases | Sequential phases |

## Files Structure

```
examples/build_pipeline/
├── Makefile              # Main orchestrator
├── README.md             # This file
├── Cargo.toml            # Rust project configuration
├── scripts/
│   ├── codegen.py       # Python code generation
│   ├── run_tests.js     # Node.js test runner
│   └── package.py       # Python packaging
├── src/                  # Generated Rust source (created by codegen.py)
├── target/               # Compiled binaries (created by cargo)
└── dist/                 # Distribution packages (created by package.py)
```

## Next Steps

1. **Run the example**: Start with `make SIZE=small` to see all phases
2. **Profile with memwatch**: Run `memwatch run -- make SIZE=medium`
3. **Try filtering**: Use `--exclude` and `--include` to focus on specific processes
4. **Export timeline**: Use `--timeline` to visualize memory phases
5. **Experiment with sizes**: Try `SIZE=large` to stress test

## See Also

- [Main examples README](../README.md) - Overview of all examples
- [MPI example](../mpi_distributed_compute.rs) - Distributed computation example
- [CLAUDE.md](../../CLAUDE.md) - Project documentation
