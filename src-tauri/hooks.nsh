!macro NSIS_HOOK_POSTINSTALL
  CreateShortcut "$DESKTOP\Swift Speak.lnk" "$INSTDIR\Swift Speak.exe"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$DESKTOP\Swift Speak.lnk"
!macroend
