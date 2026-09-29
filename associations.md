# File associations

Fuji being an application the operating system will hand a picture to. The first pass is built and verified on both platforms: `Info.plist` declares fuji's types on the Mac, and on Windows `associate.js` registers them, over the general commands in `registry.rs`, with the reasoning in its essay. This document is now the plan for the second pass, decided on 2026-09-28 by looking over the fence at ftorrent, which built the same thing for its two contested types first.

**Fuji offers and never takes, and the user decides.** It appears wherever the system asks what could open a file, it becomes a type's default only when the user says yes to that type, and a choice the user makes in the system's own screens is followed rather than argued with. The essay at the top of `associate.js` carries the values — simple, modern, polite, assertive, the user in control — and how the design follows from them.

## The concession the second pass answers

**The first pass registers correctly and nobody can reasonably find how to use it.** On macOS, making fuji the default means Get Info, a collapsed *Open with:* section, and a small *Change All…* button, while the Open With submenu offers no way to set a default at all. On Windows the working route is Open with → Choose another app, which raises a dialog matching nothing else in the system. The settings section below is the fix, and it is the only interface fuji will have for this: **no banner**, and no question asked at launch or anywhere else.

## The design

**One answer per extension, kept as three lists.** `[associations]` in `fuji.toml` holds `yes`, `no` and `ask`, each a list of extensions written lowercase with the dot, in `imageTypes` order. Three lines however many extensions fuji learns. Every extension fuji opens is in exactly one list: one missing from all three is ask, one in two lists is ask and is reported, and one fuji does not open is reported and dropped. Ask is the factory for everything, so an extension a later fuji adds — video and sound will add dozens — starts undecided and never inherits a yes given to something else. Ask means undecided; nothing asks, and the word is kept short and ready in case something someday does.

**What each answer does:**

| | Windows | Mac |
| --- | --- | --- |
| **yes** | fuji writes the extension's fallback, its own default value naming fuji's ProgID, and rewrites it at every launch; where Windows has saved another program, the section says so and offers Windows Settings | fuji sets itself as the default for the type, once, at the moment of the yes, and never again |
| **no** | fuji takes the fallback back, only while it still names fuji | fuji sets nothing; where the Mac still opens it with fuji, the section says so and points to Get Info |
| **ask** | as no | as no |

**Every answer leaves fuji in Open with.** No means "not my default", and being offered costs the user nothing.

**Fuji only ever takes back what it wrote, and never picks another program for the user.** On Windows that is the fallback, beneath the user's saved choice. On the Mac the default fuji sets *is* the saved choice, so a no after a yes cannot be undone without choosing someone else, and fuji does not choose. That is also why the Mac's yes happens once: repeating it at every launch would take the type back from whatever the user picked in Get Info since, which is the old habit the essay describes. Rewriting the Windows fallback at every launch is harmless, because the user's saved choice always sits above it.

**Fuji follows the user's choices made in the system.** When fuji finds the system already opening an ask extension with this copy of fuji, the answer becomes yes: a user who installs fuji, sets defaults in Windows Settings or Get Info, and then opens fuji's settings finds those answers recorded. Following only ever turns ask into yes. A no is never changed, and a yes the system does not share stays a yes and shows as a disagreement, so fuji's answers never change under the user because another program took a type.

**Fuji asks the system only while the settings section is showing.** At launch it registers, and writes the fallback for each yes, which needs no lookup. Asking who opens each extension happens when the section appears, again whenever the window regains focus, which is how fuji notices the user coming back from Windows Settings, and after every change. A user who never opens settings never pays for it.

**On the Mac, extensions that share a type move together.** The Mac sets a default per type rather than per extension, and `.jpg`, `.jpeg` and `.jpe` are all `public.jpeg`, so a yes on one is a yes on all three; `.jfif` gets a type of its own (see below). Windows sets each extension independently.

## Measured on the Windows box, 2026-09-28

**What Windows answers when asked who opens a type.** `AssocQueryString`, the lookup Explorer and Settings use, following the user's saved choice first and the fallbacks after:

- A desktop program answers a ProgID, a display name, and an executable: `.bmp` gave `Paint.Picture`, Paint, and `mspaint.exe`.
- **A Store app answers a ProgID and a name and no executable.** `.gif` gave `AppX43hn…`, Photos, and error `0x80070483`, no association, for the executable. Photos is the usual incumbent for pictures, so a lookup that treated a missing executable as nothing, as ftorrent's does, would report Photos as nothing.
- An extension nothing is registered for answers `Unknown`, "Pick an app", and `OpenWith.exe` — the catch-all class and its chooser — unless the lookup is given `ASSOCF_INIT_IGNOREUNKNOWN`, and then it answers no association for all three while every other answer stays the same.

So `registry_opens` passes that flag and answers all three as the shell gives them, blank where it gives nothing, and the page decides: fuji opens it when the executable is this copy's, nothing opens it when the name is blank, and the name is what the section shows.

**Settled by ftorrent on the same box, and true here since it is the same Windows and the same calls.** `ms-settings:defaultapps?registeredAppUser=Fuji` opens fuji's own page on Windows 11 with the April 2023 update; Windows 10 does not know the parameter and opens Default apps, where *Set defaults by app* lists fuji's types. One link for both, and no version check. And a user's saved choice of the application survives an uninstall and reinstall: the uninstaller leaves `UserChoice` naming the ProgIDs, which apply again once a copy is installed.

## The uninstall hook

**Uninstalling takes back everything the running program wrote**, in an `NSIS_HOOK_POSTUNINSTALL` macro in `src-tauri/windows/hooks.nsh`, named by `bundle.windows.nsis.installerHooks`. Measured before it existed: an uninstall took the install folder, the shortcuts, the uninstall key and the WebView2 cache, and left all ten `Fuji.*` ProgIDs, the `Capabilities` block, the `RegisteredApplications` value and all ten `OpenWithProgids` entries, pointing at an executable that was gone.

- **It reads what to remove from the registry rather than holding a list.** `Capabilities\FileAssociations` maps each extension to its ProgID, and fuji writes an entry there for every extension it offers, so the hook enumerates it. Fuji's list will grow by dozens and the hook never learns it, and an extension a later version stopped offering is still taken back.
- **After the uninstall, not before.** The section's first step asks to close a running fuji, and a user who says no stops the uninstall there; the pre hook would already have taken the keys, leaving a working install that opens nothing.
- **A fallback goes only while it still names fuji**, and a key goes only when that leaves it empty, so another program's registration keeps its place.
- **It skips an update, and runs on an upgrade by hand.** As of Tauri 2.11.4 the reinstall page selects uninstalling first and runs the old `uninstall.exe` without `/UPDATE`, so the registrations go and come back on the new copy's first launch, which the finish page offers; the user's saved choice survives that, as above. It is the old copy's uninstaller that runs, so a change to the hook reaches upgrades one version late.

## What the user meets, walked through rather than predicted

**macOS.** Drag `Fuji.app` to Applications and nothing appears, because macOS never announces a new handler. From that moment fuji is in Finder's Open With for its ten types, before it has ever run. Double-click still opens Preview. A user who wants fuji selects a file, ⌘I, *Open with* → Fuji, *Change All…*, and confirms the sheet.

**Windows, on the Windows 10 box 2026-09-13.** The installer writes nothing about file types. Launching the installed copy once registers it; only the copy the installer recorded registers at all, wherever the user chose to put it. Then, in the order a user meets them: a **double-click** raises Windows' chooser with fuji listed and the incumbent pre-selected — one shot, and not taking it cannot be recovered; **Open with → Fuji** opens the picture but does *not* change the default, which is right and which users find confusing; **Open with → Choose another app**, tick *Always use this app*, is the route that works.

**What changes in Explorer once fuji opens a type is more visible than expected.** The Type column shows fuji's own name for the type, and every file of it wears `document-image.ico`, which ships beside the executable and which each ProgID's `DefaultIcon` names — most visible in Details view and with thumbnails turned off. `icon.md` owns the artwork.

## Where the work stands

Built on the Windows box, 2026-09-28, and waiting for the user's smoke test of an installed copy:

- `settings.js` reads and writes lists, and `[associations]` holds the three.
- `registry.rs` gains `registry_delete` and `registry_opens`.
- `associate.js` holds the answers, writes and takes back the fallbacks, looks up who opens each extension, and follows the user's choices.
- The settings panel has a section listing every extension with its answer and what opens it now. **It is deliberately plain**: the user has an idea for this interface, and the plain version exists so the whole stack under it can be exercised first.
- The uninstall hook.

**Still to build, on the Mac, and only the Mac can.** Two commands: which application opens a type, and making this application the default for one — `NSWorkspace.urlForApplication(toOpen:)` and `setDefaultApplication(at:toOpen:completionHandler:)`, whose content-type forms arrived in macOS 12 and are believed to answer the old worry that the reading direction needed a file rather than a type; that is to confirm there. With them, a way to name the type an extension belongs to, so siblings move together. `win2mac.md` carries this to the Mac. Until then the section shows the Mac's answers and cannot act on them.

**Linux is left out for now.** Its packages declare no `MimeType`, so fuji cannot be handed a picture there; `fuji.desktop.hbs` is where that starts, as a static list the way `Info.plist` is one.

## Settled, and recorded so nobody researches it twice

**The installed-copy gate works on Windows**, checked on the Windows box on 2026-09-28. `associate.js` registers only when the running executable's folder equals `InstallLocation` under `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Uninstall\Fuji`, which Tauri's NSIS template writes on every install, in quotes, into whatever folder the user chose on its folder page, and which the uninstaller removes. The installed copy passed the gate and logged `associate: 10 types registered, 0 values written` on every launch, and copies run from `target/release` and `target/debug` registered nothing. The template also keeps the folder as the default value of `Software\<manufacturer>\Fuji`, and that one is the wrong thing to test: the uninstaller deletes it only when *delete the application data* is ticked.

**The uninstaller's *delete the application data* checkbox reaches the WebView2 cache and not fuji's own state.** It removes `%APPDATA%\<identifier>` and `%LOCALAPPDATA%\<identifier>`, where the web engine keeps a cache that runs to a hundred megabytes. `fuji.toml` and the `fuji-temp` logs sit in the home folder and survive it — deliberate, since a settings file a user edited is not obviously the installer's to destroy — and that includes the three answer lists, so a reinstall remembers them.

**macOS needs no document icon, so the one fuji has is Windows only.** Checked on the Mac mini 2026-09-14 with `.jpg` and `.webp` set to fuji: Finder still draws every file as a picture of itself, because Quick Look previews win over a handler's document icon.

**Explorer keeps drawing thumbnails for a type fuji owns, and it cannot be otherwise.** Fuji writes no `ShellEx` key anywhere, so it never registers as a thumbnail provider and has nothing to displace; and the thumbnail lookup does not follow `UserChoice` — it runs through the extension's own `ShellEx` or through `SystemFileAssociations\image` by `PerceivedType`, and fuji touches none of those.

**The first pass took nothing from anyone.** Before and after on the same machine: all ten ProgIDs and the `Capabilities` block appeared, fuji was appended to each extension's `OpenWithProgids` beside what was there, and no extension's default value was written and no `UserChoice` moved.

**Declaring by extension is enough on macOS.** It synthesised the right system UTIs — `public.jpeg`, `public.svg-image` — from bare `CFBundleTypeExtensions`, with no `LSItemContentTypes` anywhere. `.jfif` is the exception and gets a dynamic UTI, so *Change All* on a `.jpg` does not cover it.

**Two bundles claim these types on a development machine** and share a bundle identifier — the copy in `/Applications` and the one under `target/release/bundle/macos/`. LaunchServices stores the choice by identifier rather than path, so it may launch either. Usually the installed one wins.
