use crate::types::{JobProfile, memory};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;

/// Write filter metadata as CSV comment header
fn write_filter_comment(file: &mut File, profile: &JobProfile, include_stats: bool) -> Result<()> {
    if let Some(ref filter) = profile.filter {
        write!(file, "# Filter: {}", filter.to_csv_comment())?;

        if include_stats {
            if let (Some(filtered_count), Some(filtered_rss)) = (
                profile.filtered_process_count,
                profile.filtered_total_rss_kib,
            ) {
                writeln!(
                    file,
                    " ({} processes filtered out, totaling {} KiB)",
                    filtered_count, filtered_rss
                )?;
            } else {
                writeln!(file)?;
            }
        } else {
            writeln!(file)?;
            writeln!(
                file,
                "# Note: total_rss_kib and process_count both show all processes (unfiltered)"
            )?;
            writeln!(
                file,
                "# Filtering only affects the per-process CSV export and final summary display"
            )?;
        }
    }
    Ok(())
}

/// Export per-process peak RSS to CSV
pub fn export_process_csv(profile: &JobProfile, path: &str) -> Result<()> {
    let mut file =
        File::create(path).context(format!("Failed to create per-process CSV file: {}", path))?;

    write_filter_comment(&mut file, profile, true)?;

    // Write header
    writeln!(
        file,
        "pid,ppid,command,max_rss_kib,max_rss_mib,first_seen,last_seen"
    )?;

    // Write each process (filter out processes with 0 RSS)
    for proc in profile.processes.iter().filter(|p| p.max_rss_kib > 0) {
        let max_rss_mib = proc.max_rss_kib as f64 / memory::KIB_PER_MIB;
        writeln!(
            file,
            "{},{},\"{}\",{},{:.2},{},{}",
            proc.pid,
            proc.ppid,
            escape_csv(&proc.command),
            proc.max_rss_kib,
            max_rss_mib,
            proc.first_seen.to_rfc3339(),
            proc.last_seen.to_rfc3339()
        )?;
    }

    Ok(())
}

/// Export timeline data to CSV
pub fn export_timeline_csv(profile: &JobProfile, path: &str) -> Result<()> {
    let mut file =
        File::create(path).context(format!("Failed to create timeline CSV file: {}", path))?;

    let timeline = profile.timeline.as_ref()
        .context("Timeline data not available. This is a bug - timeline should be tracked when --timeline is used.")?;

    write_filter_comment(&mut file, profile, false)?;

    // Write header
    writeln!(
        file,
        "timestamp,elapsed_seconds,total_rss_kib,total_rss_mib,process_count"
    )?;

    // Write each timeline point
    for point in timeline {
        let total_rss_mib = point.total_rss_kib as f64 / memory::KIB_PER_MIB;
        writeln!(
            file,
            "{},{:.3},{},{:.2},{}",
            point.timestamp.to_rfc3339(),
            point.elapsed_seconds,
            point.total_rss_kib,
            total_rss_mib,
            point.process_count
        )?;
    }

    Ok(())
}

/// Escape CSV field values
fn escape_csv(s: &str) -> String {
    // Replace quotes with double quotes
    s.replace('"', "\"\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("hello"), "hello");
        assert_eq!(escape_csv("hello \"world\""), "hello \"\"world\"\"");
        assert_eq!(escape_csv("test"), "test");
    }
}
