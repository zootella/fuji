# Bookmark

Where the work stands and what comes next, kept so a session on any machine picks up where the last one left off. Written on the Mac mini on 2026-10-24, and brought forward on the Windows box on 2026-10-26.

## Done this stretch

- The inventory panel on t audited, and its third property added: `ffmpeg`, mpv's probe for every file the type table or the scout calls video, kept apart from `tip` and `stat` and never reconciled with them.
- Fuji's Rust sorted into three crates, one Cargo workspace at the root with one lockfile and the target folder pinned under the Tauri crate: brass is `desktop/src-tauri`, holding the commands and what a window needs; steel is `steel`, the safe crate, holding the scout, the walker, the reads, the disk, find and the fits; copper is `copper`, holding every call into C and the operating system's libraries away from a window, the media library, the inventory's loop, the thumbnailers, the registry, launch services, the display, the memory counters and the process's start. The page is plastic. `four-places.md` is the record of the four and the rule for placing new code.
- Tests over steel and copper for what left brass, a second pass over them by the guide, and `rust-test-guide.md`, the style guide for tests, written from the research on what makes a suite good.
- The sandbox process: an inventory run in a second process of fuji's own, chosen by a radio on its panel, which is twinning; sandboxing on the Mac and on Linux, with the Mac's crashes kept from the system; `fuji --inventory` and `fuji --sandbox-check` for scripts and tests; and `pnpm walk` in copper, which runs the same walk on every machine, and in the linux workspace, which runs it in containers.
- The sandbox process on Windows: walled in with a job object and the Untrusted integrity level, its crashes kept from Windows Error Reporting by its own error mode, started by the main process through CreateProcessW with a list of its three streams rather than every inheritable handle, the check's Windows lines, the walk counting reports under ProgramData too, and the walk's hang step, which holds the sandbox process a second in to see Stop kill it.

## Next, in order

- **The deadline**, so an overnight inventory moves on past a file that hangs or crashes the sandbox process and stops for a drive that stops answering. inventory.md holds the design and what is open.
- **The Security article** on the site, from the history drafted in `site/docs/security.md` into fuji's architecture: steel as the wall around code, the sandbox process as the wall around a running process, the two independent, and what no design can do. security.md holds the research and the second half's outline, and windows-security.md the Windows facts and sources.
- **The sandbox process on a real Linux desktop**, from `mac2linux.md`, and on Windows the main process's own label against being ended from below.
- **The burn-down**: security.md and windows-security.md into the article; `safe-scout.md` and `core-scout-trout.md`, both spent, deleted.
