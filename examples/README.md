# Memwatch Examples

This directory contains example programs demonstrating memwatch's capabilities for profiling different types of workloads.

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
memwatch run -- mpirun -n 4 target/release/examples/mpi_distributed_compute

# With JSON output for automated analysis
memwatch run --json -- mpirun -n 4 target/release/examples/mpi_distributed_compute

# With CSV export for per-process breakdown
memwatch run --csv mpi_processes.csv -- mpirun -n 4 target/release/examples/mpi_distributed_compute

# With timeline export for plotting memory over time
memwatch run --timeline mpi_timeline.csv -- mpirun -n 4 target/release/examples/mpi_distributed_compute
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
