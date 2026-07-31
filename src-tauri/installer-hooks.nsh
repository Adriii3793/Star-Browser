# NSIS installer hooks for star (Windows only).
#
# By default an uninstaller leaves %APPDATA%\<identifier> and
# %LOCALAPPDATA%\<identifier> in place, so history, profile and settings
# survive a reinstall. That is the right behaviour for an upgrade, but it
# also means a user who uninstalls to "start fresh" never sees onboarding
# again. This hook asks once, on uninstall, and clears the data only if the
# user says yes.

!macro NSIS_HOOK_POSTUNINSTALL
  ; Never prompt during an unattended/silent uninstall -- it would hang the run.
  IfSilent skip_star_data_removal

  MessageBox MB_YESNO|MB_ICONQUESTION \
    "Do you also want to delete your star data?$\r$\n$\r$\nThis removes your profile, browsing history and settings. Choose No to keep them for a future reinstall." \
    IDNO skip_star_data_removal

  ; WebView2 (msedgewebview2.exe) can still be flushing its profile in
  ; %LOCALAPPDATA%\<id>\EBWebView for a few seconds after the app exits,
  ; which makes RMDir fail silently and leaves the data behind. Retry a few
  ; times with a pause so the engine has time to release its locks.
  StrCpy $R9 0
  star_data_removal_try:
  RMDir /r "$APPDATA\com.studio.star"
  RMDir /r "$LOCALAPPDATA\com.studio.star"
  IfFileExists "$LOCALAPPDATA\com.studio.star" star_data_still_there
  IfFileExists "$APPDATA\com.studio.star" star_data_still_there skip_star_data_removal

  star_data_still_there:
  IntOp $R9 $R9 + 1
  IntCmp $R9 6 skip_star_data_removal 0 skip_star_data_removal
  Sleep 1500
  Goto star_data_removal_try

  skip_star_data_removal:
!macroend
