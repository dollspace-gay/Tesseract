# Bootloader Technologies for Pre-Boot Authentication

## Executive Summary

This document provides comprehensive research on bootloader technologies for implementing pre-boot authentication in Secure Cryptor's full disk encryption feature. The research covers GRUB, UEFI, TPM 2.0 integration, and platform-specific approaches for Windows, Linux, and macOS.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [GRUB Bootloader](#grub-bootloader)
3. [UEFI and Secure Boot](#uefi-and-secure-boot)
4. [TPM 2.0 Integration](#tpm-20-integration)
5. [Platform-Specific Approaches](#platform-specific-approaches)
6. [Implementation Recommendations](#implementation-recommendations)
7. [Security Considerations](#security-considerations)

---

## Architecture Overview

### Pre-Boot Authentication Flow

```
Power On
    ↓
UEFI Firmware
    ↓
Secure Boot Validation
    ↓
Bootloader (GRUB/Windows Boot Manager)
    ↓
Pre-Boot Authentication
    ↓
TPM Unsealing / Password Verification
    ↓
Decrypt Master Key
    ↓
Unlock Encrypted Volume
    ↓
Load Operating System
```

### Key Components

1. **UEFI Firmware**: Modern firmware interface that replaced BIOS
2. **Secure Boot**: Cryptographic verification of boot components
3. **Bootloader**: GRUB (Linux), Windows Boot Manager (Windows), custom loader
4. **TPM 2.0**: Hardware security module for key storage and attestation
5. **Pre-Boot Environment**: Minimal environment for user authentication

---

## GRUB Bootloader

### Overview

GRUB (Grand Unified Bootloader) is the standard bootloader for Linux systems and supports:
- LUKS encrypted partition unlocking
- Password-based authentication
- TPM 2.0 integration (as of 2024)
- Secure Boot compatibility

### Technical Capabilities

#### LUKS Support

- **GRUB 2.04+**: Supports LUKS version 1 (LUKS2 support limited)
- **Cryptomount Command**: Built-in command to unlock LUKS devices
- **Encrypted /boot**: Possible since GRUB can decrypt LUKS before loading kernel

**Important Limitations:**
- GRUB doesn't fully support LUKS2 - devices holding /boot need LUKS format version 1
- For LUKS2: Only PBKDF2 is supported (not Argon2id)

#### Configuration

To enable encrypted boot support in GRUB:

```bash
# /etc/default/grub
GRUB_ENABLE_CRYPTODISK=y
```

#### TPM Integration (2024 Update)

**Major Development**: GRUB2 patches incorporating TPM features merged into openSUSE Tumbleweed and SL-Micro 6.0 in August 2024.

**Features:**
- Auto-unlocking using TPM without password
- Integration with `fde-tools` and `pcr-oracle`
- Secure measurement of boot components

### Platform Support

- **Linux**: Native support
- **Windows**: Not used (Windows Boot Manager instead)
- **macOS**: Not typically used (Apple Boot Camp uses custom loader)

---

## UEFI and Secure Boot

### UEFI Overview

**Unified Extensible Firmware Interface (UEFI)** is a modern firmware interface that:
- Replaces legacy BIOS
- Provides programmable boot environment
- Enables Secure Boot
- Required for TPM 2.0 functionality (no CSM/Legacy mode)

### Secure Boot

**Purpose**: Cryptographic verification of boot chain components

**Chain of Trust:**
```
UEFI Firmware (Platform Key)
    ↓ Verifies
Bootloader Signature (KEK/DB keys)
    ↓ Verifies
Kernel/Initrd Signature
    ↓ Verifies
System Drivers
```

**Key Database:**
- **PK (Platform Key)**: Root of trust, typically OEM
- **KEK (Key Exchange Keys)**: Microsoft, Linux Foundation, custom
- **DB (Signature Database)**: Authorized signatures
- **DBX (Forbidden Signatures)**: Revoked signatures

### EFI System Partition (ESP)

**Critical Limitation:** ESP **cannot be encrypted**

**Reasons:**
- UEFI firmware must read ESP before any decryption
- Must be FAT32 format (UEFI specification requirement)
- Contains bootloader files (.efi executables)

**Mitigation:**
- ESP only contains signed bootloader stubs
- Actual boot files (kernel, initrd) stored on encrypted /boot
- Secure Boot ensures ESP integrity

### Custom Keys

Organizations can replace manufacturer Secure Boot keys with custom keys:

```bash
# Generate custom keys
openssl req -new -x509 -newkey rsa:2048 -keyout PK.key -out PK.crt
openssl req -new -x509 -newkey rsa:2048 -keyout KEK.key -out KEK.crt
openssl req -new -x509 -newkey rsa:2048 -keyout db.key -out db.crt

# Sign bootloader
sbsign --key db.key --cert db.crt grubx64.efi
```

---

## TPM 2.0 Integration

### Overview

**Trusted Platform Module (TPM) 2.0** provides:
- Hardware-backed key storage
- Platform integrity measurement
- Sealed storage (keys released only in trusted state)
- Cryptographic acceleration

### Platform Configuration Registers (PCRs)

PCRs store measurements of system state:

| PCR | Measurement |
|-----|-------------|
| 0 | UEFI firmware and embedded drivers |
| 1 | Platform and motherboard configuration |
| 2 | Option ROM code |
| 3 | Option ROM data |
| 4 | Boot Manager code |
| 5 | Boot Manager configuration and data (GPT table) |
| 6 | Platform manufacturer specific |
| 7 | **Secure Boot state** (most commonly used) |
| 8-15 | Static OS measurements |
| 16-23 | Debug and testing |

### Key Sealing

**Concept:** Encrypt the LUKS master key with TPM, sealed to specific PCR values

**Workflow:**
```
1. Generate LUKS master key
2. Seal key to TPM with PCR policy (e.g., PCR 0,7)
3. Store sealed blob in initramfs or /boot
4. At boot:
   a. TPM measures current boot state
   b. If PCR values match policy, unseal key
   c. Use key to unlock LUKS
   d. Otherwise, fall back to password
```

**Security Property:** Key only unsealed if:
- Firmware unchanged (PCR 0)
- Secure Boot configuration unchanged (PCR 7)
- Bootloader unchanged (PCR 4)

### Implementation Tools

#### Linux

**1. systemd-cryptenroll (Modern Approach)**

```bash
# Enroll TPM into LUKS
systemd-cryptenroll /dev/sda2 --tpm2-device=auto --tpm2-pcrs=0+7

# Configure automatic unlock
# /etc/crypttab
luks-root UUID=xxxx none tpm2-device=auto
```

**2. Clevis (Alternative)**

```bash
# Install
apt-get install clevis clevis-tpm2 clevis-luks clevis-initramfs

# Bind LUKS to TPM
clevis luks bind -d /dev/sda2 tpm2 '{"pcr_ids":"0,7"}'

# Update initramfs
update-initramfs -u
```

#### Windows (BitLocker)

```powershell
# Enable BitLocker with TPM
Enable-BitLocker -MountPoint "C:" -EncryptionMethod XtsAes256 `
    -UsedSpaceOnly -TpmProtector

# Add PIN for additional security
Add-BitLockerKeyProtector -MountPoint "C:" -TpmAndPinProtector
```

### Kernel Requirements

**Linux Kernel 6.10+**: TPM must support AES-128-CFB for session encryption

**Compatibility Issue:** Older Intel PTT (Platform Trust Technology) TPMs may not support this mode and will fail to initialize.

---

## Platform-Specific Approaches

### Linux

#### Recommended Stack

```
UEFI Secure Boot
    ↓
Signed GRUB bootloader
    ↓
TPM-sealed key OR password prompt
    ↓
Unlock LUKS2 root
    ↓
Load kernel/initrd from encrypted /boot
```

#### Disk Layout

```
/dev/sda1: ESP (FAT32, 512MB, unencrypted)
    └── EFI/
        ├── BOOT/BOOTX64.EFI (signed GRUB stub)
        └── ubuntu/grubx64.efi (signed)

/dev/sda2: /boot (ext4, 1GB, LUKS1)
    └── grub/, kernel, initrd

/dev/sda3: / (ext4/btrfs, remaining, LUKS2 with Argon2id)
    └── Root filesystem
```

#### Tools and Packages

```bash
# Ubuntu/Debian
apt-get install cryptsetup grub-efi-amd64-signed shim-signed \
    systemd tpm2-tools clevis clevis-tpm2

# Arch Linux
pacman -S cryptsetup grub efibootmgr sbctl tpm2-tools systemd
```

### Windows

#### BitLocker Architecture

```
UEFI Firmware
    ↓
Windows Boot Manager (bootmgr.efi)
    ↓
TPM validation OR Pre-boot PIN
    ↓
Unseal FVEK (Full Volume Encryption Key)
    ↓
Unlock system volume
    ↓
Load Windows kernel
```

#### TPM Modes

1. **TPM-only**: Automatic unlock (no user interaction)
2. **TPM + PIN**: Requires user PIN entry
3. **TPM + Startup Key**: Requires USB drive
4. **TPM + PIN + Startup Key**: Maximum security

#### Requirements

- **TPM 2.0** in native UEFI mode (no Legacy/CSM)
- **Windows 11/10** Pro/Enterprise (Home supports device encryption only)
- **Secure Boot** enabled (recommended)

#### Group Policy Configuration

```
Computer Configuration
  → Administrative Templates
    → Windows Components
      → BitLocker Drive Encryption
        → Operating System Drives
          → Require additional authentication at startup
            [✓] Allow BitLocker without a compatible TPM
            [✓] Configure TPM startup PIN: Require startup PIN with TPM
```

### macOS

#### FileVault 2

macOS uses **FileVault 2** for full disk encryption:

- **Not bootloader-based**: Integrated into macOS boot process
- **APFS encryption**: Volume-level encryption, not block device
- **Recovery Key**: 24-character alphanumeric key
- **T2/Apple Silicon**: Hardware encryption with Secure Enclave

**Compatibility Note:** Secure Cryptor cannot replace FileVault on macOS but can provide additional encryption for data volumes.

---

## Implementation Recommendations

### Phase 1: Research Complete ✅

- [x] GRUB capabilities and limitations
- [x] UEFI Secure Boot integration
- [x] TPM 2.0 sealing mechanisms
- [x] Platform-specific approaches

### Phase 2: Design Pre-Boot Environment

**Requirements:**
1. Minimal bootloader or GRUB module
2. Password input interface (keyboard handling)
3. TPM communication (if available)
4. Fallback to password if TPM fails
5. Decrypt master key from header
6. Unlock volume sectors

**Options:**

#### Option A: GRUB Module (Linux-focused)
**Pros:**
- Leverages existing GRUB infrastructure
- Well-tested cryptographic support
- Automatic Secure Boot integration

**Cons:**
- Linux-specific
- Limited to LUKS1 for /boot
- Requires GRUB_ENABLE_CRYPTODISK

#### Option B: UEFI Application (Cross-platform)
**Pros:**
- Platform-independent (works on any UEFI system)
- Full control over UX
- Can support both Windows and Linux

**Cons:**
- Must implement everything from scratch
- Requires signing for Secure Boot
- More complex development

#### Option C: Hybrid Approach (Recommended)
**Linux:** Integrate with GRUB + systemd-cryptenroll
**Windows:** Custom UEFI pre-boot app or Windows Boot Manager integration
**macOS:** Volume encryption only (not system volume)

### Phase 3: TPM Integration Design

**Key Management:**

```rust
// Pseudo-code
struct BootKey {
    // Sealed by TPM
    tpm_sealed_blob: Option<Vec<u8>>,
    tpm_pcr_policy: Vec<u8>, // PCRs 0,7

    // Encrypted with password-derived key
    password_encrypted_master_key: Vec<u8>,
    password_salt: [u8; 32],
}

fn unlock_volume(header: &VolumeHeader) -> Result<MasterKey> {
    // Try TPM first
    if let Some(blob) = header.tpm_sealed_blob {
        if let Ok(key) = tpm_unseal(blob, &header.pcr_policy) {
            return Ok(key);
        }
    }

    // Fall back to password
    let password = prompt_password()?;
    let key = derive_key_from_password(&password, &header.salt)?;
    decrypt_master_key(&header.encrypted_master_key, &key)
}
```

### Phase 4: Secure Boot Compatibility

**Self-Signing Workflow:**

1. Generate custom keys (PK, KEK, DB)
2. Enroll keys in UEFI firmware
3. Sign bootloader with custom DB key
4. Sign kernel modules (if needed)
5. Test boot process

**Microsoft Signing (Alternative):**
- Submit bootloader to Microsoft for signing
- Requires EV code signing certificate
- Works with standard Secure Boot

---

## Security Considerations

### Threat Model

**Threats Mitigated:**
- ✅ Offline disk access (device theft)
- ✅ Evil maid attacks (Secure Boot + TPM PCR binding)
- ✅ Bootloader tampering (Secure Boot verification)
- ✅ Firmware backdoors (TPM attestation)

**Threats NOT Mitigated:**
- ❌ DMA attacks (Thunderbolt, FireWire) - requires IOMMU
- ❌ Cold boot attacks - requires fast memory encryption
- ❌ Compromised UEFI firmware - requires hardware root of trust
- ❌ Physical TPM access (advanced attacks) - requires physical security

### Best Practices

1. **Always enable Secure Boot** when using TPM sealing
2. **Bind to PCR 7** (Secure Boot state) at minimum
3. **Provide password fallback** for TPM failures
4. **Implement anti-brute-force** for password attempts
5. **Use Argon2id** for password-based key derivation
6. **Clear memory** after use (zeroize)
7. **Audit boot chain** regularly

### Recovery Scenarios

| Scenario | Recovery Method |
|----------|----------------|
| Forgotten password | Recovery key (separate slot) |
| TPM failure | Password fallback |
| BIOS update (PCR change) | Re-seal to new PCRs or use password |
| Motherboard replacement | Password or recovery key |
| Secure Boot disabled | Password authentication |

---

## References

### Documentation

- [Debian LUKS Encrypted Boot](https://cryptsetup-team.pages.debian.net/cryptsetup/encrypted-boot.html)
- [Arch Linux TPM Guide](https://wiki.archlinux.org/title/Trusted_Platform_Module)
- [Microsoft BitLocker Docs](https://learn.microsoft.com/en-us/windows/security/operating-system-security/data-protection/bitlocker/)
- [systemd-cryptenroll Manual](https://www.freedesktop.org/software/systemd/man/systemd-cryptenroll.html)

### Tutorials

- [Secure Boot and TPM2 for LUKS - Manjaro](https://forum.manjaro.org/t/howto-using-secure-boot-and-tpm2-to-unlock-luks-partition-on-boot/101626)
- [Arch Install with LUKS2, LVM2, Secure Boot, TPM2](https://github.com/joelmathewthomas/archinstall-luks2-lvm2-secureboot-tpm2)
- [Full Disk Encryption with GRUB2 and TPM - SUSE](https://www.suse.com/c/full-disk-encryption-grub2-tpm/)

### Tools

- **systemd-cryptenroll**: Modern TPM2 integration for systemd-based systems
- **Clevis**: Framework for automated decryption using TPM, Tang, or other methods
- **sbctl**: Secure Boot key management for Linux
- **tpm2-tools**: Command-line tools for TPM 2.0

---

## Conclusion

Pre-boot authentication for full disk encryption is achievable across all major platforms with varying levels of integration:

- **Linux**: Excellent support via GRUB + systemd-cryptenroll + TPM2
- **Windows**: Native BitLocker provides complete solution
- **macOS**: FileVault 2 integrated, custom solution not feasible for system volume

**Recommended Approach for Secure Cryptor:**

1. **Phase 5a (Post-Boot)**: Encrypt data volumes, mountable after OS boots
   - Cross-platform support
   - Simpler implementation
   - Immediate value

2. **Phase 5b (True Pre-Boot)**: System volume encryption
   - Linux: GRUB module + TPM integration
   - Windows: UEFI pre-boot application or EFI driver
   - Requires extensive testing and Secure Boot signing

This research provides the foundation for implementing Phase 5 boot-time encryption features.

---

*Document created: 2025-11-16*
*Last updated: 2025-11-16*
*Status: Research Complete*
