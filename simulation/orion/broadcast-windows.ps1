param(
    [ValidateRange(1, 3600)][int]$DurationSeconds = 300,
    [ValidateSet('tr', 'bmv712')][string]$Model = 'tr',
    [switch]$CheckFixture
)
# Run with Windows PowerShell 5.1. Public synthetic key; no charger control.
$ErrorActionPreference = 'Stop'
function New-OrionPayload([uint16]$Counter, [byte[]]$Plaintext) {
    if ($Plaintext.Length -gt 16) { throw 'Payload must fit one AES block.' }
    $aes = [System.Security.Cryptography.Aes]::Create()
    $aes.Mode = [System.Security.Cryptography.CipherMode]::ECB
    $aes.Padding = [System.Security.Cryptography.PaddingMode]::None
    $aes.Key = [byte[]](0..15)
    $iv = New-Object byte[] 16
    $iv[0] = $Counter -band 255
    $iv[1] = $Counter -shr 8
    $encryptor = $aes.CreateEncryptor()
    try {
        # TR plaintext fits one AES block. CTR's first keystream block is AES(nonce).
        $stream = $encryptor.TransformFinalBlock($iv, 0, 16)
        $record = if ($Model -eq 'bmv712') { 2 } else { 4 }
        $payload = [byte[]](0x10,0x02,0x34,0x12,$record,$iv[0],$iv[1],0x00) + $Plaintext
        for ($i = 0; $i -lt $Plaintext.Length; $i++) {
            $payload[8 + $i] = $Plaintext[$i] -bxor $stream[$i]
        }
        return ,$payload
    } finally {
        $encryptor.Dispose()
        $aes.Dispose()
    }
}
function Set-Field([byte[]]$Bytes, [int]$Offset, [int]$Width, [long]$Value) {
    for ($bit = 0; $bit -lt $Width; $bit++) {
        $index = [int][Math]::Floor(($Offset + $bit) / 8)
        $mask = 1 -shl (($Offset + $bit) % 8)
        if (($Value -shr $bit) -band 1) { $Bytes[$index] = $Bytes[$index] -bor $mask }
    }
}
function New-BatteryPlaintext([int]$Voltage, [int]$Current, [int]$Consumed, [int]$Soc, [int]$StarterVoltage = -1) {
    $bytes = New-Object byte[] 16
    Set-Field $bytes 0 16 120 # illustrative time-to-go, minutes
    Set-Field $bytes 16 16 $Voltage # centivolts
    Set-Field $bytes 32 16 0 # no alarms
    if ($StarterVoltage -lt 0) {
        Set-Field $bytes 48 16 65535
        Set-Field $bytes 64 2 3 # auxiliary disabled (legacy fixtures)
    } else {
        Set-Field $bytes 48 16 $StarterVoltage # starter centivolts
        Set-Field $bytes 64 2 0 # auxiliary second-battery voltage
    }
    Set-Field $bytes 66 22 ($Current -band 0x3fffff) # signed milliamps
    Set-Field $bytes 88 20 $Consumed # magnitude of consumed Ah, tenths
    Set-Field $bytes 108 10 $Soc # tenths of a percent
    Set-Field $bytes 118 10 1023 # reserved bits must be set
    return ,$bytes
}
if ($CheckFixture) {
    if ($Model -eq 'bmv712') {
        $actual = New-OrionPayload 0x1234 (New-BatteryPlaintext 1276 -5250 450 775)
        $expectedHex = (Get-Content (Join-Path $PSScriptRoot '../bmv712/fixture.hex') -Raw).Trim().ToUpperInvariant()
    } else {
        $actual = New-OrionPayload 0x1234 ([byte[]](3,0,0x28,5,0xa0,5,0x81,0,0,0x80))
        $expected = [System.IO.File]::ReadAllBytes((Join-Path $PSScriptRoot '../../device-interface/tests/fixtures/victron/orion_tr.bin'))
        $expectedHex = [BitConverter]::ToString($expected).Replace('-', '')
    }
    if ([BitConverter]::ToString($actual).Replace('-', '') -ne $expectedHex) { throw 'Encoder differs from independent ciphertext fixture.' }
    if ($Model -eq 'bmv712') {
        $charging = New-OrionPayload 0x1234 (New-BatteryPlaintext 1400 12000 474 763)
        $chargingHex = (Get-Content (Join-Path $PSScriptRoot '../bmv712/charging-fixture.hex') -Raw).Trim().ToUpperInvariant()
        if ([BitConverter]::ToString($charging).Replace('-', '') -ne $chargingHex) { throw 'Charging encoder differs from independent fixture.' }
        $dual = New-OrionPayload 0x1234 (New-BatteryPlaintext 1276 -5250 450 775 1280)
        $dualBytes = [System.IO.File]::ReadAllBytes((Join-Path $PSScriptRoot '../../device-interface/tests/fixtures/victron/bmv_starter.bin'))
        if ([BitConverter]::ToString($dual) -ne [BitConverter]::ToString($dualBytes)) { throw 'Starter-voltage encoder differs from independent fixture.' }
    }
    Write-Output "PASS: $Model encoder matches independent ciphertext fixture."
    return
}
[void][Windows.Devices.Bluetooth.Advertisement.BluetoothLEAdvertisementPublisher,Windows.Devices.Bluetooth,ContentType=WindowsRuntime]
[void][Windows.Devices.Bluetooth.Advertisement.BluetoothLEManufacturerData,Windows.Devices.Bluetooth,ContentType=WindowsRuntime]
[void][Windows.Storage.Streams.DataWriter,Windows.Storage.Streams,ContentType=WindowsRuntime]
[void][Windows.Storage.Streams.IBuffer,Windows.Storage.Streams,ContentType=WindowsRuntime]
$publisher = $null
Write-Host "SIMULATED $Model : manufacturer ID 0x02E1, synthetic product 0x1234."
Write-Host 'Scan with an iPhone BLE scanner in the foreground. Device may be unnamed.'
Write-Host "Running for $DurationSeconds seconds. Ctrl-C stops. Public synthetic key: 000102030405060708090a0b0c0d0e0f"
try {
    for ($step = 0; $step -lt $DurationSeconds; $step++) {
        if ($null -ne $publisher) {
            $publisher.Stop()
            $deadline = [DateTime]::UtcNow.AddSeconds(5)
            while ($publisher.Status.ToString() -notin @('Stopped','Aborted')) {
                if ([DateTime]::UtcNow -gt $deadline) { throw 'Publisher did not stop.' }
                Start-Sleep -Milliseconds 50
            }
        }
        $inputVoltage = 1320 + ($step % 10)
        $plain = [byte[]](3,0,($inputVoltage -band 255),($inputVoltage -shr 8),0xa0,5,0,0,0,0)
        if ($Model -eq 'bmv712') {
            # Scripted UI demonstration, not an electrochemical battery model.
            $phase = $step % 120
            if ($phase -lt 60) {
                $voltage = 1276 - [int][Math]::Floor($phase / 10)
                $current = -5250
                $soc = 775 - [int][Math]::Floor($phase / 5)
            } else {
                $voltage = 1400 + [int][Math]::Floor(($phase - 60) / 10)
                $current = 12000
                $soc = 763 + [int][Math]::Floor(($phase - 60) / 5)
            }
            $consumed = (1000 - $soc) * 2 # illustrative 200 Ah bank
            $starterVoltage = if ($phase -lt 60) { 1280 - [int][Math]::Floor($phase / 15) } else { 1420 + [int][Math]::Floor(($phase - 60) / 15) }
            $plain = New-BatteryPlaintext $voltage $current $consumed $soc $starterVoltage
            Write-Host "SIMULATED house: $($voltage / 100.0) V, $($current / 1000.0) A, $($soc / 10.0)%; starter: $($starterVoltage / 100.0) V (current/SOC unavailable)"
        }
        $payload = New-OrionPayload ([uint16]($step + 1)) $plain
        $publisher = New-Object Windows.Devices.Bluetooth.Advertisement.BluetoothLEAdvertisementPublisher
        $data = New-Object Windows.Devices.Bluetooth.Advertisement.BluetoothLEManufacturerData
        $data.CompanyId = 0x02e1
        $writer = New-Object Windows.Storage.Streams.DataWriter
        try {
            $writer.WriteBytes($payload)
            $buffer = $writer.DetachBuffer()
            # Reflection bypasses PowerShell's COM-to-IBuffer argument converter.
            [Windows.Devices.Bluetooth.Advertisement.BluetoothLEManufacturerData].GetProperty('Data').SetValue($data, $buffer, $null)
        } finally { $writer.Dispose() }
        $listType = [System.Collections.Generic.ICollection[Windows.Devices.Bluetooth.Advertisement.BluetoothLEManufacturerData]]
        $arguments = New-Object object[] 1
        $arguments[0] = $data.PSObject.BaseObject
        $listType.GetMethod('Add').Invoke($publisher.Advertisement.ManufacturerData, $arguments)
        $publisher.Start()
        $deadline = [DateTime]::UtcNow.AddSeconds(10)
        while ($publisher.Status.ToString() -ne 'Started') {
            if ($publisher.Status.ToString() -eq 'Aborted') { throw 'Windows aborted advertising; adapter/driver may not support BLE broadcasting.' }
            if ([DateTime]::UtcNow -gt $deadline) { throw "Advertising timeout: $($publisher.Status)" }
            Start-Sleep -Milliseconds 100
        }
        Write-Host "Advertising counter=$($step + 1) status=$($publisher.Status) payload=$([BitConverter]::ToString($payload))"
        Start-Sleep -Seconds 1
    }
} finally {
    if ($null -ne $publisher) { $publisher.Stop() }
    Write-Host 'Broadcast stopped.'
}
