use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    pub fn into_profile(self, command: Vec<String>, interval_ms: u64, exit_code: Option<i32>) -> JobProfile {
        let end_time = Utc::now();
        let duration_seconds = (end_time - self.start_time).num_milliseconds() as f64 / 1000.0;

        let mut processes: Vec<ProcessStats> = self.process_stats.into_values().collect();
        processes.sort_by_key(|p| std::cmp::Reverse(p.max_rss_kib));

        JobProfile {
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
        }
    }
}
