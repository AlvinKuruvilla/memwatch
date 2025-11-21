use crate::types::ProcessSample;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::ProcessInspector;

/// Linux process inspector using /proc filesystem
pub struct LinuxProcessInspector;

impl LinuxProcessInspector {
    pub fn new() -> Self {
        Self
    }

    fn read_proc_stat(&self, pid: i32) -> Result<(i32, String)> {
        let stat_path = format!("/proc/{}/stat", pid);
        let stat_content = fs::read_to_string(&stat_path)
            .context(format!("Failed to read {}", stat_path))?;

        // Parse /proc/[pid]/stat format:
        // pid (comm) state ppid ...
        // We need to handle command names with spaces and parentheses
        let start_paren = stat_content.find('(')
            .context("Invalid stat format: missing '('")?;
        let end_paren = stat_content.rfind(')')
            .context("Invalid stat format: missing ')'")?;

        let after_comm = &stat_content[end_paren + 1..].trim();
        let fields: Vec<&str> = after_comm.split_whitespace().collect();

        if fields.len() < 2 {
            anyhow::bail!("Invalid stat format: not enough fields");
        }

        // Field 0 is state, field 1 is ppid
        let ppid = fields[1].parse::<i32>()
            .context("Failed to parse ppid")?;

        let comm = stat_content[start_paren + 1..end_paren].to_string();

        Ok((ppid, comm))
    }

    fn read_proc_status_rss(&self, pid: i32) -> Result<u64> {
        let status_path = format!("/proc/{}/status", pid);
        let status_content = fs::read_to_string(&status_path)
            .context(format!("Failed to read {}", status_path))?;

        for line in status_content.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let rss_kib = parts[1].parse::<u64>()
                        .context("Failed to parse VmRSS value")?;
                    return Ok(rss_kib);
                }
            }
        }

        // If VmRSS is not found, the process might not have RSS (kernel threads)
        Ok(0)
    }

    fn read_cmdline(&self, pid: i32) -> Result<String> {
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        let cmdline_content = fs::read(&cmdline_path)
            .context(format!("Failed to read {}", cmdline_path))?;

        if cmdline_content.is_empty() {
            // Kernel thread or empty cmdline - use comm from stat
            return Ok(String::new());
        }

        // cmdline is null-separated
        let cmdline = cmdline_content
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).to_string())
            .collect::<Vec<_>>()
            .join(" ");

        Ok(cmdline)
    }
}

impl ProcessInspector for LinuxProcessInspector {
    fn snapshot_all(&self) -> Result<Vec<ProcessSample>> {
        let proc_path = Path::new("/proc");
        let mut processes = Vec::new();

        let entries = fs::read_dir(proc_path)
            .context("Failed to read /proc directory")?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let filename = entry.file_name();
            let pid_str = match filename.to_str() {
                Some(s) => s,
                None => continue,
            };

            let pid = match pid_str.parse::<i32>() {
                Ok(p) => p,
                Err(_) => continue, // Not a PID directory
            };

            // Try to read process info, skip if we can't (process may have exited)
            let (ppid, comm) = match self.read_proc_stat(pid) {
                Ok(info) => info,
                Err(_) => continue,
            };

            let rss_kib = match self.read_proc_status_rss(pid) {
                Ok(rss) => rss,
                Err(_) => continue,
            };

            let cmdline = match self.read_cmdline(pid) {
                Ok(cmd) if !cmd.is_empty() => cmd,
                _ => comm.clone(),
            };

            processes.push(ProcessSample {
                pid,
                ppid,
                rss_kib,
                command: cmdline,
            });
        }

        Ok(processes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_read_self() {
        let inspector = LinuxProcessInspector::new();
        let pid = std::process::id() as i32;

        let (ppid, comm) = inspector.read_proc_stat(pid).unwrap();
        assert!(ppid > 0);
        assert!(!comm.is_empty());

        let rss = inspector.read_proc_status_rss(pid).unwrap();
        assert!(rss > 0);

        let cmdline = inspector.read_cmdline(pid).unwrap();
        assert!(!cmdline.is_empty());
    }
}
