# Installation script for Secure Cryptor auto-mount service on Windows
# Run as Administrator

param(
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$ServiceName = "SecureCryptorAutoMount"
$ServiceDisplayName = "Secure Cryptor Auto-Mount"
$ServiceDescription = "Automatically mounts encrypted volumes at system startup"
$InstallDir = "$env:ProgramFiles\SecureCryptor"
$ConfigDir = "$env:ProgramData\SecureCryptor"
$BinaryName = "secure-cryptor.exe"

# Check if running as Administrator
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "This script must be run as Administrator"
    exit 1
}

if ($Uninstall) {
    Write-Host "Uninstalling Secure Cryptor Auto-Mount Service..." -ForegroundColor Yellow

    # Stop service if running
    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($service) {
        if ($service.Status -eq 'Running') {
            Write-Host "Stopping service..." -ForegroundColor Yellow
            Stop-Service -Name $ServiceName -Force
        }

        Write-Host "Removing service..." -ForegroundColor Yellow
        sc.exe delete $ServiceName
    }

    Write-Host "Service uninstalled successfully" -ForegroundColor Green
    exit 0
}

# Installation
Write-Host "Installing Secure Cryptor Auto-Mount Service..." -ForegroundColor Green
Write-Host ""

# Check if binary exists
$BinaryPath = Get-Command secure-cryptor.exe -ErrorAction SilentlyContinue
if (-not $BinaryPath) {
    Write-Error "secure-cryptor.exe not found in PATH. Please install Secure Cryptor first."
    exit 1
}

# Create installation directory
Write-Host "Creating installation directory..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

# Copy binary
Write-Host "Copying binary..." -ForegroundColor Yellow
Copy-Item -Path $BinaryPath.Source -Destination "$InstallDir\$BinaryName" -Force

# Create configuration directory
Write-Host "Creating configuration directory..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null

# Create default configuration if it doesn't exist
$ConfigFile = "$ConfigDir\automount.json"
if (-not (Test-Path $ConfigFile)) {
    Write-Host "Creating default configuration..." -ForegroundColor Yellow
    $defaultConfig = @{
        volumes = @()
        global_timeout = 120
        background = $true
    } | ConvertTo-Json -Depth 10

    $defaultConfig | Out-File -FilePath $ConfigFile -Encoding UTF8

    Write-Host "Created $ConfigFile" -ForegroundColor Green
    Write-Host "Edit this file to add your encrypted volumes" -ForegroundColor Cyan
}

# Install service
Write-Host "Installing Windows service..." -ForegroundColor Yellow

$serviceBinaryPath = "`"$InstallDir\$BinaryName`" service run"

# Check if service already exists
$existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($existingService) {
    Write-Host "Service already exists, updating..." -ForegroundColor Yellow
    Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
    sc.exe delete $ServiceName
    Start-Sleep -Seconds 2
}

# Create service
sc.exe create $ServiceName binPath= $serviceBinaryPath start= auto
sc.exe description $ServiceName $ServiceDescription
sc.exe failure $ServiceName reset= 86400 actions= restart/5000/restart/5000/restart/5000

# Set service to depend on TPM service (if available)
$tpmService = Get-Service -Name "TBS" -ErrorAction SilentlyContinue
if ($tpmService) {
    Write-Host "Configuring TPM dependency..." -ForegroundColor Yellow
    sc.exe config $ServiceName depend= TBS
}

Write-Host ""
Write-Host "Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Edit configuration: notepad $ConfigFile"
Write-Host "2. Start service: Start-Service $ServiceName"
Write-Host "3. Check status: Get-Service $ServiceName"
Write-Host "4. View logs: Get-EventLog -LogName Application -Source SecureCryptor -Newest 10"
Write-Host ""
Write-Host "Example configuration:" -ForegroundColor Cyan
Write-Host @"
{
  "volumes": [
    {
      "id": "data-volume",
      "name": "Encrypted Data",
      "container_path": "D:\\secure.crypt",
      "mount_point": "E:\\",
      "auth": {"method": "tpm", "pcr_indices": [0, 7]},
      "read_only": false,
      "required": false,
      "timeout": 60,
      "auto_unmount": true
    }
  ],
  "global_timeout": 120,
  "background": true
}
"@

Write-Host ""
Write-Host "To uninstall, run: .\install-service-windows.ps1 -Uninstall" -ForegroundColor Yellow
