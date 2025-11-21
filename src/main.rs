use clap::Parser;
use memwatch::cli::{Cli, Commands};
use memwatch::inspector;
use memwatch::reporter;
use memwatch::sampler;
use std::process;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            interval,
            json,
            quiet,
            command,
        } => {
            if let Err(e) = run_command(command, interval, json, quiet) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
}

fn run_command(
    command: Vec<String>,
    interval_ms: u64,
    json: bool,
    quiet: bool,
) -> anyhow::Result<()> {
    // Create platform-specific inspector
    let inspector = inspector::create_inspector();

    // Run and profile the command
    let profile = sampler::run_and_profile(command, interval_ms, &inspector)?;

    // Output results
    if json {
        reporter::print_json(&profile)?;
    } else if !quiet {
        reporter::print_summary(&profile);
    }

    Ok(())
}
