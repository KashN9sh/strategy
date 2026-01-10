; NSIS Installer Script for Cozy Kingdom
; Compile with: makensis installer.nsi (requires NSIS installed on Windows)

!define APP_NAME "Cozy Kingdom"
!define APP_VERSION "0.1.0"
!define APP_PUBLISHER "Korolev Roman"
!define APP_EXE "Cozy Kingdom.exe"
!define APP_DIR "Cozy Kingdom"

; Modern UI
!include "MUI2.nsh"

; Installer settings
Name "${APP_NAME}"
OutFile "Cozy Kingdom Setup.exe"
InstallDir "$PROGRAMFILES\${APP_DIR}"
RequestExecutionLevel admin

; Interface settings
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "${NSISDIR}\Contrib\Graphics\Header\nsis3-grey.bmp"
!define MUI_WELCOMEFINISHPAGE_BITMAP "${NSISDIR}\Contrib\Graphics\Wizard\nsis3-grey.bmp"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "Cozy Kingdom Portable\LICENSE"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Languages
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Russian"

; Installer sections
Section "Game Files" SecGame
    SectionIn RO ; Read-only, always installed
    
    SetOutPath "$INSTDIR"
    
    ; Copy executable
    File "Cozy Kingdom Portable\${APP_EXE}"
    
    ; Copy resources
    SetOutPath "$INSTDIR\assets"
    File /r /x "*.aseprite" "Cozy Kingdom Portable\assets\*.*"
    
    SetOutPath "$INSTDIR\shaders"
    File /r "Cozy Kingdom Portable\shaders\*.*"
    
    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    
    ; Registry entries
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_DIR}" \
        "DisplayName" "${APP_NAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_DIR}" \
        "DisplayVersion" "${APP_VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_DIR}" \
        "Publisher" "${APP_PUBLISHER}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_DIR}" \
        "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_DIR}" \
        "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_DIR}" \
        "NoRepair" 1
    
    ; Start menu shortcuts
    CreateDirectory "$SMPROGRAMS\${APP_DIR}"
    CreateShortcut "$SMPROGRAMS\${APP_DIR}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
    CreateShortcut "$SMPROGRAMS\${APP_DIR}\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
    
    ; Desktop shortcut (optional)
    ; CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
SectionEnd

Section "Start Menu Shortcut" SecShortcut
    CreateDirectory "$SMPROGRAMS\${APP_DIR}"
    CreateShortcut "$SMPROGRAMS\${APP_DIR}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
SectionEnd

Section "Desktop Shortcut" SecDesktop
    CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
SectionEnd

; Uninstaller
Section "Uninstall"
    ; Remove files
    Delete "$INSTDIR\${APP_EXE}"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir /r "$INSTDIR\assets"
    RMDir /r "$INSTDIR\shaders"
    RMDir "$INSTDIR"
    
    ; Remove shortcuts
    RMDir /r "$SMPROGRAMS\${APP_DIR}"
    Delete "$DESKTOP\${APP_NAME}.lnk"
    
    ; Remove registry entries
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_DIR}"
    
    ; Remove user data (optional - ask user?)
    ; RMDir /r "$APPDATA\${APP_DIR}"
SectionEnd

; Section descriptions
LangString DESC_SecGame ${LANG_ENGLISH} "Core game files (required)"
LangString DESC_SecShortcut ${LANG_ENGLISH} "Create Start Menu shortcut"
LangString DESC_SecDesktop ${LANG_ENGLISH} "Create Desktop shortcut"

LangString DESC_SecGame ${LANG_RUSSIAN} "Основные файлы игры (обязательно)"
LangString DESC_SecShortcut ${LANG_RUSSIAN} "Создать ярлык в меню Пуск"
LangString DESC_SecDesktop ${LANG_RUSSIAN} "Создать ярлык на рабочем столе"

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SecGame} $(DESC_SecGame)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecShortcut} $(DESC_SecShortcut)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} $(DESC_SecDesktop)
!insertmacro MUI_FUNCTION_DESCRIPTION_END
