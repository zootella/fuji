; Hooks into the NSIS installer Tauri generates, named in bundle.windows.nsis.installerHooks in tauri.conf.json. Tauri includes this file at the top of its installer script and inserts each NSIS_HOOK_ macro defined here at a fixed point in its install and uninstall sections.

; Uninstalling takes back what associate.js told Windows fuji can open. Those are writes the running program made, not the installer, so a bare NSIS uninstall knows nothing of them and would leave fuji offered in Open with and listed in Settings, through a command naming a program that is gone. Everything is under HKEY_CURRENT_USER, like the install itself, and every name comes from the product name associate.js reads too.
; It holds no list of extensions. associate.js writes an entry under Capabilities\FileAssociations for every extension it offers, the extension named and fuji's ProgID for it as the value, so this reads that list back and takes each one in turn; fuji will offer dozens of kinds of file, and this never has to learn them, and an extension a later fuji stopped offering is still taken back.
; This runs after the uninstall rather than before it because the section's first real step asks to close a running fuji, and a user who says no there stops the uninstall; the pre hook would already have taken the keys, leaving a working install that opens nothing. Here the files are gone and $INSTDIR still names the folder they were in.
; Removing fuji in Settings runs this, and so does running a newer installer by hand, as of Tauri 2.11.4: its reinstall page selects uninstalling first and runs the old uninstall.exe without /UPDATE, so the registrations go here and come back on the new copy's first launch, which the finish page offers. The user's own saved choice is not among them: it is sealed, only Windows' screens change it, and this leaves it naming fuji's ProgIDs, which point nowhere until a copy is installed again and then apply as before, so an upgrade keeps a choice of fuji. It is the old copy's uninstaller that runs, so a change to this file reaches upgrades one version later. A reinstall at the same version runs no uninstaller at all, since the page then offers Add/Reinstall components instead, so while the version stays 0.1.0 a build installed over the top leaves behind whatever a type since switched off had registered, seen on the Windows box 2026-09-29.
; The installer's own "delete the application data" checkbox is a separate thing from this hook. It removes %APPDATA%\<identifier> and %LOCALAPPDATA%\<identifier>, which is the WebView2 cache, up to a hundred megabytes, and never fuji.toml or the fuji-temp logs, which sit in the home folder; that is deliberate, since a settings file a user edited is not the installer's to destroy, and it is how a reinstall remembers the three answer lists.
!macro NSIS_HOOK_POSTUNINSTALL
	${If} $UpdateMode <> 1 ; an update keeps its registrations, the same guard Tauri's template puts on the shortcuts and the Run value
		DeleteRegValue HKCU "Software\RegisteredApplications" "${PRODUCTNAME}" ; first, the reverse of associate.js's order, so Settings stops listing fuji before what it points at goes

		StrCpy $R1 0 ; which value of FileAssociations comes next
		${Do}
			EnumRegValue $R2 HKCU "Software\${PRODUCTNAME}\Capabilities\FileAssociations" $R1 ; an extension, like .webp, or blank past the last one
			${If} $R2 == ""
				${ExitDo}
			${EndIf}
			ReadRegStr $R3 HKCU "Software\${PRODUCTNAME}\Capabilities\FileAssociations" $R2 ; fuji's ProgID for it, like Fuji.webp
			${If} $R3 == "${PRODUCTNAME}$R2" ; only the ProgID shape associate.js makes, and above all never a blank one, which would turn the delete below into all of Software\Classes
				DeleteRegKey HKCU "Software\Classes\$R3" ; fuji's own ProgID, which no other program writes, taken whole
				DeleteRegValue HKCU "Software\Classes\$R2\OpenWithProgids" $R3 ; the offer, one value in a list other programs share
				ReadRegStr $R4 HKCU "Software\Classes\$R2" "" ; the extension's own default value, the fallback associate.js writes when the user says yes
				${If} $R4 == $R3 ; so it goes only while it still names fuji; another program's stays
					DeleteRegValue HKCU "Software\Classes\$R2" ""
				${EndIf}
				DeleteRegKey /ifempty HKCU "Software\Classes\$R2\OpenWithProgids" ; the list and the extension's key go only if now empty, and /ifempty counts every subkey and value, so another program's registration stays
				DeleteRegKey /ifempty HKCU "Software\Classes\$R2"
			${EndIf}
			IntOp $R1 $R1 + 1 ; nothing is deleted from FileAssociations itself inside the loop, so the numbering holds still under it
		${Loop}

		DeleteRegKey HKCU "Software\${PRODUCTNAME}\Capabilities" ; only now, since it was the list the loop read
		DeleteRegKey /ifempty HKCU "Software\${PRODUCTNAME}" ; Tauri keeps the install folder in a key beside Capabilities, so the parent goes only once nothing is left in it
		DeleteRegKey HKCU "Software\Classes\Applications\${MAINBINARYNAME}.exe" ; the executable's own key, with the types it supports

		System::Call "shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)" ; SHCNE_ASSOCCHANGED, the same call registry.rs makes, so Explorer drops the icons and menus it cached
	${EndIf}
!macroend
