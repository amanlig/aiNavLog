# Run on Windows after Android Studio installation completes.
# Uses Google's SDK tools and preserves any existing aiNavLog virtual device.
[CmdletBinding()]
param([string]$DeviceName = 'aiNavLog_Pixel')
$ErrorActionPreference = 'Stop'
$androidSdkPath = Join-Path $env:LOCALAPPDATA 'Android\Sdk'
$studioJavaPath = Join-Path $env:ProgramFiles 'Android\Android Studio\jbr'
$sdkManagerPath = Join-Path $androidSdkPath 'cmdline-tools\latest\bin\android.exe'
$avdManagerPath = Join-Path $androidSdkPath 'cmdline-tools\latest\bin\avdmanager.bat'
if (-not (Test-Path (Join-Path $studioJavaPath 'bin\java.exe'))) {
    throw 'Finish the Android Studio installation first (approve the Windows administrator prompt).'
}
if (-not (Test-Path $sdkManagerPath)) {
    throw 'Install Android SDK Command-line Tools (latest) through Android Studio SDK Manager first.'
}
# Process-local Java configuration; does not change persistent Windows settings.
$previousJavaPath = $env:JAVA_HOME
try {
    $env:JAVA_HOME = $studioJavaPath
    $imagePackage = 'system-images;android-36;google_apis_playstore;x86_64'
    Write-Host 'Installing SDK tools and a Google Play Android 16 image. Review/accept SDK licenses when prompted.'
    & $sdkManagerPath --no-metrics "--sdk=$androidSdkPath" sdk install 'platform-tools' 'emulator' ($imagePackage -replace ';', '/')
    if ($LASTEXITCODE -ne 0) { throw 'SDK installation did not complete.' }
    $emulatorPath = Join-Path $androidSdkPath 'emulator\emulator.exe'
    & $emulatorPath -accel-check
    if ($LASTEXITCODE -ne 0) {
        throw 'Hardware acceleration is unavailable. Enable Windows Hypervisor Platform in Windows Features, restart when convenient, then rerun this script.'
    }
    $existingDevices = & $emulatorPath -list-avds
    if ($existingDevices -notcontains $DeviceName) {
        $avdClassPath = Join-Path $androidSdkPath 'cmdline-tools\latest\lib\avdmanager-classpath.jar'
        'no' | & (Join-Path $studioJavaPath 'bin\java.exe') "-Dcom.android.sdkmanager.toolsdir=$androidSdkPath\cmdline-tools\latest" -classpath $avdClassPath com.android.sdklib.tool.AvdManagerCli create avd --name $DeviceName --package $imagePackage --device 'pixel_7'
        if ($LASTEXITCODE -ne 0) { throw 'Virtual device creation failed; inspect avdmanager list device for available profiles.' }
    }
    Start-Process -FilePath $emulatorPath -ArgumentList @('-avd', $DeviceName)
    Write-Host 'Emulator launched. Sign into Google Play with an eligible tester account and open the internal-test opt-in link.'
} finally {
    $env:JAVA_HOME = $previousJavaPath
}
