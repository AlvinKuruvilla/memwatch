use clap::{CommandFactory, FromArgMatches};
use memwatch::cli::{Cli, Commands};
use memwatch::csv_writer;
use memwatch::inspector;
use memwatch::reporter;
use memwatch::sampler;
use std::process;

fn main() {
    // Create command with extended version info and parse
    let matches = Cli::command()
        .long_version(Cli::get_long_version())
        .get_matches();

    let cli = Cli::from_arg_matches(&matches)
        .map_err(|e| e.exit())
        .unwrap();

    match cli.command {
        Commands::Run {
            interval,
            json,
            quiet,
            csv,
            timeline,
            silent,
            exclude,
            include,
            command,
        } => {
            match run_command(command, interval, json, quiet, csv, timeline, silent, exclude, include) {
                Ok(exit_code) => {
                    // Exit with the child process's exit code
                    process::exit(exit_code);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
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
    silent: bool,
    exclude: Option<String>,
    include: Option<String>,
) -> anyhow::Result<i32> {
    // Create platform-specific inspector
    let inspector = inspector::create_inspector();

    // Track timeline if requested
    let track_timeline = timeline_path.is_some();

    // Run and profile the command
    let profile = sampler::run_and_profile(command, interval_ms, track_timeline, silent, exclude, include, &inspector)?;

    // Capture exit code before consuming profile
    let exit_code = profile.exit_code.unwrap_or(0);

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

    Ok(exit_code)
}
