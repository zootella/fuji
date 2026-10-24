# Safe scout

What decides whether Rust code goes in the scout, the crate at `./scout`, or in fuji's own crate at `desktop/src-tauri`, and what ergo is beside them. Which process runs the code is a different question and never this one: the scout is compiled into fuji, and runs in fuji's own process or in the restricted copy of fuji an inventory can run in, the same code in both. Nothing here is a change to make: it's what to keep in mind as the three grow.

## Why the scout is a crate of its own

**A wall the compiler holds.** The scout's crate root forbids `unsafe` code and denies, through clippy, anything that could panic: an index out of range, an unwrap, arithmetic that could overflow. A crate reaches only what its `Cargo.toml` names, so the scout can't call fuji's own `unsafe` code, since it can't see it. The code that reads every file a collection holds, which is the code a hostile file attacks, is safe Rust by a rule checked on every build rather than by a reviewer's attention.

**Tests alone.** The scout's tests build and run in seconds on any machine, with no Tauri and no media library around them.

**Fuzzing in practice.** libFuzzer runs the reader and nothing else, millions of inputs, where fuzzing inside fuji would instrument Tauri, the platform bindings and libmpv along with it.

**No knowledge of fuji.** It knows file formats and file systems and nothing of the app, so a sister project could use it as it stands. The fuji-shaped parts are the thin command in `scout.rs` and the inventory in `inventory.rs`, which read the slices and hand them over.

## What "safe" covers

**Forbidding `unsafe` covers a crate's own source, and not what it calls.** Calling a safe function whose insides use `unsafe` is allowed, or no Rust program could read a file. The scout's own logic can't misuse memory; the `unsafe` it relies on sits in Rust's standard library, the most reviewed Rust there is, and in its other dependencies.

**The walker reaches the operating system through the standard library.** Listing a folder and describing a name are safe standard library calls, which make the POSIX calls on the Mac and Linux and the Win32 ones on Windows inside it. The standard library's small per-platform extensions are safe too, and the walker uses them for a folder's device and inode, the Mac's flag on a cloud file not yet downloaded, and Windows' file attributes.

**Fuji's own crate can't forbid `unsafe`.** The standard library wraps what every platform shares, files, folders, processes, threads, time and sockets, and nothing else. AppKit's windows, ImageIO, WIC, the Windows registry, the Mac's memory counters and libmpv have no safe wrapper, so fuji calls them through the foreign function interface itself, in `unsafe` blocks.

**Ergo is safe by fact, not by promise.** Its own Rust holds no `unsafe`, but it doesn't forbid it at its root, so the compiler doesn't hold it there. The crates it depends on use `unsafe` inside: blake3, sha1 and sha2 for fast processor-specific hashing, and getrandom to ask the operating system for random bytes. Forbidding `unsafe` in ergo would be the ergo project's change.

**The scout compiles ergo's hashing crates.** It depends on ergo for time spans and byte sizes, and ergo always builds its hashes, so the scout's builds carry them unused. The bytes of a collection's files still pass only through the scout's own safe code.

## Where new Rust code goes

**Needs Tauri or the operating system's own libraries.** A window, an event, a command the page calls, or a call through `unsafe`: fuji's crate.

**Knowledge about something from outside, computed from its input.** The bytes of a file, the names and kinds on a disk, in safe Rust alone, where a hostile input is the risk and sample files and fuzzing are how the code earns trust: the scout.

**Both.** Split it the way the reader is split: the scout gets the core that decides, and fuji gets a thin command that does the reading or the talking to the operating system and hands the data over.

**Ergo's job.** Hashing, dates, times and sizes: fuji's command calls ergo. That's why the whole-file hashes and the fuji tip hash are in `inventory.rs`, on fuji's side, where reading a whole file is a command's disk work and the hashing is ergo's.

## What the separation costs

**Lockfiles.** The scout keeps its own `Cargo.lock`, and its fuzz target a third, beside fuji's, and a dependency they share moves in every one of them together. Moving ergo from 0.1 to 0.2 touched every Cargo lockfile in the repository and moved the scout off a version it was content with, since `0.1` and `0.2` can't unify in one build.

**A second place to look.** Knowing where a thing lives takes the rule above.

**A seam that carries only data.** What crosses between the crates is plain data the page can read, like the reader's answer, and nothing shaped by Tauri.

**Two folder readers.** `find.rs` in fuji reads many folders in one call, and the contact sheet uses it. `walk.rs` in the scout walks a folder tree with every name's kind, and the inventory in `inventory.rs` walks with it.

**Testing fuji's own crate.** It works the ordinary way, a `#[cfg(test)]` module in any file and `cargo test` in `desktop/src-tauri`, and its first build compiles all of Tauri. On the Mac the test program dies at launch for want of libmpv, since the library's folder is given only to the app bundle: a test binary of fuji's library crashed that way on the Mac mini on 2026-10-17. A line in `build.rs` giving test builds the same folder is the likely fix, and is untried.

**Fuzzing fuji's own crate.** A fuzz target can depend on fuji's library and call any public function, and the build would instrument Tauri, the platform bindings and libmpv with it, which is slow and fragile. A pure core that needs fuzzing moves into a small crate instead, the way the reader did.

## Moves considered

**The scout into fuji.** *Gains:* one crate, fewer lockfiles, nothing to keep in step. *Losses:* tests alone, practical fuzzing, and the compiler's wall: a module can forbid `unsafe` in itself, but it can still call the rest of its crate's.

**More of fuji into the scout.** *What could move:* only code with no Tauri and no foreign calls, which in fuji's crate is the arithmetic of `fit.rs` and the folder reading in `find.rs`, which could become a thin command over the walker and end the overlap. *What stays:* every command, the platform layer, and the log, which is most of fuji's Rust: fuji's crate is wide and close to the operating system by nature, and the scout is small and deep.

**A Cargo workspace.** A root `Cargo.toml` naming `desktop/src-tauri`, `scout` and `argon` as members, with one shared `Cargo.lock` and one build cache, the way pnpm's workspace shares one lockfile. *Gains:* each crate keeps its own rules and its own tests, and a shared dependency resolves once. *Losses:* a root manifest to keep, and Tauri's tooling and the Linux containers' staging would both meet it; the fuzz target usually stays outside. Untried here.
