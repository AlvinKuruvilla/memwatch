use crate::types::ProcessSample;
use anyhow::{Context, Result};
use std::process::Command;

use super::ProcessInspector;

/// macOS process inspector using ps command
pub struct MacProcessInspector;

impl MacProcessInspector {
    pub fn new() -> Self {
        Self
    }
}

impl ProcessInspector for MacProcessInspector {
    fn snapshot_all(&self) -> Result<Vec<ProcessSample>> {
        let output = Command::new("ps")
            .args(["-axo", "pid,ppid,rss,command"])
            .output()
            .context("Failed to execute ps command")?;

        if !output.status.success() {
            anyhow::bail!("ps command failed with status: {}", output.status);
        }

        let stdout = String::from_utf8(output.stdout)
            .context("ps output was not valid UTF-8")?;

        parse_ps_output(&stdout)
    }
}

fn parse_ps_output(output: &str) -> Result<Vec<ProcessSample>> {
    let mut processes = Vec::new();

    for (line_num, line) in output.lines().enumerate() {
        // Skip header line
        if line_num == 0 {
            continue;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse: PID PPID RSS COMMAND
        // First, split by whitespace to get all parts
        let mut parts = line.split_whitespace();

        let pid = match parts.next() {
            Some(p) => p.parse::<i32>().context(format!("Failed to parse PID from: {}", p))?,
            None => continue,
        };

        let ppid = match parts.next() {
            Some(p) => p.parse::<i32>().context(format!("Failed to parse PPID from: {}", p))?,
            None => continue,
        };

        let rss_kib = match parts.next() {
            Some(r) => r.parse::<u64>().context(format!("Failed to parse RSS from: {}", r))?,
            None => continue,
        };

        // Rest of the line is the command
        let command = parts.collect::<Vec<_>>().join(" ");
        if command.is_empty() {
            continue;
        }

        processes.push(ProcessSample {
            pid,
            ppid,
            rss_kib,
            command,
        });
    }

    Ok(processes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ps_output() {
        let output = r#"  PID  PPID  RSS COMMAND
    1     0   1234 /sbin/launchd
  123     1   5678 /usr/bin/safari
  456   123  91011 /Applications/Safari.app/Contents/MacOS/Safari --flag
"#;

        let processes = parse_ps_output(output).unwrap();
        assert_eq!(processes.len(), 3);

        assert_eq!(processes[0].pid, 1);
        assert_eq!(processes[0].ppid, 0);
        assert_eq!(processes[0].rss_kib, 1234);
        assert_eq!(processes[0].command, "/sbin/launchd");

        assert_eq!(processes[1].pid, 123);
        assert_eq!(processes[1].ppid, 1);
        assert_eq!(processes[1].rss_kib, 5678);

        assert_eq!(processes[2].pid, 456);
        assert_eq!(processes[2].ppid, 123);
        assert_eq!(processes[2].rss_kib, 91011);
        assert!(processes[2].command.contains("--flag"));
    }
}
