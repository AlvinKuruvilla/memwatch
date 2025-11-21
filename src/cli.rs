use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "memwatch")]
#[command(about = "Cross-platform job-level memory profiler", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a command and profile its memory usage
    Run {
        /// Sampling interval in milliseconds
        #[arg(short, long, default_value = "500")]
        interval: u64,

        /// Output JSON instead of human-readable text
        #[arg(long)]
        json: bool,

        /// Suppress human-readable output (useful with --json)
        #[arg(long)]
        quiet: bool,

        /// Command to run (everything after --)
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },
}
