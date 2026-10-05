!macro EXPENSES_REMOVE_OLD_INSTALL NAME
  ${If} ${FileExists} "$LOCALAPPDATA\${NAME}\uninstall.exe"
    StrCpy $R9 0
    ${Do}
      RMDir /r "$LOCALAPPDATA\${NAME}"
      ${IfNot} ${FileExists} "$LOCALAPPDATA\${NAME}\*.*"
        ${Break}
      ${EndIf}
      Sleep 500
      IntOp $R9 $R9 + 1
    ${LoopUntil} $R9 >= 20
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${NAME}"
    DeleteRegKey HKCU "Software\gxd3v\${NAME}"
    Delete "$SMPROGRAMS\${NAME}.lnk"
    Delete "$DESKTOP\${NAME}.lnk"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  SetShellVarContext current
  !insertmacro EXPENSES_REMOVE_OLD_INSTALL "Expenses Manager"
  !insertmacro EXPENSES_REMOVE_OLD_INSTALL "GX Expenses"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current
    StrCpy $R9 0
    expenses_retry_delete:
      ${If} ${FileExists} "$APPDATA\${BUNDLEID}\*.*"
      ${OrIf} ${FileExists} "$LOCALAPPDATA\${BUNDLEID}\*.*"
        ${If} $R9 < 20
          Sleep 500
          RmDir /r "$APPDATA\${BUNDLEID}"
          RmDir /r "$LOCALAPPDATA\${BUNDLEID}"
          IntOp $R9 $R9 + 1
          Goto expenses_retry_delete
        ${EndIf}
        IfSilent +2
          MessageBox MB_ICONEXCLAMATION|MB_OK "Não foi possível apagar todos os dados (ficheiros em uso). Pastas a remover manualmente:$\r$\n$APPDATA\${BUNDLEID}$\r$\n$LOCALAPPDATA\${BUNDLEID}"
      ${EndIf}
  ${EndIf}
!macroend
