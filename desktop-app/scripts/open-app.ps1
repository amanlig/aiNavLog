$ErrorActionPreference = 'Stop'
$executable = Join-Path $PSScriptRoot '..\builds\windows-x64\aiNavLog.exe'
$process = Start-Process -FilePath $executable -PassThru
for ($attempt=0; $attempt -lt 50; $attempt++) {
    try {
        $health = Invoke-WebRequest 'http://127.0.0.1:49378/health' -UseBasicParsing -TimeoutSec 1
        if ($health.Content -eq 'aiNavLog desktop 1') {
            Write-Output "aiNavLog Windows app launched (process $($process.Id))."
            return
        }
    } catch {}
    if ($process.HasExited) { throw "aiNavLog closed during startup: $($process.ExitCode)" }
    Start-Sleep -Milliseconds 200
}
throw 'App did not become ready'
