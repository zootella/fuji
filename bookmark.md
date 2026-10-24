# Bookmark

Where the work stands and what comes next, kept so a session on any machine picks up where the last one left off. Written on the Mac mini on 2026-10-24.

## Done this stretch

- The inventory panel on t audited, and its third property added: `ffmpeg`, mpv's probe for every file the type table or the scout calls video, kept apart from `tip` and `stat` and never reconciled with them.
- Fuji's Rust sorted into three crates, one Cargo workspace at the root with one lockfile and the target folder pinned under the Tauri crate: brass is `desktop/src-tauri`, holding the commands and what a window needs; steel is `steel`, the safe crate, holding the scout, the walker, the reads, the disk, find and the fits; copper is `copper`, holding every call into C and the operating system's libraries away from a window, the media library, the inventory's loop, the thumbnailers, the registry, launch services, the display, the memory counters and the process's start. The page is plastic. `four-places.md` is the record of the four and the rule for placing new code.
- Tests over steel and copper for what left brass, 46 in steel and 16 in copper, and `rust-test-guide.md`, the style guide for tests, written from the research on what makes a suite good.

## Next, in order

- **A second pass over the tests**, by the guide: run cargo-mutants over steel and read what survives; property tests for the contracts across calls, find's resume from where it stopped and the inventory's line grammar parsing back; the fit table generated from fit.js over a grid rather than the 48 rows typed by hand; a stop set from another thread partway through an inventory; and the environmental tests in copper, the display's pixels, the machine's gigabyte, the process's hour, sharpened, renamed as smoke tests, or dropped.
- **The restricted copy of fuji**: the inventory run in a second process, chosen on its panel beside running in this one. The copy reads the chosen folder and nothing else, writes nothing, sends each line and its counts up a pipe, and fuji proper writes the file and shows the counts; Stop and the window closing kill it. The Mac's profile first, Windows and Linux after. Copper's own binary is the shape it can take, linking the media library and the standard library and nothing else; building the entry inside fuji's executable first and moving it later repeats nothing. scan.md holds the decided items and the open ones.
- **The Security article** on the site, from the history drafted in `site/docs/security.md` into fuji's architecture: steel as the wall around code, the restricted copy as the wall around a running process, the two independent, and what no design can do. security.md holds the research and the second half's outline.
- **The burn-down**: security.md into the article and the code's essays; scan.md's items into their homes as the sandbox lands; `safe-scout.md` and `core-scout-trout.md`, both spent, deleted.
