use crate::types::JobProfile;
use anyhow::Result;

/// Format bytes in KiB to human-readable format (KiB, MiB, GiB)
fn format_memory(kib: u64) -> String {
    const KIB_IN_MIB: f64 = 1024.0;
    const KIB_IN_GIB: f64 = 1024.0 * 1024.0;

    let kib_f64 = kib as f64;

    if kib_f64 >= KIB_IN_GIB {
        format!("{:.1} GiB", kib_f64 / KIB_IN_GIB)
    } else if kib_f64 >= KIB_IN_MIB {
        format!("{:.1} MiB", kib_f64 / KIB_IN_MIB)
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

/// Print human-readable summary
pub fn print_summary(profile: &JobProfile) {
    println!("\nJob: {}", profile.command.join(" "));
    println!("Duration:        {}", format_duration(profile.duration_seconds));
    println!("Samples:         {}", profile.samples);
    println!();

    // Filter out processes with 0 RSS for display
    let valid_processes: Vec<_> = profile.processes.iter()
        .filter(|p| p.max_rss_kib > 0)
        .collect();

    if profile.max_total_rss_kib == 0 || valid_processes.is_empty() {
        println!("Max total RSS:   {} (process exited too quickly to measure)",
                 format_memory(profile.max_total_rss_kib));
        println!("\nNote: The command completed before memory could be sampled.");
        println!("For very short-running commands, try using a shorter sampling interval (-i).");
    } else {
        println!("Max total RSS:   {}", format_memory(profile.max_total_rss_kib));

        if let Some(max_process) = valid_processes.first() {
            println!(
                "Max per process: {} (pid {})",
                format_memory(max_process.max_rss_kib),
                max_process.pid
            );
        }

        println!("\nPer-process peak RSS:");
        for proc in valid_processes {
            // Truncate long command lines
            let cmd = if proc.command.len() > 60 {
                format!("{}...", &proc.command[..57])
            } else {
                proc.command.clone()
            };

            println!(
                "  pid {:5}  {:>10}  {}",
                proc.pid,
                format_memory(proc.max_rss_kib),
                cmd
            );
        }
    }
    println!();
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
