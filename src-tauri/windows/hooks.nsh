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
