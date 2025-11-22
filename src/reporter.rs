use crate::types::{memory, JobProfile};
use anyhow::Result;
use std::collections::HashMap;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Format bytes in KiB to human-readable format (KiB, MiB, GiB)
fn format_memory(kib: u64) -> String {
    let kib_f64 = kib as f64;

    if kib_f64 >= memory::KIB_PER_GIB {
        format!("{:.1} GiB", kib_f64 / memory::KIB_PER_GIB)
    } else if kib_f64 >= memory::KIB_PER_MIB {
        format!("{:.1} MiB", kib_f64 / memory::KIB_PER_MIB)
    } else {
        format!("{} KiB", kib)
    }
}

/// Format duration in seconds to HH:MM:SS
fn format_duration(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

/// Extract command name from full command line
fn extract_command_name(command: &str) -> String {
    // Take first word (command name)
    let first_word = command.split_whitespace().next().unwrap_or(command);

    // Get basename from path
    if let Some(pos) = first_word.rfind('/') {
        first_word[pos + 1..].to_string()
    } else {
        first_word.to_string()
    }
}

/// Print human-readable summary with colors and compact formatting
pub fn print_summary(profile: &JobProfile) {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    // Job header
    println!("\nJob: {}", profile.command.join(" "));
    print!("Duration: {}  |  Samples: {}",
           format_duration(profile.duration_seconds),
           profile.samples);
    println!();

    // Filter out processes with 0 RSS for display
    let valid_processes: Vec<_> = profile.processes.iter()
        .filter(|p| p.max_rss_kib > 0)
        .collect();

    if profile.max_total_rss_kib == 0 {
        // No data captured at all
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
        println!("\nMax total RSS: {} (no data captured)",
                 format_memory(profile.max_total_rss_kib));
        let _ = stdout.reset();

        println!("\n⚠ Warning: The command completed too quickly to capture memory usage.");
        println!("\nPossible reasons:");
        println!("  • Command executed in < {}ms (sampling interval)", profile.interval_ms);
        println!("  • Process spawned child and immediately exited");
        println!("  • Command failed or was killed immediately");
        println!("\nSuggestions:");
        println!("  • Use a shorter interval: memwatch run -i 50 -- <command>");
        println!("  • Check if the command actually ran: echo $?");
        println!("  • For instant commands (like 'echo'), memory profiling may not be useful");
    } else if valid_processes.is_empty() && profile.filter.is_some() {
        // All processes were filtered out
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
        println!("\n⚠ Warning: All processes were filtered out.");
        let _ = stdout.reset();
        println!("\nTotal job memory was {}, but no processes match the filter criteria.",
                 format_memory(profile.max_total_rss_kib));

        if let Some(ref filter) = profile.filter {
            println!("\nActive filters:");
            for line in filter.display_patterns() {
                println!("  • {}", line);
            }
        }

        println!("\nSuggestions:");
        println!("  • Check your filter patterns for typos");
        println!("  • Use broader patterns (e.g., 'test' instead of '^test$')");
        println!("  • Run without filters to see all processes: memwatch run --json -- <command>");
    } else if valid_processes.is_empty() {
        // No valid processes (no filter applied)
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
        println!("\nMax total RSS: {} (no data captured)",
                 format_memory(profile.max_total_rss_kib));
        let _ = stdout.reset();

        println!("\n⚠ Warning: The command completed too quickly to capture memory usage.");
        println!("\nPossible reasons:");
        println!("  • Command executed in < {}ms (sampling interval)", profile.interval_ms);
        println!("  • Process spawned child and immediately exited");
        println!("  • Command failed or was killed immediately");
        println!("\nSuggestions:");
        println!("  • Use a shorter interval: memwatch run -i 50 -- <command>");
        println!("  • Check if the command actually ran: echo $?");
        println!("  • For instant commands (like 'echo'), memory profiling may not be useful");
    } else {
        // Memory summary section
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true));
        print!("\nMEMORY SUMMARY");
        let _ = stdout.reset();
        println!();

        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
        print!("  Total peak:    {}", format_memory(profile.max_total_rss_kib));
        let _ = stdout.reset();

        // Show filtering info if applicable
        if profile.filter.is_some() {
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true));
            print!(" (all processes)");
            let _ = stdout.reset();
        }
        println!();

        if let Some(max_process) = valid_processes.first() {
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
            print!("  Process peak:  {} ", format_memory(max_process.max_rss_kib));
            let _ = stdout.reset();
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true));
            print!("(pid {})", max_process.pid);
            let _ = stdout.reset();
            println!();
        }

        // Per-process peaks table
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true));
        print!("\nPER-PROCESS PEAKS");
        let _ = stdout.reset();

        // Show filter annotation in header if applicable
        if let (Some(filtered_count), Some(filtered_rss)) = (profile.filtered_process_count, profile.filtered_total_rss_kib) {
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true));
            print!(" ({} processes filtered out, {} total)", filtered_count, format_memory(filtered_rss));
            let _ = stdout.reset();
        }
        println!();

        // Table header
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true));
        println!("  {:>5}  {:>10}  {:>8}  {}", "PID", "MEMORY", "TIME", "COMMAND");
        let _ = stdout.reset();

        // Table rows
        for proc in valid_processes {
            let elapsed_secs = (proc.peak_time - profile.start_time).num_milliseconds() as f64 / 1000.0;

            // PID (dimmed)
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true));
            print!("  {:>5}  ", proc.pid);
            let _ = stdout.reset();

            // Memory (green)
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
            print!("{:>10}  ", format_memory(proc.max_rss_kib));
            let _ = stdout.reset();

            // Time (yellow)
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
            print!("@ {:5.1}s  ", elapsed_secs);
            let _ = stdout.reset();

            // Command (default)
            println!("{}", proc.command);
        }

        // Process groups table
        let groups = compute_process_groups(&profile.processes);
        if groups.len() > 1 {
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true));
            print!("\nPROCESS GROUPS");
            let _ = stdout.reset();
            println!();

            // Table header
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true));
            println!("  {:24}  {:>9}  {:>12}", "COMMAND", "PROCESSES", "TOTAL PEAK");
            let _ = stdout.reset();

            // Sort by total RSS (descending)
            let mut group_vec: Vec<_> = groups.into_iter().collect();
            group_vec.sort_by_key(|(_, total)| std::cmp::Reverse(*total));

            // Table rows
            for (cmd_name, (count, total_rss)) in group_vec {
                print!("  {:24}  ", cmd_name);

                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true));
                print!("{:>9}  ", count);
                let _ = stdout.reset();

                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
                print!("{:>12}", format_memory(total_rss));
                let _ = stdout.reset();
                println!();
            }
        }
    }
    println!();
}

/// Compute process groups by command name
fn compute_process_groups(processes: &[crate::types::ProcessStats]) -> HashMap<String, (usize, u64)> {
    let mut groups: HashMap<String, (usize, u64)> = HashMap::new();

    for proc in processes.iter().filter(|p| p.max_rss_kib > 0) {
        let cmd_name = extract_command_name(&proc.command);
        groups
            .entry(cmd_name)
            .and_modify(|(count, total)| {
                *count += 1;
                *total += proc.max_rss_kib;
            })
            .or_insert((1, proc.max_rss_kib));
    }

    groups
}

/// Print JSON output
pub fn print_json(profile: &JobProfile) -> Result<()> {
    let json = serde_json::to_string_pretty(profile)?;
    println!("{}", json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_memory() {
        assert_eq!(format_memory(512), "512 KiB");
        assert_eq!(format_memory(1024), "1.0 MiB");
        assert_eq!(format_memory(2048), "2.0 MiB");
        assert_eq!(format_memory(1024 * 1024), "1.0 GiB");
        assert_eq!(format_memory(1024 * 1024 * 2), "2.0 GiB");
        assert_eq!(format_memory(1536 * 1024), "1.5 GiB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.0), "00:00:00");
        assert_eq!(format_duration(59.5), "00:00:59");
        assert_eq!(format_duration(60.0), "00:01:00");
        assert_eq!(format_duration(3661.0), "01:01:01");
        assert_eq!(format_duration(7384.0), "02:03:04");
    }
}
