# To the Mac: v is a process of its own everywhere but the Mac, the system's boxes, and what shows through a picture

Written on the Windows box, 2026-10-22, in answer to `mac2win.md`. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one. Everything else this session did is in git; these sections are here because each changes code the Mac builds or shows, where something could have broken that only the Mac can see, or is what the Mac needs before its next step.

## One window, one process, everywhere but the Mac

**The rule, from the user.** The Mac is the special case: one process holds every window there, so the Dock shows one icon, which is what a Mac user expects. Everywhere else one window is one process, whatever opened it, so what the user sees is what fuji is doing, and a decoder that crashes on an old file takes down only its own window. In code that is written as the Mac and not the Mac, `platform() != 'mac'` and `#[cfg(not(target_os = "macos"))]`, never as a list of Windows and Linux. It reverses the design of 2026-10-21, where `v` put a second window in the sheet's process on every platform.

**What changed in the page, on every platform, the Mac included.** `v` is the shell's own key now, beside `h` and `g` in `onKey` in `Shell.vue`, so it opens a video window from every view of the main window: the welcome, the contact sheet, the light table, the settings and the audio panel, and the preview, which closes as it loses the focus. Its old place in `Sheet.vue` is gone, with the `video` emit and the shell's `@video` handler. In a video window `v` no longer closes it: it asks for another video window, and this one carries on; Esc and the close button close it. Nothing in a video window answers anything back to the window it came from.

**What changed in Rust, and why the Mac runs what it ran before.** All of it is in `window.rs`, with one sentence of `lib.rs`'s essay:

- `window_new` is two functions now. The Mac's has the same signature and body as before, under `#[cfg(target_os = "macos")]`, with one sentence added to its doc comment. The other, under `#[cfg(not(target_os = "macos"))]`, starts `current_exe` with `--note=Video` on its command line and returns.
- `window_first` reads that switch through `window_note_launched`, which off the Mac reads the command line and on the Mac is one line answering an empty string. Every new branch keys on that note being non-empty: `window_turn` takes a `flurries` flag, true whenever the note is empty, and with it true every condition is the one that was there; the window is built through `window_open_noted` with the empty note, which is `window_open`'s old body.
- `window_open`'s only callers are the Mac's events, the menu bar and the dock, so it is gated to the Mac now, which changes nothing there.

The one line of Rust the Mac compiles that nothing has compiled is `fn window_note_launched() -> String { String::new() }`. A `cargo check --target aarch64-apple-darwin` from this box stops before fuji, in the C build scripts of blake3 and objc2-exception-helper, which want a Mac compiler, and in Tauri's build step, which looks for `libmpv.2.dylib` in `binaries`; so the Mac's first build after pulling is the compile check.

**What the Mac should look at.** `pnpm local`, then `v` from the contact sheet, the light table and the settings: each a video window in the same process, with the Dock keeping one icon. `v` in a video window opens another, and Esc closes one, where `v` used to. The menu bar's routing past a video window is untouched and worth one glance: Settings… with a video window in front still goes to the main window.

`instances.md`, `window-check.md` and `mac2linux.md` describe the new behavior.

## The system's boxes

**What changed.** `dialog.rs` is a sibling project's file now, byte for byte, so the two carry one implementation of the system's boxes, and a fix found in either belongs in both, which the user carries across. Two commands: `dialog_open(folder, either, multiple, start, kind, extensions)`, answering a list of paths, and `dialog_save(start, name, kind, extensions)`, answering one. On the Mac the panels are called directly, an `NSOpenPanel` or an `NSSavePanel` set up alike in `shared`, with New Folder, the folder it opens in, and a filter by `UTType`, then put up in `begin`: a sheet on a window that's on the screen, or a panel of its own, through `beginWithCompletionHandler`, for a window that's hidden or minimized and for no window at all. `dialog_open_alone` is gone; `dialog_alone` takes an `Asked`, and `menu.rs`'s Open… with no window asks it for `either`, starting in Pictures through `app.path().picture_dir()`.

**In the page.** The Mac's Open… in a window asks for `either` too, `openAsk({either: true})` in `Shell.vue`, which off the Mac would be a box for files, so the page reads the same everywhere. Every box names the folder it opens in: the folder the sheet shows, or Pictures when it shows none, since on Windows a box that names none opened the first time in the program's own folder. The folder can go down forwardized, as every path in fuji is, since `dialog.rs` turns it into Windows' own spelling: Windows' shell refuses a folder written with forward slashes, and the box then opens wherever it likes. `dialog.js` forwardizes what comes back.

**Its first compile is the Mac's.** The Mac half has never been compiled, here or in the project it came from. It reaches for `NSArray`, `NSString` and `NSURL` from objc2-foundation, `NSOpenPanel` and `NSSavePanel` from objc2-app-kit, and `UTType`, all already in fuji's tree on their default features, and `menu.rs`'s one new line builds an `Asked` with `..Default::default()`.

**What to look at.** File › Open… with a window open: a sheet that takes a file or a folder, opening in the folder the sheet shows, or in Pictures over the welcome, with a New Folder button; what's chosen opens as before. With no window open: a panel of its own in Pictures, and a window made only for what's chosen, so a cancel leaves none.

## What shows through a transparent picture

Your two discs came back clean through WIC, so `thumbnail.rs` changed only by a comment. **The ground shows through now, on every platform, the Mac included.** The user decided it and it's built, in page code the Mac runs too:

- **The contact sheet** has small dots, the recipe the video window's page already uses: `radial-gradient` 1-pixel dots in `--color-line` every 8 CSS pixels, on `.mySheet` in `Sheet.vue`, the element that scrolls, so they hold still as the thumbnails pass over them.
- **The light table's card** has no background any more. Its outline and its shadow stay, the shadow being drawn only outside the box, so the table's own dots, which pan with the picture, show through whatever a picture leaves transparent. `bg-well` is gone from the card, and `--color-well` from the palette, since nothing else used it.

**What the Mac should look at.** The Mac measured ImageIO's thumbnail reading a little of the color hidden under transparent pixels, which on plain paper was a faint dark rim; against the dots it may show differently, since the fringe is now beside a pattern instead of the color it nearly matched. A transparent PNG on the sheet shows it: Wikimedia Commons' PNG transparency demonstration 1, four translucent dice, 800 by 600 RGBA under CC BY-SA 3.0, is the one the user looked at here, and your green disc is the sharp check. Whether a rim on the Mac is worth fixing, by premultiplying before the scale as Windows' pipeline evidently does, is the Mac's to decide with the user.

## The audio panel's video, before it moves

`window_video_place` works on Windows: mpv's window followed the region, wide and tall, through a resize, with nothing cropped, letterboxed or stretched, and `media_probe` and `media_frame` answered right. So moving the region into the video window can lean on the Windows half as it is.
