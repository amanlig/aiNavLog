# Run in Windows PowerShell with Rust MSVC and Visual Studio C++ Build Tools installed.
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    cargo build --release --locked --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Windows build failed.' }
    Write-Host "Executable: $PSScriptRoot\target\x86_64-pc-windows-msvc\release\ainavlog-device-interface.exe"
} finally {
    Pop-Location
}
