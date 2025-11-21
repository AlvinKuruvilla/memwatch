use crate::types::ProcessSample;
use anyhow::Result;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub use linux::LinuxProcessInspector as PlatformInspector;
#[cfg(target_os = "macos")]
pub use macos::MacProcessInspector as PlatformInspector;

/// Trait for inspecting process information across different platforms
pub trait ProcessInspector {
    /// Return a snapshot of all processes on the system
    fn snapshot_all(&self) -> Result<Vec<ProcessSample>>;
}

/// Create a platform-specific process inspector
pub fn create_inspector() -> PlatformInspector {
    PlatformInspector::new()
}
