//! Platform abstraction layer for pulse.
//!
//! **Taxonomy Classification**: Abstraction (Platform Abstraction Layer).

#[cfg(windows)]
pub mod win32;

#[cfg(not(windows))]
pub mod stub;

#[cfg(windows)]
pub use win32 as current;

#[cfg(not(windows))]
pub use stub as current;

pub mod config;
pub mod identity;
pub mod monitors;
pub mod shell_terminal;
pub mod sys_info;
pub mod window;
pub mod ebpf;


pub mod sysinfo_shim;
