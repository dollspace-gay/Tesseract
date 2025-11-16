/// Encrypted volume management module
///
/// This module provides functionality for creating and managing encrypted
/// volumes that can be mounted as filesystems using FUSE (Linux/macOS)
/// or WinFsp (Windows).

pub mod header;
pub mod keyslot;

pub use header::VolumeHeader;
pub use keyslot::{KeySlots, MasterKey, MAX_KEY_SLOTS};
