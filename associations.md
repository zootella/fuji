# File associations

Fuji being a program the operating system will hand a file to. **On Windows it is built and smoke tested**, on the Windows 10 box on 2026-09-28 and 29, **and on the Mac it is built and smoke tested**, on the Mac mini on 2026-10-01. The reasoning lives with the code: the design and the platform facts in the essay atop `associate.js`, the panel in `SettingsFileTypes.vue` and `SettingsFileTypeCard.vue`, the lookups in `registry.rs` and `launch.rs`, the hook in `src-tauri/windows/hooks.nsh`, and every extension fuji knows in `fileTypes.js`. This file is only what is still open.

## Open on the Mac

- **Extensions that share a kind move together, and the panel doesn't say so.** `.jpg`, `.jpeg` and `.jpe` are all `public.jpeg`, so Choose Fuji on one moves all three chips under Opens with Fuji at once, which is right and may surprise. `launch.rs` could answer the kind beside the application, and the card could name the siblings: "Moves with .jpeg and .jpe". Windows would never show it, since there every extension stands alone. An enhancement after the first draft, by the user's call on 2026-10-01.

## Open on Windows

- **Switching a type off would leave it registered.** Registration walks only the enabled entries, so an extension switched off keeps its ProgID, its place in Open with and any fallback a yes wrote, until an uninstall; meanwhile the answers drop it from `fuji.toml`, so switching it back on starts it at ask. The uninstaller already enumerates `Capabilities\FileAssociations` to learn what fuji wrote, and registration could do the same to take back what is no longer enabled; that needs one Rust command that lists a key's value names. Nothing has been switched off yet, and this should land before anything is.
- **A type with no saved choice was never exercised.** There the fallback alone decides, so a yes should make fuji the default at once with no trip to Windows Settings. Every type on the Windows box already carried a saved choice, so this needs a fresh Windows profile.
- **Windows 11 is documented, not measured.** Its Default apps link should open fuji's own page listing every type fuji offers; only Windows 10's behavior, the general page, has been seen.
- **Linux is left out.** Its packages declare no `MimeType`, so fuji cannot be handed a file there; `fuji.desktop.hbs` is where that starts, as a static list the way `Info.plist` is one.

## The contact sheet's chips and the launch

Today the sheet shows only the pictures in a folder. Soon it may list every file, so the user has the context of where they are: a file fuji cannot draw shows as a chip, the extension alone in a box, like `[.txt]`, and a double-click on one becomes a launch, the operating system opening it with its own default exactly as Explorer or Finder would. Fuji never has to understand a file to be the place a user finds it. The launch needs the opener plugin's open-path permission, scoped with thought, since it is a way to start any program the system associates with any file fuji lists. Neither the chip's look nor the launch is built; today a file nothing can draw gets the placeholder.

## A User Guide page to write

What a user meets when making Fuji their picture viewer belongs on the site, in the User Guide's brochure-and-manual shape, and this paragraph stays here until the page exists. On macOS, dragging `Fuji.app` to Applications announces nothing, and fuji is in Finder's Open With for its types before it has ever run; a user who wants fuji uses ⌘I, *Open with*, *Change All…*, and confirms the sheet, or clicks Choose Fuji on the type in fuji's own settings, which does the same at once. On Windows the installer writes nothing about file types, and the installed copy registers itself on its first launch. **Open with → Fuji** opens a file once and does not change the default, which is right and which users find confusing; *Choose another app* with *Always use this app* ticked is the route that does, and Fuji's own settings panel is the other. Once fuji opens a type, Explorer's Type column shows fuji's type for it and every such file wears the document icon, most visibly in Details view with thumbnails off.
