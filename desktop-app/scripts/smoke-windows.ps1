param([Parameter(Mandatory=$true)][string]$AppExe,[Parameter(Mandatory=$true)][string]$TestDirectory)
$ErrorActionPreference = 'Stop'
[System.Net.ServicePointManager]::Expect100Continue = $true
$base = 'http://127.0.0.1:49379'
New-Item -ItemType Directory -Path $TestDirectory -Force | Out-Null
function Start-TestApp {
    $process = Start-Process -FilePath $AppExe -ArgumentList @('--serve-only','--port','49379','--data-dir',('"{0}"' -f $TestDirectory)) -PassThru -WindowStyle Hidden -RedirectStandardOutput (Join-Path $TestDirectory 'stdout.log') -RedirectStandardError (Join-Path $TestDirectory 'stderr.log')
    for ($attempt = 0; $attempt -lt 50; $attempt++) {
        if ($process.HasExited) { throw "Windows executable exited: $($process.ExitCode)" }
        try { $health = Invoke-WebRequest "$base/health" -UseBasicParsing -TimeoutSec 1; if ($health.Content -eq 'aiNavLog desktop 1') { return $process } } catch {}
        Start-Sleep -Milliseconds 200
    }
    Stop-Process -Id $process.Id -Force
    throw 'Windows desktop server did not start'
}
function Get-AppConfiguration {
    $html = (Invoke-WebRequest "$base/elec/maintenance" -UseBasicParsing).Content
    if ($html -notmatch 'globalThis\.__AINAVLOG_DESKTOP__=(\{[^<]+?\});') {throw 'Missing packaged runtime configuration'}
    return $Matches[1] | ConvertFrom-Json
}
function Send-Command($config,$request) {
    $bytes = [System.Text.Encoding]::UTF8.GetBytes(($request | ConvertTo-Json -Depth 10 -Compress))
    return Invoke-RestMethod "$base/maintenance" -Method Post -ContentType 'application/json' -Headers @{'Origin'=$base;'X-AiNavLog-Token'=$config.token} -Body $bytes
}
$process = $null
try {
    $process = Start-TestApp
    $config = Get-AppConfiguration
    $initial = Send-Command $config @{op='list'}
    if ($initial.error -or $initial.entries.Count -ne 0) {throw 'Expected fresh Windows test storage'}
    $entry = @{id='windows-smoke';equipmentId='alternator';system='electrical';date='2026-10-30';kind='Inspection';description='Windows native SQLite test';provider='Owner';hours=312.5;parts='';costMinor=12345;currency='CAD';reference='Test';notes='';nextDate='2027-04-30';nextHours=500}
    $saved = Send-Command $config @{op='save';entry=$entry}
    if (-not $saved.ok) {throw 'Windows save failed'}
    $saved = Send-Command $config @{op='save';entry=$entry}
    $rows = Send-Command $config @{op='list'}
    if ($rows.entries.Count -ne 1 -or $rows.entries[0].costMinor -ne 12345) {throw 'Retry duplicated or corrupted record'}
    try {Invoke-WebRequest "$base/maintenance" -UseBasicParsing -Method Post -ContentType 'application/json' -Headers @{'Origin'=$base} -Body '{"op":"list"}' | Out-Null;throw 'Missing session token was accepted'} catch {if ($_.Exception.Response.StatusCode.value__ -ne 403) {throw}}
    Stop-Process -Id $process.Id -Force; $process.WaitForExit();$process=$null
    $process=Start-TestApp;$config=Get-AppConfiguration
    $rows=Send-Command $config @{op='list'}
    if ($rows.entries.Count -ne 1 -or $rows.entries[0].description -ne 'Windows native SQLite test') {throw 'Record did not survive Windows app restart'}
    Write-Output 'Windows EXE: embedded UI, native SQLite save/retry, token enforcement and process-restart retention passed.'
} finally {if ($null -ne $process -and -not $process.HasExited){Stop-Process -Id $process.Id -Force}}
