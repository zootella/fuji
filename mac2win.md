# To the Windows box: the video spike's link and measurements, and a build log you have not seen yet

Written on the Mac mini, 2026-10-12. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one. The first section asks for work, the second only says what changed since your last pull.

## The video spike

`video-spike.md` at the root is the plan and holds the Mac's numbers. The short of it: libmpv is linked into fuji through declarations written by hand in `mpv.rs`, carried in the Mac bundle, and measured, and nothing draws a frame. The Windows steps are the same four, with its own library, and each has a mechanism that makes it different here.

**The baseline first, before anything links.** Build `pnpm installer` on the code as pulled, where `mpv.rs` is compiled only on the Mac and a stand-in in `lib.rs` answers that the library is not linked. The shell now writes two lines at the top of every log, the launch timed whole and the footprint at rest two seconds later; `log.record` has to be on in `fuji.toml`. Launch the built exe three times from the bundle folder, a few seconds and a quit each, and read the launch and rest lines from the three files in `fuji-temp`. The first launch after a build is the cold one. The Windows body of `uptime.rs`, which reads the process creation time through GetProcessTimes, has never been compiled, since it sits behind its target; the windows crate feature it needs, Win32_System_Threading, is already on. The Mac's tables in the sprintdoc show the shape to match.

**The library.** zhongfly's mpv-winbuild releases on GitHub publish an lgpl dev archive each day, named `mpv-dev-lgpl-x86_64-<date>-git-<hash>.7z` as of 2026-10-12, whose `libmpv-2.dll` carries FFmpeg and libplacebo inside it, one file, which is the reason for that build over any other. Take the plain `x86_64` one and not the `x86_64-v3` beside it, which wants a newer processor than the Windows box has. It is a 7z archive; Windows 10's own `tar` reads 7z through libarchive, and 7-Zip does if it does not. Put the dll in `src-tauri/binaries/`, which `.gitignore` already covers; it is tens of megabytes and never tracked. The archive's `include/mpv/client.h` is the header `mpv.rs` declares against, and it is worth a minute to check the seven signatures there match, since the Mac's were checked against Homebrew's copy of the same version.

**The link.** Two gates open. In `lib.rs`, `mod mpv` is behind `target_os = "macos"` with the stand-in for everything else; it becomes macos or windows. In `mpv.rs`, the extern block carries `#[link(name = "mpv")]`, which on the Mac finds `libmpv.dylib`; on Windows it wants `#[link(name = "libmpv-2", kind = "raw-dylib")]`, one attribute per target through `cfg_attr`. The raw-dylib kind makes the linker import from the dll by name, with no import library and no tool run to make one, which is why it is chosen over the `.def` in the archive; it has been stable since Rust 1.71 and `rust-toolchain.toml` pins a later one. `build.rs` adds a link search path on the Mac alone and needs nothing on Windows.

**Shipping.** The loader searches the executable's own folder first, so the dll beside `fuji.exe` is found with no path, and `win-setup.js` already stages everything in `bundle.resources` beside the exe. But `tauri_build` reads `bundle.resources` at compile time on every platform and fails the build on a file that is not there, so the entry cannot go in `tauri.conf.json`, where the Mac would trip on it. It goes in `src-tauri/tauri.windows.conf.json`, which Tauri merges into its configuration on Windows alone, and which can be tracked, since it names nothing secret: `{"bundle": {"resources": {"binaries/libmpv-2.dll": "libmpv-2.dll"}}}`. Two things to verify there rather than trust. That the merge keeps the resources `tauri.conf.json` already lists, the tile manifest and the document icons; Tauri merges configuration as a JSON merge patch, which unites objects and replaces arrays, and resources is an object. And where `tauri_build` puts the dll for `tauri dev`: it copies resources into `target/debug` so the dev exe can find them, and the dll has to land in the exe's own folder and not in a subfolder, or the first launch dies with "libmpv-2.dll was not found" before main.

**Then the after, the same way.** Three launches, the launch and rest lines, and the installer's size against 3,857,673 bytes today, since makecab compresses what it carries. On Windows the launch cost is paid on every double-click, because each picture window is a process of its own, which is the reason this number matters more here than on the Mac, where it was about 50 ms once. `pnpm installer` now runs `node frameworks.js` first, which prints that there is nothing to gather off the Mac and exits, and that is right.

**Eyes, once it links.** `a` on the contact sheet shows the audio panel, with one line, the library's version, its client api and FFmpeg's; `a` again goes back. `s` shows the settings, and the About section at the bottom is lines now, each part's name a link in gray with its version in black; the engine's line should read "WebView2" with the runtime's four-part version and "Microsoft Edge" after it, and the libmpv and FFmpeg lines should be there. If the version line on the audio panel is an error sentence in the warning color, that is the library refusing, and the sentence says why.

Playing sound, the extension cluster and Linux are not asked of you; the Mac takes the sound next, and the sprintdoc has the rest.

## Since your last pull: the build log and the apple workspace

**The Mac release now goes through Apple, from a fourth workspace, `apple`, and none of it runs here.** `pnpm install` at the root installs its one dependency, which nothing on this machine uses, and `pnpm notarize` in desktop is the Mac's alone.

**`pnpm installer`, `pnpm hash` and `pnpm upload` publish the setup program as they always have, and now also log their steps.** Each step writes a line to `build-log.txt` at the repository root as it starts and as it ends. The file is gitignored, so each machine keeps its own. The code is `build-log.js`, beside `scripts.js`, and its essay describes the line format. `win-setup.js` and `scripts.js` both import it. It runs nothing when imported, so `scripts.js` still runs its verb exactly as before.

**Times people read are now written like `2026-Oct-11 Sun 02:45:07.123 PM`** across the pipeline, while the sidecar's date and the log's first field stay ISO, in UTC.

`pnpm installer` runs Tauri's build as before. Then `node dmg.js` returns at once without a word, since a dmg is the Mac's alone. Then `win-setup.js` prints its usual lines between two new ones:

```
2026-Oct-11 Sun 02:45:07.123 PM  exe started  of the program tauri just built, by pnpm installer
2026-Oct-11 Sun 02:45:13.456 PM  exe finished  Fuji_0.1.0_x64-setup.exe  <sha256>  in the bundle folder, where pnpm hash finds it, took 6.3 seconds
```

`pnpm hash` prints the line it always printed, between `hash started fuji.exe` and `hash finished fuji.exe <sha256> took …`. `pnpm upload` prints scp's output, between `upload started fuji.exe` and `upload finished fuji.exe <sha256> and fuji.exe.json, took …`.

`build-log.txt` then holds those six lines. Each carries the commit the checkout was at, followed by a + when tracked files had changed since. Hash changes the sidecar, so the upload lines will carry the + until the sidecar is committed. Without git on the path, the commit reads `-` and nothing else changes.

If anything in those three commands mentions a dmg, Apple or notarization, that is new, so tell the user.
