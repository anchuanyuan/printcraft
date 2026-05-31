; PrintCraft NSIS 安装器脚本
; 构建: makensis installers/windows/installer.nsi

!include "MUI2.nsh"

; 基本信息
Name "PrintCraft"
OutFile "printcraft-setup.exe"
InstallDir "$LOCALAPPDATA\PrintCraft"
InstallDirRegKey HKCU "Software\PrintCraft" "InstallDir"
RequestExecutionLevel user

; 界面
!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "SimpChinese"

Section "PrintCraft 核心文件" SecCore
  SetOutPath "$INSTDIR"

  ; 主文件
  File "target\release\printcraft-server.exe"
  File "target\release\printcraft.exe"
  File /oname=sdk\printcraft.js "..\sdk\dist\printcraft.js"

  ; 创建卸载器
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; 写入注册表
  WriteRegStr HKCU "Software\PrintCraft" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\PrintCraft" \
    "DisplayName" "PrintCraft"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\PrintCraft" \
    "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\PrintCraft" \
    "Publisher" "PrintCraft"

  ; 创建开始菜单
  CreateDirectory "$SMPROGRAMS\PrintCraft"
  CreateShortCut "$SMPROGRAMS\PrintCraft\PrintCraft 服务.lnk" "$INSTDIR\printcraft-server.exe"
  CreateShortCut "$SMPROGRAMS\PrintCraft\卸载 PrintCraft.lnk" "$INSTDIR\uninstall.exe"

  ; 开机自启
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" \
    "PrintCraft" "$\"$INSTDIR\printcraft-server.exe$\""
SectionEnd

Section "Uninstall"
  ; 停止服务
  ExecWait 'taskkill /f /im printcraft-server.exe'

  ; 删除文件
  Delete "$INSTDIR\printcraft-server.exe"
  Delete "$INSTDIR\printcraft.exe"
  Delete "$INSTDIR\sdk\printcraft.js"
  RMDir "$INSTDIR\sdk"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  ; 删除快捷方式
  Delete "$SMPROGRAMS\PrintCraft\PrintCraft 服务.lnk"
  Delete "$SMPROGRAMS\PrintCraft\卸载 PrintCraft.lnk"
  RMDir "$SMPROGRAMS\PrintCraft"

  ; 删除注册表
  DeleteRegKey HKCU "Software\PrintCraft"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\PrintCraft"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "PrintCraft"
SectionEnd
