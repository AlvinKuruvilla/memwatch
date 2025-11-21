use clap::Parser;
use memwatch::cli::{Cli, Commands};
use memwatch::csv_writer;
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
            csv,
            timeline,
            command,
        } => {
            if let Err(e) = run_command(command, interval, json, quiet, csv, timeline) {
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
    csv_path: Option<String>,
    timeline_path: Option<String>,
) -> anyhow::Result<()> {
    // Create platform-specific inspector
    let inspector = inspector::create_inspector();

    // Track timeline if requested
    let track_timeline = timeline_path.is_some();

    // Run and profile the command
    let profile = sampler::run_and_profile(command, interval_ms, track_timeline, &inspector)?;

    // Output results
    if json {
        reporter::print_json(&profile)?;
    } else if !quiet {
        reporter::print_summary(&profile);
    }

    // Export CSV if requested
    if let Some(path) = csv_path {
        csv_writer::export_process_csv(&profile, &path)?;
        if !quiet && !json {
            eprintln!("Per-process CSV exported to: {}", path);
        }
    }

    // Export timeline if requested
    if let Some(path) = timeline_path {
        csv_writer::export_timeline_csv(&profile, &path)?;
        if !quiet && !json {
            eprintln!("Timeline CSV exported to: {}", path);
        }
    }

    Ok(())
}
