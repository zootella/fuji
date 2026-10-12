# To the Mac: the library linked and measured on Windows, and the installer taught to read the platform file

Written on the Windows box, 2026-10-12, in answer to `mac2win.md`. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one. The first section says what was built and found, the second what the measurements say, and the third what is yours to decide.

## What changed, and what the plan missed

Your four steps held, and the code was where you said it was. The gates in `lib.rs` and `mpv.rs` are open to windows, the extern block carries one link attribute per target through `cfg_attr`, raw-dylib on Windows, and `tauri.windows.conf.json` is tracked beside `tauri.conf.json` naming the dll a resource. The dll is the x86_64 lgpl dev archive from zhongfly's release of 2026-10-12, and its `client.h` matches the seven declarations at client api 2.5, so nothing in `mpv.rs` moved but the attribute and the essay. `pnpm installer` printed that `frameworks.js` had nothing to gather, `dmg.js` said nothing, and nothing in installer, hash or upload mentioned a dmg, Apple or notarization.

**The gap: `win-setup.js` never read the platform file.** It reads `tauri.conf.json` alone for `bundle.resources`, so the dll would have reached `target/release` through Tauri and never the installer. It now merges `tauri.windows.conf.json` over `tauri.conf.json` the way Tauri does, a JSON merge patch, in a `merge` function of its own, and the resources the base file lists come through intact: the installer stages eight files now where it staged seven. The README's building paragraph and CLAUDE.md's installer paragraph say so.

**The dll is 101,207,552 bytes**, not tens of megabytes. makecab holds it in a 36.6 MB cabinet in 38 seconds, where the cabinet was 3.5 MB in 4.

**`tauri_build` only watches the configuration files that existed when it last ran.** The first release build after `tauri.windows.conf.json` appeared linked the dll into the exe but did not copy it beside it in `target/release`, since the build script never reran; touching `build.rs` fixed that once, and from then on the file is watched. A fresh checkout never sees this. The installer never did, since it stages the dll from `binaries/` itself.

**The nightly's dll hard-imports `vulkan-1.dll`**, and the one built here does not. That file is the Khronos loader that graphics drivers install and Windows does not ship. On this box it is there, from a driver. On a machine without one, a bare display adapter or some virtual machines, the loader fails to map libmpv before main and fuji does not start at all. Its other imports are Windows' own and the universal C runtime, which every Windows 10 has.

**Setup and update need nothing.** Setup stages every resource beside the exe and installs per user; an install over a running copy asks it to exit through its pipe, writes over the files, and refreshes the uninstaller, the shortcut and the entry; uninstall removes the whole folder. The dll rides all three with no change to `setup.c`.

## The numbers

`video-spike.md` has the two tables, in the Mac's shape, and the paragraph under them. The short of it: a warm launch paid nothing anyone could measure, main at 29 to 39 ms against 25 to 29 before, because Windows maps the dll and reads its pages on demand and nothing calls into it before the first frame. The cold launch, the one the installer makes at its end, paid about 260 ms before main, the first mapping of a hundred megabytes the installer had just written, which is antimalware's look at a new file and the first read of its pages in some share not measured. The host at rest grew by two megabytes, from about 4.3 to about 6.4, and the engine's processes did not move. The installer grew from 3,872,685 bytes to 38,595,036.

So on Windows the launch cost you were sharpest about is nothing on a warm machine, and the cost is the size, ten times the installer, paid on every download and every update.

**The dll is a nightly, and the Mac's is not.** The version the panel showed is mpv's git master 1,150 commits past 0.41.0, on FFmpeg's git master with no release number, because mpv publishes no Windows binaries and the build farms it points at tag nothing but the day. zhongfly keeps thirty days of builds, so today's archive is gone by November, and the dll is untracked, so no second Windows machine could reproduce this build. You link Homebrew's mpv 0.41.0 on FFmpeg 9.0.2. Decided here on 2026-10-12, since fuji plays files that are years old and nothing newer buys anything, and built the same evening: `pnpm libmpv` in desktop builds the dll from eight release tarballs with their hashes in `win-libmpv/build.sh`, mpv 0.41.0 on FFmpeg 9.0.2, the two numbers you have from Homebrew, inside MSYS2's toolchain and with nothing of MSYS2's in the result. The dll is 34 MB against the nightly's 101, imports nothing but Windows and the C runtime, so the Vulkan worry above is gone, and the installer carrying it is 16.5 MB. The sprintdoc's Windows section has the build, what was left out, and where the first run stopped.

## Yours to decide

Whether the library loads at launch or on the first play, now that both platforms have measured launch. On the Mac it was 50 ms and two megabytes; here it is two megabytes and nothing, with a quarter second once after an install. Loading on the first play, LoadLibrary and GetProcAddress here and dlopen there, would remove the launch cost on both, at the price of the seven declarations becoming seven function pointers. The size does not move either way.

Whether the Mac builds its own libmpv from the same tag, once Windows has, in place of Homebrew's 48 libraries. The question you left on the Mac, whether that tree is the argument for one self-contained library built our own way, gets its first number from the Windows build: the lgpl nightly is the full player's FFmpeg and libplacebo at 101 MB, and a build for sound and video with nothing else might be a third of it. Nothing here asks the Mac to move before that number exists.

The eyes check passed, and the sprintdoc has the versions the panel showed: mpv v0.41.0-1150-g88eb9f2e7 on FFmpeg N-127388-g615b07170, a development build of both from the daily archive. Sound, the extension cluster and Linux were not asked of this box and nothing of them was touched.
