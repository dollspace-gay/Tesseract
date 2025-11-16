# Userspace Filesystem Library Research

Research for implementing encrypted container/volume mounting across platforms.

## Executive Summary

**Recommendation:** Use **fuser** for Linux, **WinFsp** for Windows, and **macFUSE** for macOS.

All platforms have mature Rust bindings available, enabling cross-platform encrypted filesystem implementation.

---

## Linux: FUSE

### Primary Option: `fuser` crate

**Repository:** https://github.com/cberner/fuser
**Crate:** https://crates.io/crates/fuser
**Status:** Actively maintained (as of 2025)

#### Key Features:
- Compatible with both FUSE 2 and FUSE 3
- Implements most features of libfuse up to version 3.10.3
- Pure Rust rewrite leveraging Rust's architecture
- Developed and tested on Linux and FreeBSD
- Uses stable Rust

#### Usage:
```rust
use fuser::Filesystem;

// Implement the Filesystem trait
impl Filesystem for MyEncryptedFS {
    // Implement required methods
}
```

#### Requirements:
- FUSE kernel driver and libraries must be installed
- Some platforms require userland utils (e.g., `fusermount`)

#### Alternative Crates:
- **easy_fuser**: High-level ergonomic wrapper around fuser
- **fuse-backend-rs**: For implementing Fuse daemons based on /dev/fuse or virtiofs
- **fuse_mt**: Higher-level access similar to FUSE C API

---

## Windows: WinFsp vs Dokany

### Primary Recommendation: **WinFsp**

**Website:** https://winfsp.dev/
**GitHub:** https://github.com/winfsp/winfsp
**Rust Bindings:** `winfsp` crate (updated October 2025)

#### Why WinFsp over Dokany:
1. **Better Performance:** Generally faster than Dokany
2. **Official Support:** Microsoft-endorsed solution
3. **Wide Compatibility:** Windows 7-11 on x86, x64, ARM64
4. **Multiple APIs:** Native, FUSE2, FUSE3, and .NET APIs
5. **Better Rust Bindings:** `winfsp-rs` provides safe wrappers

#### Key Features:
- Enables user-mode filesystem development without kernel programming
- No knowledge of Windows kernel required
- Respects Rust's aliasing rules in bindings
- Network drive support (with proper configuration)

#### Rust Bindings:
```rust
use winfsp::*;

// Implement filesystem using safe Rust bindings
```

### Alternative: Dokany

**Rust Bindings:** `dokan-rust` crate
**Note:** Can interfere with WinFsp's network drive handling
**Use Case:** Legacy support or specific Dokany requirements

---

## macOS: macFUSE

### Primary Option: **macFUSE**

**Website:** https://macfuse.github.io/
**GitHub:** https://github.com/macfuse/macfuse
**Latest Version:** 5.1.1 (Released November 7, 2025)

#### Recent Major Improvement (macOS 26+):
- New **FSKit backend** enables filesystems to run entirely in **user space**
- **No more kernel extension** on macOS 26+
- **No rebooting** into recovery mode required

#### Compatibility:
- Supports macOS 12 through macOS 26
- Rust `fuser` crate can work with macFUSE with symlink workaround

#### Rust Integration Issue & Solution:
The Rust `fuse` crate looks for `osxfuse.pc` but macFUSE provides `fuse.pc`.

**Solution:**
```bash
# Create symlink for backward compatibility
ln -s /usr/local/lib/pkgconfig/fuse.pc /usr/local/lib/pkgconfig/osxfuse.pc
```

#### Alternative: FUSE-T

**Website:** https://www.fuse-t.org/
**Type:** Kext-less implementation using NFS v4 local server

**Advantages:**
- Better performance (excellent macOS NFSv4 client implementation)
- No kernel extension required (works on all macOS versions)

**Disadvantages:**
- Different architecture (NFS-based vs kernel extension)
- Less mature than macFUSE

---

## Cross-Platform Strategy

### Recommended Approach

```rust
// Conditional compilation for cross-platform support

#[cfg(target_os = "linux")]
use fuser::{Filesystem, ...};

#[cfg(target_os = "windows")]
use winfsp::{FileSystemContext, ...};

#[cfg(target_os = "macos")]
use fuser::{Filesystem, ...}; // Works with macFUSE via symlink

// Common filesystem trait
trait EncryptedFilesystem {
    fn read(&self, path: &Path, offset: u64, size: u32) -> Result<Vec<u8>>;
    fn write(&mut self, path: &Path, offset: u64, data: &[u8]) -> Result<u32>;
    // ... other operations
}

// Platform-specific implementations
#[cfg(target_os = "linux")]
impl Filesystem for LinuxEncryptedFS { /* ... */ }

#[cfg(target_os = "windows")]
impl FileSystemContext for WindowsEncryptedFS { /* ... */ }

#[cfg(target_os = "macos")]
impl Filesystem for MacOSEncryptedFS { /* ... */ }
```

---

## Dependencies to Add

### Cargo.toml:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
fuser = "0.14"  # or latest

[target.'cfg(target_os = "windows")'.dependencies]
winfsp = "0.4"  # or latest

[target.'cfg(target_os = "macos")'.dependencies]
fuser = "0.14"  # Works with macFUSE
```

---

## Implementation Considerations

### Security:
1. All three solutions run in **user space** (macOS 26+ completely, others traditionally)
2. No kernel-level vulnerabilities in our code
3. Platform filesystem permissions still apply

### Performance:
1. **WinFsp** is faster than Dokany on Windows
2. **FUSE-T** offers better performance than macFUSE on macOS
3. **fuser** on Linux has excellent performance

### Compatibility:
1. All solutions support modern OS versions
2. macFUSE's FSKit backend is future-proof for macOS
3. WinFsp supports ARM64 Windows

---

## Next Steps

1. **Prototype Phase:**
   - Create basic FUSE filesystem example on Linux using `fuser`
   - Test WinFsp integration on Windows
   - Verify macFUSE compatibility on macOS (with symlink fix)

2. **Design Phase:**
   - Define common `EncryptedFilesystem` trait
   - Design encrypted container format
   - Plan sector-based encryption strategy

3. **Implementation Phase:**
   - Implement platform-specific backends
   - Add cross-platform abstraction layer
   - Integration with existing Secure Cryptor codebase

---

## References

- **fuser:** https://github.com/cberner/fuser
- **WinFsp:** https://winfsp.dev/
- **macFUSE:** https://macfuse.github.io/
- **FUSE-T:** https://www.fuse-t.org/
- **Survey of Rust FUSE Libraries:** https://hackmd.io/@safenetwork/rJ2o9Nzkv

---

*Research completed: 2025-11-16*
*For: Secure Cryptor - Phase 4: Whole Drive Encryption*
