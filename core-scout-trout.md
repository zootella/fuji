# Core, scout and trout

Fuji's Rust in three crates, and the rule for which crate a piece of code goes in. Trout is a working name until a better one comes.

## The three crates

- **Core** is `desktop/src-tauri`, the crate Tauri builds the application from. It holds only what cannot leave: anything tied to a window, the event loop or Tauri's own types. The commands, the windows, the menu bar, the dock, the trackpad, the files the operating system handed fuji, the dialogs, the log and the text written at exit, and the playing half of the media library, which draws into fuji's window.
- **Scout** is `scout`, safe Rust held by the compiler: its root forbids unsafe code and denies, through clippy, an index that could be out of range, an unwrap, a panic and arithmetic that could overflow. It holds every piece of fuji's Rust that can live under those rules, which today is the reader of a file's first bytes and the walker of a folder tree.
- **Trout** is the third crate, for Rust that needs neither Tauri nor a window and cannot live under scout's rules: every call fuji makes into C or into the operating system's own libraries outside a window.

## The rule

- A piece of Rust goes to core only if it is tied to Tauri or a window. Otherwise it goes to scout. If it cannot meet scout's rules, it goes to trout.
- The rule is applied while code is written, not only when existing code is sorted. A function in scout that wants its first unsafe call, or one in trout that wants its first Tauri type, has to move crates, and that move is the question asked at that moment: why does this need to be unsafe, why does this need Tauri. Core is the least safe crate, so it does as little as possible and is read the hardest; trout is read the same way; scout stays the citadel.
- Scout's lints stay as they are. A module moving in is rewritten to pass them.
- Dependencies run one way. Core depends on trout and scout, trout depends on scout, scout depends on neither, and all three depend on ergo.
- Where the constitution's rule that a feature lives in one file meets this rule, this rule wins: a feature's code splits by what each part needs.

## What the rule does to fuji as it is

- Core keeps lib.rs and every command's thin wrapper; window.rs, menu.rs, dock.rs, touch.rs, open.rs, desktop.rs, log.rs, paths.rs, dialog.rs and process.rs; and the playing half of media.rs.
- Scout takes find.rs, fit.rs, the reads and writes in disk.rs, tip_read and open_file from scout.rs, and the Mac and Linux half of the inventory's stat.
- Trout takes thumbnail.rs, registry.rs, launch.rs, panel.rs, memory.rs, uptime.rs, the libmpv declarations and the probe from media.rs, the Windows half of stat, which is one Win32 call, and the inventory's loop, since the loop calls the probe.
- The inventory is the first feature the rule splits: its walk and its tip reading are scout's, its loop is trout's, and its three commands with their shared counters are core's.
- stat is the first function the rule splits by platform: safe standard library calls on the Mac and Linux, one Win32 call on Windows, so it is two functions in two crates or one function in trout.

## What it buys

- Tests of scout and trout build and run in seconds with no Tauri compiled. Core's own tests run too once build.rs hands the test binary the media library's folder, which is one line; a cargo test inside inventory.rs ran that way on the Mac mini on 2026-10-23, compiling in 8 seconds after the first build and running in a tenth of a second.
- Windows-only code in trout can be type-checked from the Mac with `cargo check --target x86_64-pc-windows-msvc`, which fails today only inside Tauri's build script.
- Every unsafe call fuji makes outside a window is in one crate, so a search for unsafe in trout is the whole list, and core's list is the windows, the menus and the dock.
- The restricted copy of fuji that runs an inventory can be trout's own binary, linking libmpv and the standard library and nothing else, rather than fuji's whole executable with AppKit and WebKit mapped before any wall goes up. Its entry, its pipe, its counts, its kill and its profile are the same code in either binary, so building it inside fuji's executable first and moving its main into trout later repeats nothing, and the sandbox call itself is unsafe, so it is trout's either way.
- The page's architecture does not move. State stays above the Rust, in JavaScript; the three crates sort only the Rust beneath it.
- Build size and run speed do not change: the same code, statically linked into the same executable.

## What it costs

- A feature's code splits across crates and files, and a change to one feature can touch three crates.
- A third crate is a third Cargo.lock that moves with every shared dependency, a third README and a third place to look, unless the three become a Cargo workspace with one lockfile and one target folder, which Tauri's tooling and the Linux containers would then meet. Untried.
- The rpath that finds libmpv at run time stays in core's build.rs, since a dependency's link arguments do not reach the final binary; trout's build.rs can give the search path for linking.
- A trout binary shipped as the restricted process is a second executable to build, sign, notarize and bundle on three platforms, and to add to the Windows installer's cabinet.
- Moving the Mac and Windows modules is checked on both machines, and the sweep of what exists is a sprint of its own.

## Open

- The third crate's name.
- Whether the three become a Cargo workspace, and when.
- The order of the move: trout made first with the probe, the inventory's loop and the restricted process's entry, and the sweep of the other modules after, or the sweep first.
- Whether scout, which so far only reads, takes disk.rs's writes and copies, which the rule sends there.
- The amendment to the constitution's code chapter that states the three crates and the rule, and what replaces the rule for where new Rust code goes.
