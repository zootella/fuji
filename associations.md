# File associations

Fuji being a program the operating system will hand a file to. **On Windows it is built and smoke tested**, on the Windows 10 box on 2026-09-28 and 29: an answer per extension in `fuji.toml`, the File types section of the settings panel, registration that offers every enabled type and claims only what the user said yes to, following a choice the user makes in Windows, and an uninstall hook that takes it all back. The reasoning lives with the code: the values and the design in the essay atop `associate.js`, the section in `SettingsFileTypes.vue` and `SettingsFileTypeCard.vue`, the lookup in `registry.rs`, the hook in `src-tauri/windows/hooks.nsh`, and every extension fuji knows in `fileTypes.js`. What is left here is the Mac's half, the edges still open on Windows, the whiteboard for every other kind of file, and the facts worth not researching twice.

## The Mac's half, to build there

The Mac declares fuji's types in `Info.plist` and has done since the first pass, so fuji is in Finder's Open With already. The File types section on the Mac shows the extensions in one list with their answers, disables the buttons, and says the Mac is coming. What the Mac has to build, which `win2mac.md` carries to it:

- **Two commands:** which application opens a type, and making this application the default for one — `NSWorkspace.urlForApplication(toOpen:)` and `setDefaultApplication(at:toOpen:completionHandler:)`, in the forms that take a `UTType`, which arrived in macOS 12. Confirm first that the reading form takes a type rather than a file.
- **A yes sets the default once, at the moment of the yes, and never again.** On the Mac the default an application can set is the user's saved choice itself, so setting it at every launch would take the type back from whatever the user picked in Get Info since. A no sets nothing: where the Mac still opens the type with fuji, the section shows the disagreement and points to Get Info, since taking it back would mean choosing another application for the user.
- **Extensions that share a type move together.** `.jpg`, `.jpeg` and `.jpe` are all `public.jpeg`, so a yes on one is a yes on all three; `.jfif` gets a dynamic type of its own.
- **Compare by bundle URL**, the way Windows compares by executable path: a development Mac holds two bundles sharing one identifier, below.

## Open on Windows

- **A type with no saved choice was never exercised.** There the fallback alone decides, so a yes should make fuji the default at once, with no trip to Windows Settings. Every type on the Windows box already carried a saved choice, so this needs a fresh Windows profile to see.
- **Windows 11 is documented, not measured.** Its Default apps link should open fuji's own page, listing every type fuji offers; only Windows 10's behavior, the general Default apps page, has been seen.
- **Switching a type off after it has been on would leave it registered.** Registration walks only the enabled types, so an extension switched off keeps its ProgID, its place in Open with, and any fallback a yes wrote, until an uninstall; meanwhile reading the answers drops it from `fuji.toml`, so switching it back on starts it at ask. Nothing has ever been switched off; the first time something is, registration should take its keys back.
- **Linux is left out for now.** Its packages declare no `MimeType`, so fuji cannot be handed a file there; `fuji.desktop.hbs` is where that starts, as a static list the way `Info.plist` is one.

## Every file, and what fuji can do with each

A whiteboard, started 2026-09-29, as the list of extensions grows toward everything a media player opens — video, sound and playlists — while fuji still shows only pictures.

**Where the contact sheet is headed.** Today it shows only the pictures in a folder. Soon it may list every file, so the user has the context of where they are: a file fuji cannot draw shows as a chip, the extension alone in a box, like `[.txt]`, and a double-click on it becomes a launch, the operating system opening it with its own default exactly as Explorer or Finder would. Fuji never has to understand a file to be the place a user finds it.

**What is decided.** `fileTypes.js` is the one table, and it records capabilities rather than categories: which parts of fuji can handle each format, on which platforms — `imageNative`, `imageWeb` and `contactSheet` today, and `videoNative` and `videoWeb` on the two videos. An entry can wait in the table switched off, described and ready, and a switched-off entry appears nowhere: not in a folder, not on the sheet, not registered with the system, not in the settings. Eighty-six entries wait there now, off, from PCX and tracker modules to WebM and Opus, and the playlists and subtitles that sit beside the media, their platform lists recording what fuji intends; switching one on is when its lists get checked against the code, and the essay atop the file says why the list runs so far past what fuji plays. The player the video properties anticipate is libmpv, through Rust.

**What is not built.** Frame thumbnails from the operating system, which `thumbnail.rs` does not ask for yet; playback, through libmpv or the web renderer's video tag; the chip's look on the sheet, where today a file nothing can draw gets the placeholder; and the launch, handing a file fuji cannot show to the operating system.

**Questions.**
- The launch needs the opener plugin's open-path permission, scoped with thought, since it is a way to start any program the system associates with any file fuji lists.
- `thumbnail.rs` refuses bytes it does not recognize before any decoder runs, so every format given a native route needs a signature there, and video containers are more varied than image headers.
- `Info.plist` would grow by dozens of hand-kept entries. Generating it from the table was declined while the table changed about once a year, and is worth deciding again.
- `imageWeb` is only as true as the assumption behind it: every picture lists all three platforms because fuji has always assumed the engine decodes every picture everywhere, which is already not quite true of older macOS and AVIF.

## Settled, and recorded so nobody researches it twice

**The second pass passed its smoke test on the Windows 10 box, 2026-09-28 and 29.** A yes Windows disagreed with turned the chip amber and wrote only the fallback; Windows Settings, reached from the link, settled it, and fuji noticed when its window came back into focus. Making fuji the default through Explorer's Open with, Choose another app, Always, was followed as a yes. Uninstalling removed every key fuji wrote and none of another program's, and left the user's saved choices naming fuji; reinstalling brought back the answers from `fuji.toml` and the choices with them. Along the way: Windows shows a one-time chooser on the first double-click of a type after a new program registers for it, and Settings' *Set defaults by app*, *Manage* can label a type *Choose a default* even while it has a saved choice.

**What the user meets.** On macOS, dragging `Fuji.app` to Applications announces nothing, and fuji is in Finder's Open With for its types before it has ever run; a user who wants fuji uses ⌘I, *Open with*, *Change All…*, and confirms the sheet. On Windows the installer writes nothing about file types, and the installed copy registers itself on its first launch. **Open with → Fuji** opens a file once and does not change the default, which is right and which users find confusing; *Choose another app* with *Always use this app* ticked is the route that does. Once fuji opens a type, Explorer's Type column shows fuji's type for it and every such file wears `document-image.ico`, most visibly in Details view with thumbnails off.

**The installed-copy gate** registers only when the running executable's folder equals `InstallLocation` under `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Uninstall\Fuji`, which Tauri's NSIS template writes on every install and the uninstaller removes; copies run from `target/` register nothing. The template also keeps the folder as the default value of `Software\<manufacturer>\Fuji`, and that one is the wrong thing to test: the uninstaller deletes it only when *delete the application data* is ticked.

**The uninstaller's *delete the application data* checkbox reaches the WebView2 cache and not fuji's own state.** It removes `%APPDATA%\<identifier>` and `%LOCALAPPDATA%\<identifier>`, where the web engine keeps a cache of up to a hundred megabytes. `fuji.toml` and the `fuji-temp` logs sit in the home folder and survive it, deliberately, since a settings file a user edited is not obviously the installer's to destroy, and so do the three answer lists, which is how a reinstall remembers them.

**macOS needs no document icon.** Checked on the Mac mini 2026-09-14 with `.jpg` and `.webp` set to fuji: Finder still draws every file as a picture of itself, because Quick Look previews win over a handler's document icon.

**Explorer keeps drawing thumbnails for a type fuji owns.** Fuji writes no `ShellEx` key, so it never registers as a thumbnail provider, and the thumbnail lookup does not follow `UserChoice` — it runs through the extension's own `ShellEx` or `SystemFileAssociations\image` by `PerceivedType`, none of which fuji touches.

**Declaring by extension is enough on macOS.** It synthesised the right system UTIs — `public.jpeg`, `public.svg-image` — from bare `CFBundleTypeExtensions`, with no `LSItemContentTypes`. `.jfif` is the exception and gets a dynamic UTI, so *Change All* on a `.jpg` does not cover it.

**Two bundles claim these types on a development Mac** and share a bundle identifier — the copy in `/Applications` and the one under `target/release/bundle/macos/`. LaunchServices stores the choice by identifier rather than path, so it may launch either; usually the installed one wins.
