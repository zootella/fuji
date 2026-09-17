# File associations

Fuji being an application the operating system will hand a picture to. The first pass is built and verified on both platforms; `associate.rs` carries what fuji writes on Windows and why, `Info.plist` carries the macOS declaration, and `open.rs` carries how a picture reaches a window. What is left here is the part a user meets, and the second pass.

**Fuji offers and never takes.** It appears wherever the system asks what could open a file, and it never makes itself the default — that is the user's to do, once, in the operating system's own interface. Every decision below is a consequence.

## The concession, which a later reader would otherwise miss

**Fuji registers correctly and a person cannot reasonably be expected to find how to use it.** On macOS, making fuji the default means Get Info, then expanding a collapsed *Open with:* section, then a small button reading *Change All…* — while the Open With submenu, the obvious place, offers no way to set a default at all. The route took several tries to find for the person who had just written the plan. On Windows the working route is Open with → Choose another app, which raises a dialog matching nothing else in the system.

So the honest description of the first pass is that it works and nobody will find it, which is tolerable only while the only user is the one who built it. **The File Extensions page in the second pass is what fixes this**, and that is its reason.

**Fuji never checks whether it is the default** — not at startup, not on a launch with a file, not anywhere. That restraint is what keeps fuji from ever growing the banner that started this subject, and it has to survive into the second pass: query only when the user is looking at the answer.

## What the user meets, walked through rather than predicted

**macOS.** Drag `Fuji.app` to Applications and nothing appears, because macOS never announces a new handler. From that moment fuji is in Finder's Open With for its ten types, before it has ever run. Double-click still opens Preview. A user who wants fuji selects a file, ⌘I, *Open with* → Fuji, *Change All…*, and confirms the sheet.

**Windows, on the Windows 10 box 2026-09-13.** The installer writes nothing about file types. Launching fuji once registers it, and because the command paths come from `current_exe()` a registration written by a different copy repoints itself. Then, in the order a user meets them: a **double-click** raises Windows' chooser with fuji listed and the incumbent pre-selected — one shot, and not taking it cannot be recovered; **Open with → Fuji** opens the picture but does *not* change the default, which is right and which users find confusing; **Open with → Choose another app**, tick *Always use this app*, is the route that works.

**What changes in Explorer afterwards is more visible than expected.** The Type column shows fuji's own name for the type, and every file of it wears whatever `DefaultIcon` names. That was the application's own icon until 2026-09-17, which meant a folder of pictures drawn as a folder of identical mint discs, most visibly in Details view and with thumbnails turned off. It is now `document-image.ico`, shipped beside the executable; `icon.md` owns the artwork.

## Still to build

**The uninstall hook, Windows only.** Fuji writes its keys at runtime and the uninstaller knows nothing about them, so uninstalling today leaves fuji in Open with lists pointing at an executable that is gone. `NSIS_HOOK_PREUNINSTALL` deletes the ProgIDs, the `Applications\fuji.exe` key, the `Capabilities` block and `RegisteredApplications` value, and the `OpenWithProgids` values fuji wrote — only keys fuji created. No install hook. macOS needs nothing, because trashing the `.app` takes its claims with it.

**A File Extensions page in fuji's settings**, when fuji has settings, because a user who wants fuji to open JPEGs will reasonably start in fuji. One table, both platforms, a row per extension: the extension, what opens it now, and an action. The action is the only thing that differs and its label says which it is — *Use Fuji* on macOS calls `NSWorkspace.setDefaultApplication`, which is supported, silent and per type; *Choose in Settings…* on Windows opens `ms-settings:defaultapps?registeredAppUser=Fuji`, because Windows lets no application set a default at all. Reading the current handler is supported on both and is a few small commands on `associate.rs`.

## Open questions

**Does `ms-settings:defaultapps?registeredAppUser=Fuji` work on Windows 10**, or only on 11? The parameter is documented for 11 with the April 2023 update; plain `ms-settings:defaultapps` is the fallback. Whoever builds the settings page should answer it first, because it decides whether the button lands on fuji's own entry or drops the user on a list to hunt through.

**Reading the current handler on macOS is the awkward direction.** `NSWorkspace.urlForApplication(toOpen:)` wants a file URL rather than a type, and the type-shaped call returns candidates rather than the winner. Either a throwaway file per type, or the deprecated `LSCopyDefaultApplicationURLForContentType`, which still works. An experiment for that pass.

**Linux** gets no `MimeType=` at all — checked in the generated package on 2026-09-17, where the `.desktop` file carries `Exec`, `Icon`, `Name` and an empty `Categories=` and nothing else. An earlier note here said otherwise and was wrong. Nothing in `tauri.conf.json` configures a desktop template, so fuji cannot be handed a picture on linux and a user drags files in instead. Deliberately left that way for now: linux desktop integration is its own sprint, and `bundle.linux.deb.desktopTemplate` is where it would start.

## Settled, and recorded so nobody researches it twice

**macOS needs no document icon, so the one fuji has is Windows only.** Checked on the Mac mini 2026-09-14 with `.jpg` and `.webp` set to fuji: Finder still draws every file as a picture of itself, because Quick Look previews win over a handler's document icon. Nothing was built for that platform and nothing should be.

**Explorer keeps drawing thumbnails for a type fuji owns, and it cannot be otherwise.** Fuji writes no `ShellEx` key anywhere, so it never registers as a thumbnail provider and has nothing to displace; and the thumbnail lookup does not follow `UserChoice` — it runs through the extension's own `ShellEx` or through `SystemFileAssociations\image` by `PerceivedType`, and fuji touches none of those.

**The registration takes nothing from anyone.** Before and after on the same machine: all ten ProgIDs and the `Capabilities` block appeared, fuji was appended to each extension's `OpenWithProgids` beside what was there, and no extension's default value was written and no `UserChoice` moved.

**Declaring by extension is enough on macOS.** It synthesised the right system UTIs — `public.jpeg`, `public.svg-image` — from bare `CFBundleTypeExtensions`, with no `LSItemContentTypes` anywhere. `.jfif` is the exception and gets a dynamic UTI, so *Change All* on a `.jpg` does not cover it; nothing to fix, worth knowing.

**Two bundles claim these types on a development machine** and share a bundle identifier — the copy in `/Applications` and the one under `target/release/bundle/macos/`. LaunchServices stores the choice by identifier rather than path, so it may launch either. Usually the installed one wins.
