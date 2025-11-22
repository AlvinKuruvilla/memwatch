use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory unit conversion constants
pub mod memory {
    pub const KIB_PER_MIB: f64 = 1024.0;
    pub const MIB_PER_GIB: f64 = 1024.0;
    pub const KIB_PER_GIB: f64 = KIB_PER_MIB * MIB_PER_GIB;
}

/// Process filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_pattern: Option<String>,
}

impl FilterConfig {
    /// Format patterns as human-readable lines for display
    pub fn display_patterns(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(ref exclude) = self.exclude_pattern {
            lines.push(format!("Exclude pattern: '{}'", exclude));
        }
        if let Some(ref include) = self.include_pattern {
            lines.push(format!("Include pattern: '{}'", include));
        }
        lines
    }

    /// Format patterns for CSV comment
    pub fn to_csv_comment(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref exclude) = self.exclude_pattern {
            parts.push(format!("exclude='{}'", exclude));
        }
        if let Some(ref include) = self.include_pattern {
            parts.push(format!("include='{}'", include));
        }
        parts.join(" ")
    }
}

/// One snapshot of a single process at a point in time
#[derive(Debug, Clone)]
pub struct ProcessSample {
    pub pid: i32,
    pub ppid: i32,
    pub rss_kib: u64,
    pub command: String,
}

/// Per-process statistics tracked across the job lifetime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStats {
    pub pid: i32,
    pub ppid: i32,
    pub command: String,
    pub max_rss_kib: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub peak_time: DateTime<Utc>,
}

/// Timeline data point for time-series export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub timestamp: DateTime<Utc>,
    pub elapsed_seconds: f64,
    pub total_rss_kib: u64,
    pub process_count: usize,
}

/// Complete job memory profile
#[derive(Debug, Serialize, Deserialize)]
pub struct JobProfile {
    pub command: Vec<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_seconds: f64,
    pub interval_ms: u64,
    pub max_total_rss_kib: u64,
    pub samples: usize,
    pub processes: Vec<ProcessStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline: Option<Vec<TimelinePoint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<FilterConfig>,
    /// Number of processes that were filtered out
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filtered_process_count: Option<usize>,
    /// Total RSS of filtered processes (KiB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filtered_total_rss_kib: Option<u64>,
}

/// Snapshot of all processes in the job at a point in time
#[derive(Debug)]
pub struct JobSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_rss_kib: u64,
    pub processes: Vec<ProcessSample>,
}

/// Accumulated job state during sampling
#[derive(Debug)]
pub struct JobState {
    pub start_time: DateTime<Utc>,
    pub max_total_rss_kib: u64,
    pub samples: usize,
    pub process_stats: HashMap<i32, ProcessStats>,
    pub timeline: Option<Vec<TimelinePoint>>,
}

impl JobState {
    pub fn new(track_timeline: bool) -> Self {
        Self {
            start_time: Utc::now(),
            max_total_rss_kib: 0,
            samples: 0,
            process_stats: HashMap::new(),
            timeline: if track_timeline { Some(Vec::new()) } else { None },
        }
    }

    pub fn update(&mut self, snapshot: JobSnapshot) {
        self.samples += 1;
        self.max_total_rss_kib = self.max_total_rss_kib.max(snapshot.total_rss_kib);

        // Track timeline if requested
        if let Some(timeline) = &mut self.timeline {
            let elapsed_seconds = (snapshot.timestamp - self.start_time).num_milliseconds() as f64 / 1000.0;
            timeline.push(TimelinePoint {
                timestamp: snapshot.timestamp,
                elapsed_seconds,
                total_rss_kib: snapshot.total_rss_kib,
                process_count: snapshot.processes.len(),
            });
        }

        for proc in snapshot.processes {
            self.process_stats
                .entry(proc.pid)
                .and_modify(|stats| {
                    // Update peak time only if this is a new peak
                    if proc.rss_kib > stats.max_rss_kib {
                        stats.max_rss_kib = proc.rss_kib;
                        stats.peak_time = snapshot.timestamp;
                    }
                    stats.last_seen = snapshot.timestamp;
                })
                .or_insert_with(|| ProcessStats {
                    pid: proc.pid,
                    ppid: proc.ppid,
                    command: proc.command,
                    max_rss_kib: proc.rss_kib,
                    first_seen: snapshot.timestamp,
                    last_seen: snapshot.timestamp,
                    peak_time: snapshot.timestamp,
                });
        }
    }

    pub fn into_profile(
        self,
        command: Vec<String>,
        interval_ms: u64,
        exit_code: Option<i32>,
        exclude_pattern: Option<String>,
        include_pattern: Option<String>,
    ) -> anyhow::Result<JobProfile> {
        let end_time = Utc::now();
        let duration_seconds = (end_time - self.start_time).num_milliseconds() as f64 / 1000.0;

        let mut all_processes: Vec<ProcessStats> = self.process_stats.into_values().collect();
        all_processes.sort_by_key(|p| std::cmp::Reverse(p.max_rss_kib));

        // Apply filtering if patterns are provided
        let has_filter = exclude_pattern.is_some() || include_pattern.is_some();

        let (processes, filter, filtered_process_count, filtered_total_rss_kib) = if has_filter {
            let (filtered_processes, filter_info) = apply_filter(
                all_processes,
                exclude_pattern.as_deref(),
                include_pattern.as_deref(),
            )?;

            let (filtered_count, filtered_rss) = filter_info.expect("filter_info must be Some when patterns provided");

            (
                filtered_processes,
                Some(FilterConfig {
                    exclude_pattern,
                    include_pattern,
                }),
                Some(filtered_count),
                Some(filtered_rss),
            )
        } else {
            (all_processes, None, None, None)
        };

        Ok(JobProfile {
            command,
            start_time: self.start_time,
            end_time,
            duration_seconds,
            interval_ms,
            max_total_rss_kib: self.max_total_rss_kib,
            samples: self.samples,
            processes,
            timeline: self.timeline,
            exit_code,
            filter,
            filtered_process_count,
            filtered_total_rss_kib,
        })
    }
}

/// Apply include/exclude filters to process list.
///
/// Takes ownership of the process list to avoid cloning. Processes that pass the filter
/// are moved into the result vector, while filtered-out processes are only counted.
///
/// # Arguments
/// * `processes` - Owned vector of processes to filter
/// * `exclude_pattern` - Regex pattern to exclude (optional)
/// * `include_pattern` - Regex pattern to include (optional)
///
/// # Returns
/// Tuple of (filtered_processes, Option<(filtered_count, filtered_rss_kib)>)
///
/// # Errors
/// Returns error if regex patterns are invalid
fn apply_filter(
    processes: Vec<ProcessStats>,
    exclude_pattern: Option<&str>,
    include_pattern: Option<&str>,
) -> anyhow::Result<(Vec<ProcessStats>, Option<(usize, u64)>)> {
    use anyhow::Context;

    let exclude_regex = match exclude_pattern {
        Some(p) => Some(Regex::new(p).context(format!("Invalid exclude pattern '{}': must be valid regex", p))?),
        None => None,
    };
    let include_regex = match include_pattern {
        Some(p) => Some(Regex::new(p).context(format!("Invalid include pattern '{}': must be valid regex", p))?),
        None => None,
    };

    let mut filtered = Vec::new();
    let mut filtered_count = 0;
    let mut filtered_rss = 0u64;

    for proc in processes {
        let mut should_include = true;

        // Apply include filter first
        if let Some(ref include) = include_regex {
            should_include = include.is_match(&proc.command);
        }

        // Then apply exclude filter
        if should_include {
            if let Some(ref exclude) = exclude_regex {
                if exclude.is_match(&proc.command) {
                    should_include = false;
                }
            }
        }

        if should_include {
            filtered.push(proc);
        } else {
            // Only track statistics for filtered-out processes
            filtered_count += 1;
            filtered_rss += proc.max_rss_kib;
        }
    }

    // Always track filter info if patterns were provided, even if nothing was filtered
    let filter_info = if exclude_pattern.is_some() || include_pattern.is_some() {
        Some((filtered_count, filtered_rss))
    } else {
        None
    };

    Ok((filtered, filter_info))
}
