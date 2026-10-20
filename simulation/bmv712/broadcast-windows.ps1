param(
    [ValidateRange(1, 3600)][int]$DurationSeconds = 300,
    [switch]$CheckFixture
)
& (Join-Path $PSScriptRoot '../orion/broadcast-windows.ps1') -Model bmv712 -DurationSeconds $DurationSeconds -CheckFixture:$CheckFixture
