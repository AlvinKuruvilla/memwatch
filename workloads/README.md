# Memwatch Workloads

This directory contains demonstration workloads showing memwatch's capabilities for profiling different types of programs.

## Quick Overview

| Example | Languages | Use Case | Key Feature |
|---------|-----------|----------|-------------|
| [Build Pipeline](#build-pipeline) | Python, C, Rust, Node.js, Perl | CI/CD workflows | Multi-program process trees (5 languages) |
| [MPI Distributed Compute](#mpi-distributed-computation) | Rust (via MPI) | HPC/Scientific computing | Distributed memory tracking |

## Build Pipeline

**Directory:** `build_pipeline/`

A comprehensive multi-program example that simulates a realistic CI/CD build pipeline with code generation (Python), C library compilation (cc), Rust compilation (rustc → cc → ld), testing (Node.js), data validation (Perl), and packaging (Python). Spans **5 programming languages**.

### Quick Start

```bash
cd workloads/build_pipeline

# Run the full pipeline
memwatch run -- make

# Try different problem sizes
memwatch run -- make SIZE=large

# See all options
make help
```

### What This Demonstrates

- **Multi-language tracking**: Python, C, Rust compiler, Node.js, Perl all in one job (5 languages)
- **Complex process trees**: Real compilation spawns cargo → rustc → cc → ld (4+ levels deep)
- **Memory phases**: Six distinct phases with different memory characteristics
- **Process filtering**: Many helper processes that can be filtered with `--exclude` and `--include`
- **Timeline visualization**: Clear phases visible in `--timeline` output
- **Configurable workload**: Small/medium/large problem sizes
- **Parallel vs sequential**: Compare `make` vs `make -j` memory patterns

### Use Cases

1. **Build optimization**: Find which build phase uses the most memory
2. **CI/CD planning**: Determine how much RAM your CI runners need
3. **Compiler profiling**: Understand rustc memory usage with different codegen-units
4. **Tool comparison**: See memory differences between debug vs release builds

See [build_pipeline/README.md](build_pipeline/README.md) for complete documentation.

## MPI Distributed Computation

**File:** `mpi_distributed_compute.rs`

A realistic MPI example inspired by distributed zero-knowledge proof systems (like [zeroasset2](https://github.com/privacy-scaling-explorations/zeroasset2)). This demonstrates memwatch's value for tracking memory across MPI process trees - a challenge where traditional tools struggle.

### Prerequisites

You need OpenMPI or MPICH installed on your system:

**macOS:**
```bash
brew install open-mpi
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install libopenmpi-dev openmpi-bin
```

**Linux (Fedora/RHEL):**
```bash
sudo dnf install openmpi openmpi-devel
```

### Building and Running

#### Basic Usage

```bash
# Run with 4 processes (default problem size: 8192 elements)
mpirun -n 4 cargo run --example mpi_distributed_compute

# Run with 8 processes
mpirun -n 8 cargo run --example mpi_distributed_compute

# Custom problem size (16384 elements)
mpirun -n 4 cargo run --example mpi_distributed_compute -- --size 16384
```

#### Profiling with Memwatch

This is the primary use case - tracking memory across all MPI ranks:

```bash
# Build in release mode first (optional but recommended for realistic memory patterns)
cargo build --release --example mpi_distributed_compute

# Profile with memwatch
memwatch run -- mpirun -n 4 target/release/workloads/mpi_distributed_compute

# With JSON output for automated analysis
memwatch run --json -- mpirun -n 4 target/release/workloads/mpi_distributed_compute

# With CSV export for per-process breakdown
memwatch run --csv mpi_processes.csv -- mpirun -n 4 target/release/workloads/mpi_distributed_compute

# With timeline export for plotting memory over time
memwatch run --timeline mpi_timeline.csv -- mpirun -n 4 target/release/workloads/mpi_distributed_compute
```

### What This Example Demonstrates

#### 1. Process Tree Tracking

MPI spawns multiple processes that memwatch tracks as a single job:

```
mpirun
├── cargo (if using cargo run)
├── rank 0 process
├── rank 1 process
├── rank 2 process
└── rank 3 process
```

Memwatch correctly aggregates memory across all ranks to show total job memory.

#### 2. Memory Scaling Patterns

**Per-Process Memory Scales Inversely with Process Count:**

| Processes | Problem Size | Per-Process Memory | Total Job Memory |
|-----------|--------------|-------------------|------------------|
| 2         | 8192         | ~384 KiB          | ~768 KiB         |
| 4         | 8192         | ~192 KiB          | ~768 KiB         |
| 8         | 8192         | ~96 KiB           | ~768 KiB         |

This is typical of distributed systems with domain decomposition. Traditional tools show individual process memory, but memwatch shows the complete picture.

#### 3. Memory Phases

The example simulates realistic computation phases:

- **Initialization**: Small setup allocations
- **Data Distribution**: Each rank allocates `n/np` elements (where n=problem size, np=process count)
- **Working Buffers**: 2x allocation for computational workspace (typical for FFT operations)
- **Communication Spikes**: Temporary buffers during MPI collective operations
- **Cleanup**: Memory deallocation

#### 4. Short Execution Detection

The example runs for ~5-10 seconds. Traditional tools often miss peak memory in such quick runs, but memwatch:
- Samples immediately after process spawn
- Configurable sampling intervals (`-i` flag)
- Captures peak memory even for short-lived processes

### Real-World Applications

This pattern mirrors actual distributed systems:

#### Zero-Knowledge Proof Systems

Projects like **zeroasset2** use MPI for distributed polynomial operations:
- **Distributed FFT (DIZK algorithm)**: Splits large polynomial evaluations across nodes
- **Field arithmetic**: Each rank holds portion of coefficient vectors
- **Communication-heavy**: FFT transposes require all-to-all exchanges
- **Memory-intensive**: Cryptographic field elements (32 bytes each), working buffers, commitment keys

Example from zeroasset2:
```rust
// Each rank in a 4-node cluster holds 1/4 of a polynomial
let chunk_size = poly_degree / num_ranks;
let my_coeffs: Vec<FieldElement> = allocate_chunk(chunk_size);  // ~1GB per rank
let fft_workspace: Vec<FieldElement> = vec![...; chunk_size];   // Another 1GB
// Total job memory: 4 ranks × 2GB = 8GB peak
```

#### HPC Scientific Computing

- **Finite element simulations**: Domain decomposition across MPI ranks
- **Molecular dynamics**: Particle distribution with periodic communication
- **Climate modeling**: Spatial grid partitioning

#### Financial Privacy Systems

- **Proof of solvency**: Distributed Merkle tree computation
- **Regulatory compliance**: Privacy-preserving aggregate calculations
- **Risk analysis**: Monte Carlo simulations across cluster

### Why Memwatch vs Traditional Tools

**Problem with `top`/`htop`:**
```bash
# Run MPI job
$ mpirun -n 4 ./my_mpi_app

# In another terminal, try to track it with top
$ top -p $(pgrep -d',' my_mpi_app)
```

Issues:
1. Must manually identify all PIDs (parent + all ranks)
2. Must mentally sum memory across all processes
3. Slow sampling misses short-lived processes
4. Hard to separate compilation from execution with `cargo run`
5. No automated reporting for CI/benchmarking

**With memwatch:**
```bash
$ memwatch run -- mpirun -n 4 ./my_mpi_app
```

Benefits:
1. Automatically tracks entire process tree
2. Shows total job memory (sum across all processes)
3. Shows per-process breakdown
4. Configurable sampling intervals
5. JSON/CSV output for automation
6. Separates cargo build from execution

### Expected Output

When run with memwatch, you should see output like:

```
Duration: 5.234s
Samples: 11

Max total RSS: 2.25 MiB

Per-process peak RSS:
  PID 12345 (mpirun):              1.23 MiB
  PID 12346 (mpi_distributed_compute): 384.0 KiB
  PID 12347 (mpi_distributed_compute): 384.0 KiB
  PID 12348 (mpi_distributed_compute): 384.0 KiB
  PID 12349 (mpi_distributed_compute): 384.0 KiB
```

### Troubleshooting

**MPI not found during compilation:**
```
error: failed to run custom build command for `mpi-sys`
```

Solution: Ensure MPI is in your PATH. For macOS with Homebrew:
```bash
export PATH="/usr/local/opt/open-mpi/bin:$PATH"
```

**Cannot find mpirun:**
```bash
# macOS/Linux
which mpirun

# If not found, install OpenMPI (see Prerequisites above)
```

**Permission errors:**

Some HPC systems require module loading:
```bash
module load openmpi
mpirun -n 4 cargo run --example mpi_distributed_compute
```

### Further Reading

- [OpenMPI Documentation](https://www.open-mpi.org/doc/)
- [Rust MPI Crate](https://docs.rs/mpi/)
- [zeroasset2 Project](https://github.com/privacy-scaling-explorations/zeroasset2) - Real-world distributed ZK proofs
- [DIZK Paper](https://eprint.iacr.org/2018/691) - Distributed FFT algorithms for ZK systems

## Choosing an Example

### When to Use Build Pipeline Example

Use the **build pipeline** example when you want to:
- Profile multi-language workflows (Python + Rust + Node.js)
- Understand build system memory usage
- Demonstrate deep process trees (3+ levels)
- Show sequential phases with distinct memory patterns
- Test memwatch's filtering capabilities (many helper processes)
- Profile CI/CD pipelines

**Best for:** Development workflows, build optimization, CI/CD planning

### When to Use MPI Example

Use the **MPI distributed compute** example when you want to:
- Profile HPC/scientific computing workloads
- Track distributed memory across multiple ranks
- Understand memory scaling with process count
- Demonstrate parallel computation patterns
- Profile zero-knowledge proof systems or similar

**Best for:** HPC applications, distributed systems, scientific computing

## Example Comparison

| Aspect | Build Pipeline | MPI Distributed Compute |
|--------|----------------|-------------------------|
| **Languages** | Python, C, Rust, Node.js, Perl (5 total) | Rust (all processes same binary) |
| **Process Tree Depth** | Deep (3-4 levels) | Flat (mpirun + ranks) |
| **Process Tree Width** | Narrow (1-3 concurrent) | Wide (np ranks) |
| **Memory Pattern** | Sequential phases | Parallel uniform |
| **Execution Time** | 10-60 seconds | 5-10 seconds |
| **Prerequisites** | python3, cc, node, perl | OpenMPI/MPICH |
| **Workload Type** | Build pipeline | Distributed computation |
| **Filtering Demo** | Excellent (many tools) | Good (rank filtering) |
| **Timeline Phases** | Distinct (6 phases) | Synchronized (1 phase) |
| **Realism** | CI/CD workflows | Scientific computing |
| **Complexity** | High (multi-tool) | Moderate (single tool) |

## Contributing Examples

Have an interesting memwatch use case? Consider contributing an example:

1. **Target a specific domain**: Build systems, data pipelines, web servers, etc.
2. **Show memwatch features**: Process trees, filtering, timeline, etc.
3. **Use realistic workloads**: Real tools and actual operations
4. **Include documentation**: README with setup, usage, and expected output
5. **Cross-platform support**: Works on both macOS and Linux

See [CLAUDE.md](../CLAUDE.md) for project contribution guidelines.
