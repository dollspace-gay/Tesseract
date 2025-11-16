# Secure Cryptor Auto-Mount Services

This directory contains system service configurations for automatically mounting encrypted volumes at system boot.

## Linux (systemd)

### Installation

1. Copy the service file:
   ```bash
   sudo cp secure-cryptor-automount.service /etc/systemd/system/
   ```

2. Create configuration directory:
   ```bash
   sudo mkdir -p /etc/secure-cryptor
   ```

3. Create auto-mount configuration:
   ```bash
   sudo secure-cryptor config automount > /etc/secure-cryptor/automount.json
   # Edit the file to add your volumes
   sudo nano /etc/secure-cryptor/automount.json
   ```

4. Enable and start the service:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable secure-cryptor-automount.service
   sudo systemctl start secure-cryptor-automount.service
   ```

5. Check status:
   ```bash
   sudo systemctl status secure-cryptor-automount.service
   sudo journalctl -u secure-cryptor-automount.service
   ```

### Configuration Example

`/etc/secure-cryptor/automount.json`:
```json
{
  "volumes": [
    {
      "id": "home-encrypted",
      "name": "Encrypted Home",
      "container_path": "/data/home.crypt",
      "mount_point": "/home/user/encrypted",
      "auth": {
        "method": "tpm",
        "pcr_indices": [0, 7]
      },
      "read_only": false,
      "required": true,
      "timeout": 60,
      "auto_unmount": true
    }
  ],
  "global_timeout": 120,
  "background": true
}
```

## Windows (Windows Service)

### Installation

1. Copy files to Program Files:
   ```powershell
   Copy-Item -Path "SecureCryptorService.exe" -Destination "C:\Program Files\SecureCryptor\"
   ```

2. Install the service:
   ```powershell
   sc.exe create SecureCryptorAutoMount binPath= "C:\Program Files\SecureCryptor\SecureCryptorService.exe" start= auto
   ```

3. Create configuration:
   ```powershell
   New-Item -ItemType Directory -Path "C:\ProgramData\SecureCryptor" -Force
   secure-cryptor.exe config automount > "C:\ProgramData\SecureCryptor\automount.json"
   # Edit the JSON file
   ```

4. Start the service:
   ```powershell
   Start-Service SecureCryptorAutoMount
   ```

5. Check status:
   ```powershell
   Get-Service SecureCryptorAutoMount
   Get-EventLog -LogName Application -Source SecureCryptor -Newest 10
   ```

### Configuration Example

`C:\ProgramData\SecureCryptor\automount.json`:
```json
{
  "volumes": [
    {
      "id": "data-volume",
      "name": "Encrypted Data",
      "container_path": "D:\\secure.crypt",
      "mount_point": "E:\\",
      "auth": {
        "method": "tpm",
        "pcr_indices": [0, 7]
      },
      "read_only": false,
      "required": false,
      "timeout": 60,
      "auto_unmount": true
    }
  ],
  "global_timeout": 120,
  "background": true
}
```

## Authentication Methods

### 1. TPM (Automatic)

No user interaction required. Volume automatically unlocked if system integrity verified.

```json
{
  "auth": {
    "method": "tpm",
    "pcr_indices": [0, 7]
  }
}
```

**PCR Indices:**
- `0`: UEFI firmware
- `7`: Secure Boot state
- `4,7`: Secure Boot + Bootloader

### 2. Password Prompt (Manual)

Prompts user for password via GUI dialog or CLI.

```json
{
  "auth": {
    "method": "prompt"
  }
}
```

**Linux:** Uses `systemd-ask-password` or GUI prompt (if X11/Wayland available)
**Windows:** Uses Windows Credential UI

### 3. Keyring (Stored Password)

Retrieves password from system keyring/credential manager.

```json
{
  "auth": {
    "method": "keyring",
    "entry_name": "secure-cryptor-volume-home"
  }
}
```

**Linux:** Uses `libsecret` / GNOME Keyring / KWallet
**Windows:** Uses Windows Credential Manager

**Setup:**
```bash
# Linux
secret-tool store --label="Secure Cryptor Home Volume" \
  service secure-cryptor volume home-encrypted

# Windows
cmdkey /generic:secure-cryptor-volume-data /user:volume /pass:mypassword
```

## Troubleshooting

### Linux

**Volume not mounting:**
```bash
# Check logs
journalctl -u secure-cryptor-automount.service -b

# Test manually
sudo /usr/bin/secure-cryptor mount /data/home.crypt /mnt/test --password

# Verify TPM
sudo tpm2_pcrread sha256:0,7
```

**Permission issues:**
```bash
# Ensure mount point exists and has correct permissions
sudo mkdir -p /mnt/encrypted
sudo chown user:user /mnt/encrypted
```

### Windows

**Service won't start:**
```powershell
# Check event log
Get-EventLog -LogName Application -Source SecureCryptor -Newest 20

# Test manually
& "C:\Program Files\SecureCryptor\secure-cryptor.exe" mount "D:\secure.crypt" "E:\" --password

# Verify TPM
tpm.msc  # TPM Management Console
```

**Drive letter conflicts:**
- Ensure the mount point drive letter is available
- Use Disk Management to verify

## Security Considerations

### TPM Auto-Mount

**Pros:**
- No user interaction required
- Keys sealed to system state (Secure Boot, firmware)
- Automatic detection of tampering

**Cons:**
- Vulnerable if attacker has physical access while system is running
- BIOS/firmware updates may require re-sealing

**Recommendation:** Use TPM for convenience + password for data at rest security

### Password Storage

**NEVER** store passwords in plain text in the configuration file.

**Safe options:**
1. TPM sealing
2. System keyring with master password
3. Prompt on boot

**Unsafe:**
- Plain text in config file
- Environment variables
- Registry (Windows)

### Required vs Optional Volumes

**Required volumes:**
- Boot waits for successful mount
- System won't fully start if mount fails
- Use for critical data

**Optional volumes:**
- System boots normally
- Mount happens in background
- User notified if mount fails

## Advanced Configuration

### Multiple Volumes with Dependencies

Some volumes may depend on others (e.g., database on encrypted volume).

Use systemd dependencies (Linux):
```ini
[Unit]
After=secure-cryptor-automount.service
Requires=secure-cryptor-automount.service
```

Or Windows service dependencies:
```powershell
sc.exe config MyService depend= SecureCryptorAutoMount
```

### Conditional Mounting

Mount only if specific hardware present (e.g., YubiKey):

```json
{
  "auth": {
    "method": "prompt"
  },
  "conditions": {
    "hardware_token_present": "Yubico YubiKey"
  }
}
```

### Network Volumes

For remote unlock (enterprise scenarios):

```json
{
  "auth": {
    "method": "network",
    "server": "https://keyserver.example.com",
    "token": "/etc/secure-cryptor/machine.token"
  }
}
```

## See Also

- [Boot Loader Research](../bootloader-research.md)
- [Pre-Boot Authentication Design](../preboot-auth-design.md)
- [Container Format Specification](../container-format.md)
