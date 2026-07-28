# NSIS installer hooks for star (Windows only).
#
# By default an uninstaller leaves %APPDATA%\<identifier> in place, so history,
# profile and settings survive a reinstall. That is the right behaviour for an
# upgrade, but it also means a user who uninstalls to "start fresh" never sees
# onboarding again. This hook asks once, on uninstall, and clears the data only
# if the user says yes.

!macro NSIS_HOOK_POSTUNINSTALL
  ; Never prompt during an unattended/silent uninstall — it would hang the run.
  IfSilent skip_star_data_removal

  MessageBox MB_YESNO|MB_ICONQUESTION \
    "Do you also want to delete your star data?$\r$\n$\r$\nThis removes your profile, browsing history and settings. Choose No to keep them for a future reinstall." \
    IDNO skip_star_data_removal

  RMDir /r "$APPDATA\com.studio.star"

  skip_star_data_removal:
!macroend
