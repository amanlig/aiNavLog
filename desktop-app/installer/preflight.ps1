param([switch]$CheckOnly)
$ErrorActionPreference = 'Stop'
try {
    if (Get-Process -Name 'aiNavLog' -ErrorAction SilentlyContinue) {
        throw 'Close every aiNavLog window, then run setup again.'
    }
    if ($CheckOnly) { exit 0 }
    $payload = Join-Path $PSScriptRoot 'payload'
    $receipt = Get-Content -LiteralPath (Join-Path $payload 'build-receipt.json') -Raw | ConvertFrom-Json
    $actual = (Get-FileHash -LiteralPath (Join-Path $payload 'aiNavLog.exe') -Algorithm SHA256).Hash
    if ($actual -ne $receipt.exeSha256) { throw 'The app checksum does not match this installer. Obtain a fresh setup file.' }
    $dataDirectory = Join-Path $env:LOCALAPPDATA 'aiNavLog\Desktop'
    if (Test-Path -LiteralPath (Join-Path $dataDirectory 'ainavlog.sqlite3')) {
        $backupDirectory = Join-Path $dataDirectory ('Backups\before-update-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0,8))
        New-Item -ItemType Directory -Path $backupDirectory -Force | Out-Null
        foreach ($name in @('ainavlog.sqlite3', 'ainavlog.sqlite3-wal', 'ainavlog.sqlite3-shm')) {
            $sourceFile = Join-Path $dataDirectory $name
            if (Test-Path -LiteralPath $sourceFile) { Copy-Item -LiteralPath $sourceFile -Destination $backupDirectory }
        }
        Write-Output "Existing records backed up to $backupDirectory"
    }
    Write-Output 'App files verified. Existing data will be retained.'
} catch {
    Write-Output $_.Exception.Message
    exit 1
}
