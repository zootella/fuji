# Sorting the Rust into brass, copper and steel

The move of fuji's Rust into its three crates, and what is open.

## The move

- **Scout becomes steel.** The folder, the package, the fuzz package, the dependency lines, the workspace entry, the staging path in linux/build.js and every `::scout::` path change; the scout keeps its name as the reader inside steel, and scout.rs, scout.js and scout_tip keep theirs as the command for the scout.
- **Copper is made.** A folder at the root beside steel: a Cargo.toml naming steel by path, ergo, serde and serde_json, and the windows crate for the Windows pieces as they arrive; a package.json with test as cargo test and a line in pnpm-workspace.yaml; its own Cargo.lock; a README; and a build.rs that gives the link search for the media library in `desktop/src-tauri/binaries` and gives copper's test binary an rpath to that folder, while the shipped binary's rpath stays in brass's build.rs, since a dependency's link arguments do not reach the final binary.
- **First into copper**, from media.rs: the extern block, its structs and constants, the error and property helpers, the ask helpers, the probe and the frame; the playing half stays in brass over copper's declarations and keeps the render callbacks that reach window.rs. From inventory.rs: everything but the three commands, the state they share and the window-closed hook; the windows crate's Storage_FileSystem feature moves with the stat helpers. Into steel at the same time: tip_read, open_file and read_into from brass's scout.rs, since copper cannot depend on brass.
- **The sweep after**: thumbnail.rs, registry.rs, launch.rs, panel.rs, memory.rs and uptime.rs into copper; find.rs, fit.rs and the reads and writes of disk.rs into steel, rewritten to pass its lints; each checked on the Windows box as it goes.
- **Each step is a commit of its own, and each move is byte-exact.** A whole file moves with mv. A block moves by a script that cuts it between two anchor lines asserted to occur once, writes it to its destination unchanged, and proves the remainder with the cut put back hashes the same as the original. The edits a move forces, an import, a path, a visibility, are small and the compiler checks each. The diff is read with rename detection on, so a moved file shows only what changed in it.

## Open

- Whether the three become a Cargo workspace with one lockfile and one target folder, which Tauri's tooling and the Linux containers would then meet, and when.
- Whether steel, which so far only reads, takes disk.rs's writes and copies, which the rule sends there.
- find.rs and the walker read folders two ways; whether find.rs becomes a thin command over the walker once both are in steel.
- The line in brass's build.rs that hands its test binary the media library's folder, so a test in brass runs with no environment variable; a test ran with one on the Mac mini on 2026-10-23.
- The restricted copy as copper's own binary: a second executable to build, sign, notarize and bundle on three platforms, and to add to the Windows installer's cabinet.
- The amendment to the constitution's code chapter: the four places in place of the two layers.
