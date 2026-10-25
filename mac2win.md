# To Windows: a dark fringe on transparent thumbnails, the inventory's file system facts, and the sandbox process

Written on the Mac mini on 2026-10-23. Read this, then talk with the user before changing anything; a letter starts a conversation rather than settling one.

## What the Mac found, and fixed

**The Mac's thumbnailer let the color hidden under a PNG's clear pixels into its edges.** A PNG keeps a color for every pixel, even a fully clear one, and most programs that save PNGs leave black there. A correct shrink weighs each pixel's color by its opacity first, so a clear pixel's hidden color counts for nothing. ImageIO's thumbnail resample didn't, so the hidden black bled into every outline: a thin dark fringe, olive where it meets yellow. Measured on the Mac mini at the contact sheet's 210 pixels, ImageIO's thumbnail of `firefox.png` had 1,215 near-black partly clear edge pixels, and a correct shrink of the same file has 1. The green-hidden test disc, white on clear pixels that hide pure green, came back with 1,104 green-tinted edge pixels.

**How the user saw it: only with Firefox's logo, as a PNG and an SVG side by side on the contact sheet.** The sheet draws an SVG in the web engine, which shrinks correctly, so the SVG was clean beside it, and the PNG's olive line along the flame's yellow edge stood out against it. Neither picture alone showed it, and the green disc looked clean to the user by eye while carrying those 1,104 tinted pixels. The two pictures, from Wikimedia Commons:

- [Firefox's logo as an SVG](https://commons.wikimedia.org/wiki/File:Firefox_logo,_2019.svg)
- [the same logo as the PNG Wikimedia renders from it, 960 pixels across](https://commons.wikimedia.org/wiki/Special:FilePath/Firefox_logo,_2019.svg?width=800)

**The fix, in the Mac half of `thumbnail.rs`.** A picture with an alpha channel now comes back from ImageIO whole, decoded and turned upright, and CoreGraphics shrinks it, in premultiplied pixels. A picture with no alpha channel keeps ImageIO's scaled decode, since it hides nothing. After the change, `firefox.png` has 1 near-black edge pixel and the green disc none.

## Why Windows may have it too

**WIC's Fant scaler gets the same input ImageIO did.** For a PNG, the Windows half of `thumbnail.rs` hands the scaler straight alpha, colors not weighed by opacity, and Microsoft doesn't document whether Fant weighs them as it averages.

**The only evidence it's clean is a look, and a look missed it here.** The comment above the scaler said the green disc came back clean on the Windows box on 2026-10-22, judged by eye; it now says so. On the Mac, the same disc looked clean too, and a count found 1,104 tinted pixels.

**What would settle it is a count.** WIC's thumbnail of `firefox.png` at 210 pixels, through the same steps `thumbnail.rs` takes, against a correct shrink of the same file, counting the partly clear edge pixels whose color, unpremultiplied, is near black; the Mac's 1,215 against 1 is the size of the fault to expect. Then the look the user took here: `firefox.png` and `firefox.svg` side by side on the sheet.

**If WIC bleeds, the fix is likely small.** WIC has premultiplied pixel formats, so converting to one ahead of the scaler, and back to straight alpha in the conversion the pipeline already ends with, would keep the hidden colors out, in WIC's own terms.

## The inventory's file system facts, which only Windows answers its own way

**Three things the inventory reads about each file come through code only Windows runs, and none of it has run there.** The changed time and the count of a file's names both come from `GetFileInformationByHandleEx` on the file the inventory already has open, one struct each, since Rust's standard library reads neither outside its unstable builds; `hidden` comes from the file's hidden attribute. All of it sits behind `#[cfg(windows)]` in copper's `inventory.rs`, with the `Win32_Storage_FileSystem` feature of the `windows` crate in copper's `Cargo.toml`. Copper has no Tauri in it, so it type-checks for Windows from the Mac, `cargo check -p copper --tests --target x86_64-pc-windows-msvc`, and that passed on 2026-10-24; nothing of it has run on Windows. The same is true of everything else that moved into copper this week, the registry, the thumbnailer's WIC half, the memory counters, the process's start time and the screen's size: type-checked here, never run there. `cd copper && pnpm test` runs copper's tests, and on Windows that includes the registry's, which writes, reads and deletes a value under a key of its own.

**What could be wrong there.** The arithmetic from hundred-nanosecond steps since 1601 to milliseconds since 1970 is checked on the Mac; what the call answers is not. FAT and exFAT keep no change time, and Windows may answer zero there, which the inventory leaves out, or something else, which would show up as a `changed` that means nothing. The inventory also writes every path with forward slashes, turning Windows' backslashes over as it writes each line.

**What to look at.** An inventory of a folder on the NTFS system drive holding: a file renamed after it was last edited, whose `changed` should be later than its `modified`; a hard link made with `mklink /H`, which should carry `"links": 2` under both names; and a file marked hidden in Explorer, which should carry `"hidden": true`. Then an inventory of a USB drive, for what `changed` says there. Every path in the file should read with forward slashes.

## The sandbox process, which Windows has to finish

**What runs there and has never run.** An inventory the panel sends to the sandbox process starts fuji's own executable again with `--inventory-sandbox`, through Rust's `Command`, its stdout the inventory file the main process opened, its request down stdin and its counts up stderr; the essays atop `copper/src/inventory.rs` and `copper/src/sandbox.rs` say the rest. It walked on the Mac and on Linux in Docker. On Windows nothing of it has run: not a GUI-subsystem executable starting itself with pipes, and not a file handle handed over as a child's stdout.

**The walls to build, in the Windows arm of `enter` in `copper/src/sandbox.rs`, which answers Ok and does nothing today.** scan.md's decided item: the sandbox process puts itself, after reading its request and before the walk, at low integrity, which stops writes to most of the user's profile and HKCU, and in a job object that allows no child process, with a memory limit and the flag that ends a process on an unhandled exception, which keeps Windows Error Reporting's dialog away as the Mac's exception port keeps ReportCrash away. Neither stops reading the user's other files or the network; the AppContainer would, and it cannot open the chosen folder by path, the open question scan.md names. Copper's Windows code cannot be type-checked from the Mac, since blake3's build script wants ml64.exe, so this compiles for the first time there.

**A handle may get through.** Rust's `Command` on Windows creates the child inheriting every handle marked inheritable, and the walls judge a file as it opens, so an inheritable handle the main process holds, one WebView2 or the system made, would arrive open in the sandbox process. The Mac closes every file but the three streams as the sandbox process starts and Linux marks them to close; the Windows analog is `CreateProcessW` with `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` naming the three. A look at the sandbox process's handles, with Process Explorer or `handle -p`, while an inventory runs from the panel, settles whether any arrive.

**What to look at.** `cargo build -p fuji`, then `cd copper && pnpm walk`, which uses the debug build, since a release build has no console for `fuji --inventory` to answer on. The check prints a line a reach, with ok or NO beside what full walls would answer, so before the walls every refusal reads NO, and after them reading or listing outside the folder and connecting read NO until the AppContainer. The crash step has the sandbox process fault through the request's test option and counts fuji's reports under WER's ReportArchive and ReportQueue; a count that moves, or a "fuji has stopped working" on screen, is the failure.
