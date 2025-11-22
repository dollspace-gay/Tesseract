# YubiKey Integration Guide

## Overview

This document describes the YubiKey HMAC-SHA1 challenge-response integration for Secure Cryptor.

## Current Status

**⚠️ Implementation Note**: The YubiKey feature is currently a framework/interface implementation. The `yubikey-hmac-otp` crate (v0.10.2) has dependency compatibility issues with `generic-array` that prevent compilation.

## Architecture

The YubiKey integration provides two-factor authentication for encryption keys by combining:
1. **Password-derived key** (Argon2id) - Something you know
2. **YubiKey response** (HMAC-SHA1) - Something you have

These are combined using HKDF-SHA256 to create the final encryption key.

## Module Structure

```
src/hsm/
├── mod.rs           # Hardware Security Module trait
└── yubikey.rs       # YubiKey implementation
```

### Key Components

1. **`HardwareSecurityModule` trait** - Generic interface for HSM devices
2. **`YubiKey` struct** - YubiKey-specific implementation
3. **`YubiKeySlot`** - Slot configuration (Slot1/Slot2)
4. **`YubiKeyConfig`** - Device configuration

## Features

### Implemented (Interface)

- ✅ HSM trait definition
- ✅ YubiKey configuration structure
- ✅ Slot selection (Slot 1 / Slot 2)
- ✅ Backup key mechanism
- ✅ Key derivation interface
- ✅ Comprehensive error handling
- ✅ Test suite (requires hardware)

### Pending (Dependency Issue)

- ⏳ Actual YubiKey USB communication
- ⏳ HMAC-SHA1 challenge-response
- ⏳ Device enumeration
- ⏳ Firmware version detection

## Usage (When Implemented)

### Basic Usage

```rust
use secure_cryptor::hsm::yubikey::{YubiKey, YubiKeySlot};

// Initialize YubiKey
let yubikey = YubiKey::new()?;

// Check if available
if yubikey.is_available() {
    // Set slot
    yubikey.set_slot(YubiKeySlot::Slot2);

    // Derive key with YubiKey 2FA
    let password = b"my-password";
    let salt = [0u8; 32];
    let challenge = [0x42u8; 32];

    let key = yubikey.derive_key(password, &salt, &challenge)?;
}
```

### With Backup Key

```rust
let mut yubikey = YubiKey::new()?;

// Generate and set backup key
let backup = YubiKey::generate_backup_key();
yubikey.set_backup_key(backup.to_vec());

// Now encryption works even if YubiKey is unavailable
let key = yubikey.derive_key(password, &salt, &challenge)?;
```

### Configuration

```rust
use secure_cryptor::hsm::yubikey::{YubiKey, YubiKeyConfig, YubiKeySlot};
use std::time::Duration;

let config = YubiKeyConfig {
    slot: YubiKeySlot::Slot2,
    timeout: Duration::from_secs(5),
    allow_backup: true,
    serial: Some(12345678), // Specific YubiKey serial
};

let yubikey = YubiKey::with_config(config)?;
```

## Dependency Resolution Options

### Option 1: Wait for Upstream Fix

The `yubikey-hmac-otp` crate needs to update its `generic-array` dependency:

```toml
[dependencies]
yubikey-hmac-otp = "0.10.2"  # Currently broken
```

**Issue**: `generic-array` version incompatibility

**Resolution**: Wait for crate author to update dependencies

### Option 2: Use Fork

Create a fork of `yubikey-hmac-otp` and fix the dependency:

```toml
[dependencies]
yubikey-hmac-otp = { git = "https://github.com/yourusername/yubikey-hmac-otp", branch = "fix-deps" }
```

### Option 3: Direct USB HID Implementation

Implement HMAC-SHA1 challenge-response using `hidapi` directly:

```toml
[dependencies]
hidapi = "2.6"
sha-1 = "0.10"
hmac = "0.12"
```

This requires implementing the YubiKey USB HID protocol manually.

### Option 4: Use PIV for Authentication (Alternative)

Use the main `yubikey` crate with PIV for authentication instead of HMAC:

```toml
[dependencies]
yubikey = "0.8.0"
```

This uses RSA/ECC signatures instead of HMAC challenge-response.

## YubiKey HMAC-SHA1 Protocol

### Challenge-Response Flow

1. **Send Challenge**: 64-byte challenge to YubiKey
2. **Receive Response**: 20-byte HMAC-SHA1 response
3. **Combine with Password**: Use HKDF to mix password-derived key with YubiKey response

### Slot Configuration

YubiKeys have 2 configuration slots:
- **Slot 1**: Short press (1-3 seconds)
- **Slot 2**: Long press (3-5 seconds)

Each slot can be configured for HMAC-SHA1 challenge-response mode.

### USB HID Protocol

```
Feature Report:
- Byte 0: Slot (0x30 = Slot1, 0x38 = Slot2)
- Bytes 1-64: Challenge data
- Bytes 65-84: Response data (HMAC-SHA1 output)
```

## Security Considerations

### Backup Keys

**Purpose**: Allow decryption if YubiKey is lost/unavailable

**Storage**:
- Encrypt backup key with strong password
- Store in secure location (password manager, secure vault)
- Never store backup key in plaintext

**Generation**:
```rust
let backup = YubiKey::generate_backup_key();  // 32 bytes CSPRNG
```

### Key Derivation

The final encryption key is derived as:

```
password_key = Argon2id(password, salt)
yubikey_response = HMAC-SHA1(yubikey_secret, challenge)
final_key = HKDF-SHA256(password_key, yubikey_response)
```

This provides:
- **Something you know**: Password
- **Something you have**: YubiKey
- **Defense in depth**: Both factors required

### Attack Scenarios

| Attack | Mitigation |
|--------|------------|
| Password guessing | Argon2id memory-hard KDF |
| YubiKey theft | Still needs password |
| Both stolen | Use strong password |
| YubiKey loss | Backup key mechanism |

## Testing

### Unit Tests

```bash
# Run tests (hardware not required)
cargo test --features yubikey

# Run integration tests (requires YubiKey)
cargo test --features yubikey -- --ignored
```

### Integration Tests

The test suite includes:
- ✅ Configuration validation
- ✅ Slot conversion
- ✅ Backup key generation
- ✅ Invalid input handling
- ⏳ Hardware communication (requires YubiKey)

Tests marked `#[ignore]` require an actual YubiKey device and can be run with:

```bash
cargo test --features yubikey -- --ignored --test-threads=1
```

## Troubleshooting

### Build Errors

**Issue**: `yubikey-hmac-otp` fails to compile

**Solution**:
1. Check `generic-array` version in Cargo.lock
2. Try `cargo update`
3. Use alternative dependency resolution (see options above)

### No YubiKey Detected

**Causes**:
- YubiKey not plugged in
- USB permissions (Linux)
- Driver issues (Windows)

**Linux Permissions**:
```bash
# Add udev rule
echo 'SUBSYSTEM=="usb", ATTRS{idVendor}=="1050", MODE="0666"' | sudo tee /etc/udev/rules.d/70-yubikey.rules
sudo udevadm control --reload-rules
```

**Windows**:
- Install YubiKey Manager
- Ensure USB device drivers are loaded

### Challenge-Response Fails

**Causes**:
- Slot not configured for HMAC-SHA1
- Wrong slot selected
- YubiKey timeout

**Configuration**:
Use YubiKey Manager or `ykman` CLI to configure slot:

```bash
# Configure Slot 2 for HMAC-SHA1
ykman otp chalresp --generate 2
```

## Future Enhancements

1. **Multi-YubiKey Support**: Allow multiple backup YubiKeys
2. **FIDO2 Integration**: Add WebAuthn/FIDO2 authentication
3. **TPM Integration**: Combine with TPM 2.0 for defense in depth
4. **NFC Support**: Allow YubiKey 5 NFC communication
5. **YubiKey Bio**: Biometric authentication support

## References

- [YubiKey HMAC-SHA1 Challenge-Response](https://developers.yubico.com/OTP/OTPs_Explained.html)
- [yubikey-hmac-otp Crate](https://crates.io/crates/yubikey-hmac-otp)
- [YubiKey Manager CLI](https://developers.yubico.com/yubikey-manager/)
- [YubiKey USB HID Specification](https://developers.yubico.com/Software_Projects/)

## Contributing

To complete the YubiKey integration:

1. **Fix Dependencies**: Update `yubikey-hmac-otp` or fork it
2. **Test with Hardware**: Verify with physical YubiKey 5 devices
3. **Add Examples**: Create command-line examples
4. **Update Documentation**: Add real-world usage examples
5. **CI/CD**: Add hardware testing to CI (if possible)

## License

Same as Secure Cryptor (MIT)
