use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "memwatch")]
#[command(about = "Cross-platform job-level memory profiler", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn get_long_version() -> String {
        format!(
            "{}\nBuild date:   {}\nTarget:       {}",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_DATE").unwrap_or("unknown"),
            option_env!("BUILD_TARGET").unwrap_or("unknown")
        )
    }
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

        /// Export per-process peak RSS to CSV file
        #[arg(long, value_name = "FILE")]
        csv: Option<String>,

        /// Export time-series memory data to CSV file
        #[arg(long, value_name = "FILE")]
        timeline: Option<String>,

        /// Suppress command output (hide stdout/stderr from the profiled command)
        #[arg(long)]
        silent: bool,

        /// Exclude processes matching regex pattern from output (can be combined with --include)
        #[arg(long, value_name = "PATTERN")]
        exclude: Option<String>,

        /// Only include processes matching regex pattern in output (can be combined with --exclude)
        #[arg(long, value_name = "PATTERN")]
        include: Option<String>,

        /// Command to run (everything after --)
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },
}
