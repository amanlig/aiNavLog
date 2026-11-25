; Standard Windows delivery: a single per-user NSIS setup EXE.
Unicode true
RequestExecutionLevel user
SetCompressor /SOLID lzma
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
Name "aiNavLog"
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\aiNavLog"
BrandingText "aiNavLog"
VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey "LegalCopyright" ""
VIAddVersionKey "ProductName" "aiNavLog Setup"
VIAddVersionKey "FileDescription" "aiNavLog Windows installer"
VIAddVersionKey "FileVersion" "${APP_VERSION}"
VIAddVersionKey "ProductVersion" "${APP_VERSION}"
!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TEXT "This will install aiNavLog ${APP_VERSION} for your Windows account.$\r$\n$\r$\nClose aiNavLog before continuing. Your existing boat records and diagrams will be retained."
!define MUI_FINISHPAGE_TEXT "aiNavLog is installed.$\r$\n$\r$\nOpen aiNavLog from your desktop or Start menu when ready."
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Function .onInit
    ${IfNot} ${RunningX64}
        MessageBox MB_ICONSTOP "aiNavLog requires 64-bit Windows."
        Abort
    ${EndIf}
    SetShellVarContext current
    SetRegView 64
FunctionEnd

Section "aiNavLog" Main
    InitPluginsDir
    SetOutPath "$PLUGINSDIR"
    File /oname=preflight.ps1 "preflight.ps1"
    SetOutPath "$PLUGINSDIR\payload"
    File "${PAYLOAD_DIR}/aiNavLog.exe"
    File "${PAYLOAD_DIR}/NativeNotices.txt"
    File "${PAYLOAD_DIR}/README.txt"
    File "${PAYLOAD_DIR}/build-receipt.json"
    nsExec::ExecToStack /TIMEOUT=30000 '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\preflight.ps1"'
    Pop $0
    Pop $1
    DetailPrint "$1"
    ${If} $0 != "0"
        MessageBox MB_ICONSTOP "Setup could not continue:$\r$\n$1"
        Abort
    ${EndIf}
    SetOutPath "$INSTDIR"
    ClearErrors
    CopyFiles /SILENT "$PLUGINSDIR\payload\aiNavLog.exe" "$INSTDIR\aiNavLog.exe"
    CopyFiles /SILENT "$PLUGINSDIR\payload\NativeNotices.txt" "$INSTDIR\NativeNotices.txt"
    CopyFiles /SILENT "$PLUGINSDIR\payload\README.txt" "$INSTDIR\README.txt"
    CopyFiles /SILENT "$PLUGINSDIR\payload\build-receipt.json" "$INSTDIR\build-receipt.json"
    ${If} ${Errors}
        MessageBox MB_ICONSTOP "Could not copy app files. Close aiNavLog and run setup again."
        Abort
    ${EndIf}
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    CreateShortcut "$DESKTOP\aiNavLog.lnk" "$INSTDIR\aiNavLog.exe"
    CreateShortcut "$SMPROGRAMS\aiNavLog.lnk" "$INSTDIR\aiNavLog.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\aiNavLog" "DisplayName" "aiNavLog"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\aiNavLog" "DisplayVersion" "${APP_VERSION}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\aiNavLog" "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\aiNavLog" "DisplayIcon" "$INSTDIR\aiNavLog.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\aiNavLog" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\aiNavLog" "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\aiNavLog" "NoRepair" 1
SectionEnd

Section "Uninstall"
    SetShellVarContext current
    SetRegView 64
    InitPluginsDir
    SetOutPath "$PLUGINSDIR"
    File /oname=preflight.ps1 "preflight.ps1"
    nsExec::ExecToStack /TIMEOUT=30000 '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\preflight.ps1" -CheckOnly'
    Pop $0
    Pop $1
    ${If} $0 != "0"
        MessageBox MB_ICONSTOP "$1"
        Abort
    ${EndIf}
    Delete "$INSTDIR\aiNavLog.exe"
    Delete "$INSTDIR\NativeNotices.txt"
    Delete "$INSTDIR\README.txt"
    Delete "$INSTDIR\build-receipt.json"
    Delete "$INSTDIR\Uninstall.exe"
    Delete "$DESKTOP\aiNavLog.lnk"
    Delete "$SMPROGRAMS\aiNavLog.lnk"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\aiNavLog"
    ; Remove only an empty app directory. Never remove the user's data/profile.
    SetOutPath "$TEMP"
    RMDir "$INSTDIR"
SectionEnd
