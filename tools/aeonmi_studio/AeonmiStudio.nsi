; ============================================================
;  Aeonmi Studio v2.0  —  NSIS Installer Script
;  AI-Native Programming Language IDE
; ============================================================

Unicode True

!define APP_NAME      "Aeonmi Studio"
!define APP_VERSION   "2.0"
!define APP_PUBLISHER "Aeonmi Inc"
!define APP_URL       "https://github.com/Aeonmi-Aysa/aeonmi"
!define INSTALL_DIR   "$PROGRAMFILES64\Aeonmi"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\AeonmiStudio"

Name         "${APP_NAME} ${APP_VERSION}"
OutFile      "C:\Temp\AeonmiStudio_Setup.exe"
InstallDir   "${INSTALL_DIR}"
InstallDirRegKey HKLM "Software\Aeonmi" "InstallPath"
RequestExecutionLevel admin
SetCompressor /SOLID lzma
BrandingText "${APP_PUBLISHER}  |  AI-Native Programming Language"

; ── Pages ─────────────────────────────────────────────────────────────────────
!include "MUI2.nsh"

!define MUI_ABORTWARNING
!define MUI_ICON   "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"

; Header/banner colors
!define MUI_HEADERIMAGE
!define MUI_BGCOLOR "060C10"
!define MUI_HEADER_TRANSPARENT_TEXT

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE  "C:\Temp\AeonmiLicense.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; ── Welcome page text ─────────────────────────────────────────────────────────
!define MUI_WELCOMEPAGE_TITLE   "Welcome to Aeonmi Studio v2.0"
!define MUI_WELCOMEPAGE_TEXT    "Aeonmi is an AI-native programming language ecosystem.$\r$\n$\r$\nThis will install Aeonmi Studio — a full IDE with:$\r$\n$\r$\n  • Syntax-highlighted code editor$\r$\n  • Live VM pipeline visualization$\r$\n  • 7 built-in example programs$\r$\n  • 80+ native builtins (math, strings, collections, JSON)$\r$\n  • Multi-agent swarm support$\r$\n$\r$\nClick Next to continue."

!define MUI_FINISHPAGE_TITLE    "Aeonmi Studio Installed"
!define MUI_FINISHPAGE_TEXT     "Aeonmi Studio v2.0 has been installed.$\r$\n$\r$\nClick Finish to launch the IDE."
!define MUI_FINISHPAGE_RUN      "$INSTDIR\AeonmiStudio.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch Aeonmi Studio now"
!define MUI_FINISHPAGE_LINK     "Visit Aeonmi on GitHub"
!define MUI_FINISHPAGE_LINK_LOCATION "${APP_URL}"

; ── Install section ───────────────────────────────────────────────────────────
Section "Aeonmi Studio (required)" SecMain
    SectionIn RO
    SetOutPath "$INSTDIR"

    ; Copy main files
    File "C:\Temp\aeonmi_installer_stage\AeonmiStudio.exe"
    File "C:\Temp\aeonmi_installer_stage\Aeonmi.exe"

    ; Write registry for uninstall
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "DisplayName"          "${APP_NAME} ${APP_VERSION}"
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "UninstallString"       "$INSTDIR\Uninstall.exe"
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "InstallLocation"       "$INSTDIR"
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "Publisher"             "${APP_PUBLISHER}"
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "URLInfoAbout"          "${APP_URL}"
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "DisplayVersion"        "${APP_VERSION}"
    WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoModify"              1
    WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoRepair"              1
    WriteRegStr   HKLM "Software\Aeonmi"  "InstallPath"           "$INSTDIR"

    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Start Menu shortcuts
    CreateDirectory "$SMPROGRAMS\Aeonmi"
    CreateShortcut  "$SMPROGRAMS\Aeonmi\Aeonmi Studio.lnk"  "$INSTDIR\AeonmiStudio.exe"
    CreateShortcut  "$SMPROGRAMS\Aeonmi\Uninstall.lnk"      "$INSTDIR\Uninstall.exe"

    ; Desktop shortcut
    CreateShortcut  "$DESKTOP\Aeonmi Studio.lnk" "$INSTDIR\AeonmiStudio.exe"
SectionEnd

; ── Uninstall section ─────────────────────────────────────────────────────────
Section "Uninstall"
    Delete "$INSTDIR\AeonmiStudio.exe"
    Delete "$INSTDIR\Aeonmi.exe"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir  "$INSTDIR"

    Delete "$SMPROGRAMS\Aeonmi\Aeonmi Studio.lnk"
    Delete "$SMPROGRAMS\Aeonmi\Uninstall.lnk"
    RMDir  "$SMPROGRAMS\Aeonmi"
    Delete "$DESKTOP\Aeonmi Studio.lnk"

    DeleteRegKey HKLM "${UNINSTALL_KEY}"
    DeleteRegKey HKLM "Software\Aeonmi"
SectionEnd
