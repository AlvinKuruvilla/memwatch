//! Distributed Computation Example for MPI Memory Profiling
//!
//! This example simulates a realistic MPI workload inspired by distributed zero-knowledge
//! proof systems (like zeroasset2). It demonstrates memory patterns typical of HPC applications
//! that are challenging to track with traditional tools like `top` or `htop`.
//!
//! ## Memory Characteristics
//!
//! - **Distributed arrays**: Each MPI rank allocates a portion of total data
//! - **Per-rank scaling**: Memory per process = O(n/np) where n=problem size, np=process count
//! - **Working buffers**: Additional temporary allocations during computation
//! - **Communication spikes**: Temporary buffers during MPI collective operations
//! - **Short-lived execution**: Typical runs complete in 5-10 seconds
//!
//! ## Usage
//!
//! ```bash
//! # Basic usage with 4 processes
//! mpirun -n 4 cargo run --example mpi_distributed_compute
//!
//! # Custom problem size (16384 elements)
//! mpirun -n 8 cargo run --example mpi_distributed_compute -- --size 16384
//!
//! # Profile with memwatch
//! memwatch run -- mpirun -n 4 cargo run --release --example mpi_distributed_compute
//! ```
//!
//! ## What This Demonstrates
//!
//! This example shows why memwatch is valuable for MPI workloads:
//!
//! 1. **Process tree tracking**: All MPI ranks tracked as single job
//! 2. **Per-rank memory**: Inversely proportional to process count
//! 3. **Peak memory detection**: Captures spikes during communication phases
//! 4. **Short execution**: Traditional tools often miss peak memory in quick runs
//! 5. **Compilation vs execution**: memwatch separates cargo build from actual execution
//!
//! ## Realistic Use Case
//!
//! This pattern mirrors real distributed systems like:
//! - Zero-knowledge proof systems (distributed polynomial operations)
//! - Distributed FFT computations (DIZK algorithm)
//! - Scientific simulations with domain decomposition
//! - Financial privacy systems (like zeroasset2's solvency proofs)

use mpi::traits::*;
use std::env;
use std::time::Duration;

/// Field element size in bytes (simulates BN254/BLS12-381 field elements)
const FIELD_ELEMENT_SIZE: usize = 32;

/// Default problem size (total number of field elements)
const DEFAULT_PROBLEM_SIZE: usize = 8192;

/// Sleep duration during computation phase (milliseconds)
const COMPUTE_DURATION_MS: u64 = 2000;

/// Configuration for the distributed computation
#[derive(Debug)]
struct ComputeConfig {
    /// Total problem size (number of field elements)
    problem_size: usize,
    /// MPI rank of this process
    rank: i32,
    /// Total number of MPI processes
    world_size: i32,
}

impl ComputeConfig {
    fn new(problem_size: usize, rank: i32, world_size: i32) -> Self {
        Self {
            problem_size,
            rank,
            world_size,
        }
    }

    /// Calculate the chunk size for this rank
    fn chunk_size(&self) -> usize {
        let base_size = self.problem_size / self.world_size as usize;
        let remainder = self.problem_size % self.world_size as usize;

        // Last rank gets the remainder
        if self.rank == self.world_size - 1 {
            base_size + remainder
        } else {
            base_size
        }
    }

    /// Calculate starting index for this rank's chunk
    fn chunk_start(&self) -> usize {
        (self.problem_size / self.world_size as usize) * self.rank as usize
    }
}

/// Simulates a field element (32-byte array)
type FieldElement = [u8; FIELD_ELEMENT_SIZE];

/// Distributed vector holding a partition of field elements
struct DistributedVector {
    /// Local partition of the distributed vector
    data: Vec<FieldElement>,
}

impl DistributedVector {
    /// Create a new distributed vector for this rank
    fn new(config: &ComputeConfig) -> Self {
        let chunk_size = config.chunk_size();
        let start_index = config.chunk_start();

        println!(
            "[Rank {}] Allocating chunk: {} elements ({} bytes)",
            config.rank,
            chunk_size,
            chunk_size * FIELD_ELEMENT_SIZE
        );

        // Allocate local partition
        let mut data = Vec::with_capacity(chunk_size);
        for i in 0..chunk_size {
            // Fill with deterministic "random" data based on global index
            let global_idx = start_index + i;
            let mut elem = [0u8; FIELD_ELEMENT_SIZE];
            elem[0..8].copy_from_slice(&global_idx.to_le_bytes());
            data.push(elem);
        }

        Self { data }
    }

    /// Get memory usage in bytes
    fn memory_bytes(&self) -> usize {
        self.data.len() * FIELD_ELEMENT_SIZE
    }
}

/// Working buffers for computation (simulates FFT workspace)
struct WorkingBuffers {
    /// Temporary buffer for in-place operations
    temp_buffer: Vec<FieldElement>,
    /// Scratch space for intermediate results
    scratch: Vec<FieldElement>,
}

impl WorkingBuffers {
    /// Allocate working buffers (2x the data size)
    fn new(size: usize, rank: i32) -> Self {
        println!(
            "[Rank {}] Allocating working buffers: 2x {} elements ({} bytes)",
            rank,
            size,
            2 * size * FIELD_ELEMENT_SIZE
        );

        Self {
            temp_buffer: vec![[0u8; FIELD_ELEMENT_SIZE]; size],
            scratch: vec![[0u8; FIELD_ELEMENT_SIZE]; size],
        }
    }

    /// Get total memory usage in bytes
    fn memory_bytes(&self) -> usize {
        (self.temp_buffer.len() + self.scratch.len()) * FIELD_ELEMENT_SIZE
    }
}

/// Simulate computation on distributed data
fn compute_phase(
    config: &ComputeConfig,
    dist_vec: &mut DistributedVector,
    buffers: &mut WorkingBuffers,
) {
    println!("[Rank {}] Starting computation phase...", config.rank);

    // Simulate computation by modifying data
    for (i, elem) in dist_vec.data.iter_mut().enumerate() {
        // Simple operation: XOR with index
        elem[0] ^= (i & 0xFF) as u8;

        // Use buffers for temporary storage
        buffers.temp_buffer[i] = *elem;
    }

    // Hold memory for observation
    std::thread::sleep(Duration::from_millis(COMPUTE_DURATION_MS));

    println!("[Rank {}] Computation phase complete", config.rank);
}

/// Simulate MPI communication spike (gather/broadcast pattern)
fn communication_phase<C: mpi::topology::Communicator>(config: &ComputeConfig, world: &C) {
    println!("[Rank {}] Starting communication phase...", config.rank);

    // Simulate serialization buffer spike during MPI operations
    let spike_size = 256; // Small spike for gather metadata
    let _comm_buffer: Vec<FieldElement> = vec![[0u8; FIELD_ELEMENT_SIZE]; spike_size];

    println!(
        "[Rank {}] Allocated temporary communication buffer: {} bytes",
        config.rank,
        spike_size * FIELD_ELEMENT_SIZE
    );

    // Synchronize all ranks
    world.barrier();

    // Simulate communication delay
    std::thread::sleep(Duration::from_millis(500));

    println!("[Rank {}] Communication phase complete", config.rank);
    // _comm_buffer dropped here, simulating cleanup after communication
}

/// Print memory summary for this rank
fn print_memory_summary(
    config: &ComputeConfig,
    dist_vec: &DistributedVector,
    buffers: &WorkingBuffers,
) {
    let total_mb = (dist_vec.memory_bytes() + buffers.memory_bytes()) as f64 / (1024.0 * 1024.0);

    println!(
        "[Rank {}] Memory summary: {:.2} MiB (data: {:.2} MiB, buffers: {:.2} MiB)",
        config.rank,
        total_mb,
        dist_vec.memory_bytes() as f64 / (1024.0 * 1024.0),
        buffers.memory_bytes() as f64 / (1024.0 * 1024.0)
    );
}

fn main() {
    // Initialize MPI
    let universe = mpi::initialize().expect("Failed to initialize MPI");
    let world = universe.world();
    let rank = world.rank();
    let world_size = world.size();

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let problem_size = if args.len() > 2 && args[1] == "--size" {
        args[2].parse().unwrap_or(DEFAULT_PROBLEM_SIZE)
    } else {
        DEFAULT_PROBLEM_SIZE
    };

    // Create configuration
    let config = ComputeConfig::new(problem_size, rank, world_size);

    if rank == 0 {
        println!("=== MPI Distributed Computation Example ===");
        println!("Problem size: {} field elements", config.problem_size);
        println!("World size: {} processes", world_size);
        println!(
            "Total memory (theoretical): {:.2} MiB",
            (config.problem_size * FIELD_ELEMENT_SIZE * 3) as f64 / (1024.0 * 1024.0)
        );
        println!();
    }

    world.barrier();

    // Phase 1: Initialize distributed data
    println!("[Rank {}] === Phase 1: Initialization ===", rank);
    let mut dist_vec = DistributedVector::new(&config);

    world.barrier();

    // Phase 2: Allocate working buffers
    println!("[Rank {}] === Phase 2: Buffer Allocation ===", rank);
    let mut buffers = WorkingBuffers::new(config.chunk_size(), rank);

    print_memory_summary(&config, &dist_vec, &buffers);

    world.barrier();

    // Phase 3: Computation
    println!("[Rank {}] === Phase 3: Computation ===", rank);
    compute_phase(&config, &mut dist_vec, &mut buffers);

    world.barrier();

    // Phase 4: Communication
    println!("[Rank {}] === Phase 4: Communication ===", rank);
    communication_phase(&config, &world);

    world.barrier();

    // Phase 5: Cleanup
    println!("[Rank {}] === Phase 5: Cleanup ===", rank);
    drop(buffers);
    drop(dist_vec);

    println!("[Rank {}] Cleanup complete", rank);

    world.barrier();

    if rank == 0 {
        println!("\n=== Execution Complete ===");
        println!("Use 'memwatch run -- mpirun -n {} ...' to profile this workload", world_size);
    }
}
