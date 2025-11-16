# Pre-Boot Authentication Environment Design

## Overview

This document defines the design for Secure Cryptor's pre-boot authentication environment - the minimal interface and system that runs before the operating system loads to authenticate users and unlock encrypted volumes.

## Table of Contents

1. [Requirements](#requirements)
2. [Architecture](#architecture)
3. [User Interface Design](#user-interface-design)
4. [Authentication Flow](#authentication-flow)
5. [Platform Implementation](#platform-implementation)
6. [Error Handling](#error-handling)
7. [Security Considerations](#security-considerations)

---

## Requirements

### Functional Requirements

- **FR1**: Prompt user for password/PIN before OS loads
- **FR2**: Support multiple authentication methods (password, TPM, recovery key)
- **FR3**: Validate credentials against volume header
- **FR4**: Unlock volume if authentication succeeds
- **FR5**: Provide clear error messages and retry mechanism
- **FR6**: Support recovery scenarios (forgotten password, TPM failure)
- **FR7**: Measure boot state for TPM attestation
- **FR8**: Handle keyboard input securely (mask password)

### Non-Functional Requirements

- **NFR1**: Minimal memory footprint (< 10MB)
- **NFR2**: Fast boot time (< 2 seconds to prompt)
- **NFR3**: Resilient to power failure during authentication
- **NFR4**: Compatible with UEFI Secure Boot
- **NFR5**: Cross-platform (Windows, Linux)
- **NFR6**: Accessible (large text, high contrast)
- **NFR7**: Zero plaintext key exposure in memory after use

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────┐
│              UEFI Firmware                           │
│  - Secure Boot verification                         │
│  - TPM initialization                               │
└─────────────────────┬───────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────┐
│        Secure Cryptor Pre-Boot Module               │
│  ┌───────────────────────────────────────────────┐ │
│  │  Authentication UI                            │ │
│  │  - Password prompt                            │ │
│  │  - Recovery key prompt                        │ │
│  │  - Error display                              │ │
│  └───────────────┬───────────────────────────────┘ │
│                  │                                   │
│  ┌───────────────▼───────────────────────────────┐ │
│  │  Authentication Engine                        │ │
│  │  - Password verification                      │ │
│  │  - TPM unsealing                              │ │
│  │  - Key derivation (Argon2id)                  │ │
│  └───────────────┬───────────────────────────────┘ │
│                  │                                   │
│  ┌───────────────▼───────────────────────────────┐ │
│  │  Volume Unlocking                             │ │
│  │  - Read volume header                         │ │
│  │  - Decrypt master key                         │ │
│  │  - Configure block device                     │ │
│  └───────────────┬───────────────────────────────┘ │
└──────────────────┼───────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│            Bootloader (GRUB/WBM)                    │
│  - Load kernel from decrypted volume                │
└─────────────────────────────────────────────────────┘
```

### Module Responsibilities

#### Authentication UI Module

**Responsibilities:**
- Render text/graphics on screen
- Accept keyboard input
- Display status messages and errors
- Provide accessibility features

**Constraints:**
- No graphics dependencies (pure UEFI protocols)
- Fixed-size frame buffer (no dynamic allocation)
- Must work on all UEFI systems (no vendor-specific features)

#### Authentication Engine Module

**Responsibilities:**
- Coordinate authentication methods (password, TPM, recovery)
- Derive encryption keys from passwords (Argon2id)
- Communicate with TPM for unsealing
- Validate credentials against volume header
- Implement retry logic and lockout

**Constraints:**
- Must zeroize all sensitive data after use
- Limited to UEFI-available crypto (or embedded Rust crypto)
- No network access
- No persistent storage (except encrypted volume header)

#### Volume Unlocking Module

**Responsibilities:**
- Read volume header from disk
- Decrypt master key using derived key
- Set up block device access for bootloader
- Hand off to next boot stage

**Constraints:**
- Must not leave plaintext keys in memory
- Must work with GPT partitions
- Support multiple volume formats (LUKS compatibility)

---

## User Interface Design

### Screen Layouts

#### Main Authentication Screen

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│                     🔒 SECURE CRYPTOR                        │
│                                                              │
│              Full Disk Encryption Pre-Boot                   │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                                                        │ │
│  │  Volume: C:\ (Windows System Volume)                  │ │
│  │  UUID: a1b2c3d4-e5f6-7890-abcd-ef1234567890           │ │
│  │                                                        │ │
│  │  Enter password to unlock:                            │ │
│  │                                                        │ │
│  │  Password: ******************                         │ │
│  │                                                        │ │
│  │  [Press ENTER to unlock]                              │ │
│  │  [Press F8 for recovery options]                      │ │
│  │                                                        │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│                                                              │
│  Attempt 1 of 5 │ TPM: Available │ Secure Boot: Enabled     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### Recovery Options Screen

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│                  🔑 RECOVERY OPTIONS                         │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                                                        │ │
│  │  1. Enter Recovery Key                                │ │
│  │     Use 64-character recovery key to unlock           │ │
│  │                                                        │ │
│  │  2. Password Hint                                     │ │
│  │     Display your password hint (if set)               │ │
│  │                                                        │ │
│  │  3. Return to Password Entry                          │ │
│  │     Try password again                                │ │
│  │                                                        │ │
│  │  4. Boot from Recovery Media                          │ │
│  │     Insert USB recovery drive                         │ │
│  │                                                        │ │
│  │  Select option (1-4):                                 │ │
│  │                                                        │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  [Press ESC to cancel]                                       │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### Error Screen

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│                     ❌ AUTHENTICATION FAILED                 │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                                                        │ │
│  │  Incorrect password                                   │ │
│  │                                                        │ │
│  │  Remaining attempts: 3 of 5                           │ │
│  │                                                        │ │
│  │  If you've forgotten your password:                   │ │
│  │  - Press F8 for recovery options                      │ │
│  │  - Use your recovery key to reset password            │ │
│  │                                                        │ │
│  │  WARNING: After 5 failed attempts, the system will    │ │
│  │  temporarily lock for 5 minutes to prevent attacks.   │ │
│  │                                                        │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  [Press any key to try again]                                │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### TPM Auto-Unlock Screen (Success)

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│                   ✓ TPM AUTO-UNLOCK SUCCESSFUL               │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                                                        │ │
│  │  TPM has verified system integrity:                   │ │
│  │                                                        │ │
│  │  ✓ Firmware unchanged                                 │ │
│  │  ✓ Secure Boot enabled                                │ │
│  │  ✓ Bootloader verified                                │ │
│  │                                                        │ │
│  │  Volume unlocked successfully                         │ │
│  │                                                        │ │
│  │  Loading Windows...                                   │ │
│  │                                                        │ │
│  │  ████████████░░░░░░░░░░░░ 65%                         │ │
│  │                                                        │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Color Scheme

**High Contrast Mode (Default):**
- Background: Black (#000000)
- Text: White (#FFFFFF)
- Borders: Bright Blue (#00FFFF)
- Errors: Bright Red (#FF0000)
- Success: Bright Green (#00FF00)
- Warnings: Bright Yellow (#FFFF00)

**Rationale:** Maximum visibility in all lighting conditions, accessibility for vision impairment

### Keyboard Shortcuts

| Key | Function |
|-----|----------|
| Enter | Submit password / Confirm selection |
| Backspace | Delete character |
| Esc | Cancel / Go back |
| F8 | Recovery options |
| F9 | System information |
| F12 | Boot menu (if unlocked) |
| Tab | Switch input field (if multiple) |

---

## Authentication Flow

### Standard Boot with Password

```
┌─────────────┐
│ Power On    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ UEFI Init   │
│ - TPM Init  │
│ - Sec Boot  │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ Load Pre-Boot   │
│ Module (.efi)   │
└──────┬──────────┘
       │
       ▼
┌─────────────────────┐
│ Read Volume Header  │
│ - Detect encryption │
│ - Check TPM seal    │
└──────┬──────────────┘
       │
       ▼
┌──────────────────┐      No TPM
│ TPM Available?   ├──────────────┐
└──────┬───────────┘              │
       │ Yes                       │
       ▼                           ▼
┌──────────────────┐    ┌──────────────────┐
│ Try TPM Unseal   │    │ Show Password    │
└──────┬───────────┘    │ Prompt           │
       │                └──────┬───────────┘
       │ Success               │
       │ ┌─────────────────────┘
       ▼ ▼
┌──────────────────┐
│ Decrypt Master   │
│ Key              │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Unlock Volume    │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Load Bootloader  │
│ (GRUB / WBM)     │
└──────────────────┘
```

### Boot with TPM Failure

```
┌──────────────────┐
│ TPM Unseal Fails │
│ (PCR mismatch)   │
└──────┬───────────┘
       │
       ▼
┌──────────────────────────┐
│ Display Warning:         │
│ "System changed, using   │
│ password authentication" │
└──────┬───────────────────┘
       │
       ▼
┌──────────────────────┐
│ Show Password Prompt │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Derive Key (Argon2)  │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Try Key Slots        │
│ (Up to 8 slots)      │
└──────┬───────────────┘
       │
       ├─ Match ──────────┐
       │                  ▼
       │           ┌──────────────┐
       │           │ Unlock Vol.  │
       │           └──────────────┘
       │
       └─ No Match ───────┐
                          ▼
                   ┌──────────────┐
                   │ Increment    │
                   │ Failed Count │
                   └──────┬───────┘
                          │
                          ▼
                   ┌──────────────┐    Yes
                   │ >= 5 Failed? ├────────┐
                   └──────┬───────┘        │
                          │ No             ▼
                          │         ┌──────────────┐
                          │         │ Lockout      │
                          │         │ (5 minutes)  │
                          │         └──────────────┘
                          │
                          ▼
                   ┌──────────────┐
                   │ Retry Prompt │
                   └──────────────┘
```

### Recovery Key Flow

```
┌──────────────────┐
│ User presses F8  │
└──────┬───────────┘
       │
       ▼
┌──────────────────────┐
│ Recovery Menu        │
│ 1. Recovery Key      │
│ 2. Password Hint     │
│ 3. Return            │
│ 4. Boot USB          │
└──────┬───────────────┘
       │ Select 1
       ▼
┌──────────────────────────────┐
│ Prompt: Enter 64-char        │
│ recovery key                 │
└──────┬───────────────────────┘
       │
       ▼
┌──────────────────────────────┐
│ Validate Format:             │
│ - Exactly 64 hex chars       │
│ - No invalid characters      │
└──────┬───────────────────────┘
       │ Valid
       ▼
┌──────────────────────────────┐
│ Try Unlock with Recovery Key │
└──────┬───────────────────────┘
       │
       ├─ Success ─────────┐
       │                   ▼
       │            ┌──────────────┐
       │            │ Unlock Vol.  │
       │            │ Boot OS      │
       │            └──────────────┘
       │
       └─ Failure ─────────┐
                           ▼
                    ┌──────────────┐
                    │ Error:       │
                    │ Invalid key  │
                    └──────────────┘
```

---

## Platform Implementation

### Linux (GRUB Integration)

#### Approach: GRUB Module

**File Structure:**
```
/boot/efi/EFI/securecryptor/
    ├── grubx64.efi (signed)
    ├── modules/
    │   └── sc_preboot.mod (Secure Cryptor module)
    └── fonts/
        └── unicode.pf2
```

**GRUB Configuration:**
```bash
# /etc/grub.d/00_header
insmod sc_preboot
insmod cryptodisk
insmod luks

set timeout=0
set timeout_style=hidden

# Secure Cryptor pre-boot auth
sc_authenticate

# If successful, continue to encrypted boot
cryptomount -u <UUID>
```

**Module Implementation:**
- Written in C (GRUB modules are C-based)
- Uses GRUB's crypto APIs (gcry_* functions)
- Accesses GRUB's TPM support (tpm2_ commands)
- Integrates with GRUB's UI framework

**Challenges:**
- GRUB modules must be separate from Rust codebase
- Limited debugging capabilities
- Must be signed for Secure Boot

### Windows (UEFI Application)

#### Approach: Custom UEFI Pre-Boot Application

**File Structure:**
```
\EFI\SecureCryptor\
    ├── SecureCryptorAuth.efi (signed)
    ├── config.dat (encrypted configuration)
    └── recovery.dat (recovery info)
```

**Boot Sequence:**
```
UEFI Firmware
    ↓
SecureCryptorAuth.efi (our app)
    ↓ (if unlocked)
\EFI\Microsoft\Boot\bootmgfw.efi (Windows Boot Manager)
```

**Implementation:**
- Pure UEFI application in Rust (using `uefi-rs` crate)
- No OS dependencies
- Direct access to UEFI protocols:
  - `EFI_SIMPLE_TEXT_INPUT_PROTOCOL` (keyboard)
  - `EFI_GRAPHICS_OUTPUT_PROTOCOL` (display)
  - `EFI_BLOCK_IO_PROTOCOL` (disk access)
  - `EFI_TPM2_PROTOCOL` (TPM 2.0)

**Example Code Structure:**
```rust
#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::proto::console::text::{Input, Output};
use uefi::proto::media::block::BlockIO;

#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    // Initialize
    uefi_services::init(&mut st).unwrap();

    // Get protocols
    let stdout = st.stdout();
    let stdin = st.stdin();

    // Display UI
    stdout.clear().unwrap();
    display_banner(stdout);

    // Read volume header from disk
    let volume_header = read_volume_header(st.boot_services())?;

    // Try TPM unlock
    if let Some(master_key) = try_tpm_unlock(&volume_header) {
        unlock_volume(master_key);
        return Status::SUCCESS;
    }

    // Password authentication
    loop {
        let password = prompt_password(stdin, stdout)?;

        match authenticate(&volume_header, &password) {
            Ok(master_key) => {
                unlock_volume(master_key);
                return Status::SUCCESS;
            }
            Err(e) => {
                display_error(stdout, e);
                retry_count += 1;
                if retry_count >= 5 {
                    lockout(300); // 5 minutes
                    retry_count = 0;
                }
            }
        }
    }
}
```

**Dependencies:**
```toml
[dependencies]
uefi = "0.31"
uefi-services = "0.31"
argon2 = { version = "0.5", default-features = false }
aes-gcm = { version = "0.10", default-features = false }
zeroize = { version = "1.8", default-features = false }
```

### Cross-Platform Rust Library

**Shared Authentication Logic:**
```rust
// src/preboot/auth.rs
pub struct PrebootAuthenticator {
    volume_header: VolumeHeader,
    retry_count: usize,
    max_retries: usize,
}

impl PrebootAuthenticator {
    pub fn new(volume_header: VolumeHeader) -> Self {
        Self {
            volume_header,
            retry_count: 0,
            max_retries: 5,
        }
    }

    pub fn authenticate(&mut self, password: &str) -> Result<MasterKey, AuthError> {
        // Derive key from password
        let kdf = Argon2Kdf::new(self.volume_header.crypto_config());
        let derived_key = kdf.derive_key(
            password.as_bytes(),
            self.volume_header.salt()
        )?;

        // Try each key slot
        match self.volume_header.key_slots().unlock(password) {
            Ok(master_key) => {
                self.retry_count = 0;
                Ok(master_key)
            }
            Err(_) => {
                self.retry_count += 1;
                Err(AuthError::InvalidPassword {
                    remaining: self.max_retries - self.retry_count
                })
            }
        }
    }

    pub fn try_tpm_unlock(&self) -> Option<MasterKey> {
        // Attempt TPM unsealing
        // Implementation depends on platform
        None
    }

    pub fn should_lockout(&self) -> bool {
        self.retry_count >= self.max_retries
    }
}
```

---

## Error Handling

### Error Categories

#### Authentication Errors

| Error | Display Message | Action |
|-------|----------------|--------|
| `InvalidPassword` | "Incorrect password. X attempts remaining." | Allow retry |
| `TooManyAttempts` | "Too many failed attempts. Locked for 5 minutes." | Enforce lockout |
| `RecoveryKeyInvalid` | "Invalid recovery key format." | Return to recovery menu |
| `KeySlotCorrupted` | "Volume header corrupted. Use recovery media." | Halt |

#### TPM Errors

| Error | Display Message | Action |
|-------|----------------|--------|
| `TpmNotFound` | "TPM not available. Using password authentication." | Fall back to password |
| `TpmPcrMismatch` | "System configuration changed. Using password authentication." | Fall back to password |
| `TpmCommunicationError` | "TPM communication failed. Using password authentication." | Fall back to password |
| `TpmLocked` | "TPM is locked. Restart required." | Halt |

#### Volume Errors

| Error | Display Message | Action |
|-------|----------------|--------|
| `VolumeNotFound` | "Encrypted volume not found." | Halt |
| `HeaderCorrupted` | "Volume header corrupted. Use header backup." | Offer recovery |
| `UnsupportedVersion` | "Volume format not supported. Update required." | Halt |
| `DiskError` | "Disk read error. Check hardware." | Halt |

### Logging and Diagnostics

**Pre-Boot Environment Logging:**

Since we're in pre-boot, traditional logging isn't available. Instead:

1. **UEFI Variable Storage** (if available):
   ```
   EFI Variable: SecureCryptor-LastBoot
   - Timestamp
   - Authentication method used (TPM/Password)
   - Number of retry attempts
   - Error code (if failed)
   ```

2. **Emergency Diagnostic Mode** (F9):
   Display system information:
   ```
   SECURE CRYPTOR DIAGNOSTIC INFO

   Version: 1.0.0
   Build: 2025-11-16

   System:
   - UEFI Version: 2.8
   - Secure Boot: Enabled
   - TPM Version: 2.0
   - TPM Status: Available

   Volume:
   - UUID: a1b2c3d4...
   - Format Version: 1.0
   - Active Key Slots: 2/8
   - Last Modified: 2025-11-15 18:30 UTC

   Boot:
   - Last Successful: 2025-11-16 06:00 UTC
   - Failed Attempts: 0
   - TPM PCR 7: 0xABCD... (Secure Boot enabled)
   ```

---

## Security Considerations

### Threat Mitigation

#### Brute Force Protection

**Lockout Policy:**
- 5 failed password attempts → 5 minute lockout
- Lockout enforced by busy-wait loop (no sleep available in UEFI)
- Counter persisted in UEFI variable (survives reboot)
- After lockout, counter resets

**Rate Limiting:**
- Minimum 1 second delay between password attempts
- Prevents automated rapid guessing

#### Memory Protection

**Sensitive Data Handling:**
```rust
use zeroize::{Zeroize, Zeroizing};

fn authenticate(password: &str) -> Result<MasterKey> {
    // Use Zeroizing for automatic cleanup
    let derived_key = Zeroizing::new(derive_key(password)?);

    // Use key, automatically zeroized on drop
    let master_key = decrypt_master_key(&derived_key)?;

    Ok(master_key)
} // derived_key zeroized here
```

**Stack Wiping:**
```rust
fn cleanup_stack() {
    // Overwrite stack with zeros
    let mut stack_wipe = [0u8; 4096];
    unsafe {
        core::ptr::write_volatile(&mut stack_wipe, [0u8; 4096]);
    }
}
```

#### Evil Maid Attacks

**Mitigation via Secure Boot + TPM:**
- Bootloader must be signed (Secure Boot verification)
- TPM PCR binding detects tampered bootloader
- If PCR mismatch detected, TPM won't unseal key
- User notified of system changes

**Visual Verification:**
- Display TPM PCR values on F9 diagnostic screen
- User can verify against known good values

#### Cold Boot Attacks

**Partial Mitigation:**
- Minimize time key is in memory
- Zeroize immediately after use
- Consider:
  - Memory encryption (if UEFI supports)
  - Overwriting freed memory

**Note:** True mitigation requires hardware support (Intel TDX, AMD SEV)

### Compliance and Best Practices

#### Password Requirements

**Enforced at enrollment** (not pre-boot):
- Minimum 12 characters
- Mix of upper, lower, digits, symbols
- Not in common password lists
- Entropy > 50 bits

**Pre-boot considerations:**
- Accept any password (was set securely)
- Mask input with `*` characters
- Clear buffer immediately after use

#### Key Derivation Parameters

**Argon2id Configuration:**
```rust
Argon2Params {
    m_cost: 65536,      // 64 MB memory
    t_cost: 3,          // 3 iterations
    p_cost: 4,          // 4 parallel lanes
    output_len: 32,     // 256-bit key
}
```

**Justification:**
- Memory-hard: Resistant to GPU attacks
- Balanced: Fast enough for pre-boot (< 2 seconds)
- Secure: OWASP recommended

---

## Implementation Roadmap

### Phase 1: Linux Prototype (GRUB Module)

**Deliverables:**
- GRUB module for password authentication
- Integration with existing LUKS key slots
- Basic error handling
- Documentation

**Estimated Effort:** 2-3 weeks

### Phase 2: Windows UEFI Application

**Deliverables:**
- Standalone UEFI .efi application
- Password authentication
- Integration with Secure Cryptor volume format
- Signing for Secure Boot

**Estimated Effort:** 3-4 weeks

### Phase 3: TPM Integration

**Deliverables:**
- TPM 2.0 key sealing/unsealing
- PCR binding (0, 7)
- Fallback to password on TPM failure
- Cross-platform support

**Estimated Effort:** 2-3 weeks

### Phase 4: Hardening and Testing

**Deliverables:**
- Security audit
- Penetration testing
- Performance optimization
- User acceptance testing

**Estimated Effort:** 2 weeks

---

## Conclusion

The pre-boot authentication environment is a critical security component that must balance:

- **Security**: Strong authentication, anti-brute-force, memory protection
- **Usability**: Clear UI, fast boot, recovery options
- **Compatibility**: UEFI Secure Boot, TPM 2.0, multiple platforms

The design prioritizes:
1. **TPM auto-unlock** when system is trusted
2. **Password fallback** when TPM unavailable or PCRs changed
3. **Recovery mechanisms** for forgotten passwords
4. **Secure implementation** with memory zeroization and rate limiting

This foundation enables Secure Cryptor to provide true full-disk encryption with pre-boot authentication comparable to commercial solutions like BitLocker and LUKS.

---

*Document created: 2025-11-16*
*Last updated: 2025-11-16*
*Status: Design Complete*
