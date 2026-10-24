# Sorting the Rust into brass, copper and steel

The move of fuji's Rust into its three crates, and what is open.

## The move

- **The sweep after**: thumbnail.rs, registry.rs, launch.rs, panel.rs, memory.rs and uptime.rs into copper; find.rs, fit.rs and the reads and writes of disk.rs into steel, rewritten to pass its lints; each checked on the Windows box as it goes.
- **Each step is a commit of its own, and each move is byte-exact.** A whole file moves with mv. A block moves by a script that cuts it between two anchor lines asserted to occur once, writes it to its destination unchanged, and proves the remainder with the cut put back hashes the same as the original. The edits a move forces, an import, a path, a visibility, are small and the compiler checks each. The diff is read with rename detection on, so a moved file shows only what changed in it.

## Open

- Whether steel, which so far only reads, takes disk.rs's writes and copies, which the rule sends there.
- find.rs and the walker read folders two ways; whether find.rs becomes a thin command over the walker once both are in steel.
- The line in brass's build.rs that hands its test binary the media library's folder, so a test in brass runs with no environment variable; a test ran with one on the Mac mini on 2026-10-23.
- The restricted copy as copper's own binary: a second executable to build, sign, notarize and bundle on three platforms, and to add to the Windows installer's cabinet.
- The amendment to the constitution's code chapter: the four places in place of the two layers.
