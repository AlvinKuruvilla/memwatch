use crate::inspector::ProcessInspector;
use crate::types::{JobProfile, JobSnapshot, JobState, ProcessSample};
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// Run a command and profile its memory usage
pub fn run_and_profile(
    command: Vec<String>,
    interval_ms: u64,
    track_timeline: bool,
    inspector: &impl ProcessInspector,
) -> Result<JobProfile> {
    if command.is_empty() {
        anyhow::bail!("Command cannot be empty");
    }

    // Spawn the command
    let mut child = spawn_command(&command)
        .context("Failed to start command")?;

    let root_pid = child.id() as i32;
    let mut state = JobState::new(track_timeline);

    // Take an immediate first sample to catch quick-exit processes
    // This happens as fast as possible after spawn
    if let Ok(snapshot) = sample_job_tree(inspector, root_pid) {
        state.update(snapshot);
    }

    // Sampling loop
    loop {
        // Check if the root process is still alive
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Process has exited, do one final sample and break
                if let Ok(snapshot) = sample_job_tree(inspector, root_pid) {
                    state.update(snapshot);
                }
                break;
            }
            Ok(None) => {
                // Process still running, continue sampling
            }
            Err(e) => {
                eprintln!("Warning: Failed to check process status: {}", e);
                break;
            }
        }

        // Take a snapshot
        match sample_job_tree(inspector, root_pid) {
            Ok(snapshot) => {
                state.update(snapshot);
            }
            Err(e) => {
                eprintln!("Warning: Failed to sample processes: {}", e);
            }
        }

        // Sleep for the interval
        thread::sleep(Duration::from_millis(interval_ms));
    }

    // Wait for the process to fully exit
    let _ = child.wait();

    // Convert state to profile
    Ok(state.into_profile(command, interval_ms))
}

fn spawn_command(command: &[String]) -> Result<Child> {
    if command.is_empty() {
        anyhow::bail!("Command is empty");
    }

    let program = &command[0];
    let args = &command[1..];

    Command::new(program)
        .args(args)
        .spawn()
        .context(format!("Failed to execute: {}", program))
}

/// Sample all processes and filter to those in the job tree
fn sample_job_tree(
    inspector: &impl ProcessInspector,
    root_pid: i32,
) -> Result<JobSnapshot> {
    let all_processes = inspector.snapshot_all()?;

    // Build PID -> ProcessSample map and PID -> PPID map
    let mut pid_map: HashMap<i32, ProcessSample> = HashMap::new();
    let mut ppid_map: HashMap<i32, i32> = HashMap::new();

    for proc in all_processes {
        ppid_map.insert(proc.pid, proc.ppid);
        pid_map.insert(proc.pid, proc);
    }

    // Find all PIDs that belong to the job tree
    let job_pids = find_job_pids(root_pid, &ppid_map);

    // Collect processes in the job
    let mut job_processes = Vec::new();
    let mut total_rss_kib = 0;

    for pid in job_pids {
        if let Some(proc) = pid_map.get(&pid) {
            total_rss_kib += proc.rss_kib;
            job_processes.push(proc.clone());
        }
    }

    Ok(JobSnapshot {
        timestamp: Utc::now(),
        total_rss_kib,
        processes: job_processes,
    })
}

/// Find all PIDs that are descendants of the root PID (including root itself)
fn find_job_pids(root_pid: i32, ppid_map: &HashMap<i32, i32>) -> HashSet<i32> {
    let mut job_pids = HashSet::new();
    job_pids.insert(root_pid);

    // Iteratively find children
    let mut changed = true;
    while changed {
        changed = false;
        for (&pid, &ppid) in ppid_map {
            if !job_pids.contains(&pid) && job_pids.contains(&ppid) {
                job_pids.insert(pid);
                changed = true;
            }
        }
    }

    job_pids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_job_pids_simple() {
        let mut ppid_map = HashMap::new();
        ppid_map.insert(100, 1);  // root process, parent is init
        ppid_map.insert(200, 100); // child of root
        ppid_map.insert(300, 100); // another child of root
        ppid_map.insert(400, 200); // grandchild
        ppid_map.insert(500, 50);  // unrelated process

        let job_pids = find_job_pids(100, &ppid_map);

        assert!(job_pids.contains(&100));
        assert!(job_pids.contains(&200));
        assert!(job_pids.contains(&300));
        assert!(job_pids.contains(&400));
        assert!(!job_pids.contains(&500));
    }

    #[test]
    fn test_find_job_pids_deep_tree() {
        let mut ppid_map = HashMap::new();
        ppid_map.insert(1, 0);
        ppid_map.insert(10, 1);
        ppid_map.insert(20, 10);
        ppid_map.insert(30, 20);
        ppid_map.insert(40, 30);

        let job_pids = find_job_pids(10, &ppid_map);

        assert!(job_pids.contains(&10));
        assert!(job_pids.contains(&20));
        assert!(job_pids.contains(&30));
        assert!(job_pids.contains(&40));
        assert!(!job_pids.contains(&1));
    }
}
