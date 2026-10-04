# From the Mac to Windows: COM on threads that come and go

Written on the Mac mini, 2026-10-04, alongside the change that moved every waiting command's body onto Tokio's blocking pool. Read this, then talk to the user before changing anything; a letter starts a conversation rather than settling one.

## What changed

**Every command that waits now runs its body through `run_blocking` in `lib.rs`**, which hands it to `spawn_blocking`, Tokio's pool for blocking work. Until now those bodies ran on Tokio's worker threads, one per core, which live as long as the app. The essay above `disk_readdir` in `disk.rs` has the reasons and the costs. One of the costs lands on Windows alone, and one Windows body changed shape along the way.

**The blocking pool's threads come and go.** It starts a thread for each body that waits, up to 512, and retires one after ten idle seconds. The Windows thumbnail initializes COM on whatever thread it runs on, because WIC is a COM library. On the old workers that happened once per thread and never mattered. On threads that are created and retired all session long, an initialize that is never paired with an uninitialize is the thing Microsoft asks a program not to leave behind.

**So each render now pairs them.** In `thumbnail.rs`, `start_com` returns a `Com` guard whose `Drop` calls `CoUninitialize`, and `render` declares it first, as `let _com = start_com();`, so it drops last: every COM object — the factory, the decoder, the frame, and everything `render_com` builds — is released before COM is uninitialized, on every path out, the `?` ones included. A `CoInitializeEx` that answers `S_OK` or `S_FALSE` gets a guard; one that answers `RPC_E_CHANGED_MODE` gets none, since that call owes no uninitialize. Please read that code with a Windows eye before trusting it: the drop order is the whole of its correctness, and it was reasoned about rather than watched.

**Pairing them has a cost of its own.** `CoUninitialize` is documented to unload the DLLs COM loaded for the thread, so when the last render of a burst finishes WIC may be unloaded, and the next burst loads it again. Renders in flight together keep it loaded for each other, so this is per burst rather than per render, and nobody has measured it. Initializing once per thread and uninitializing from a thread-local destructor was considered and set aside: Rust may run those destructors under the loader lock, where Microsoft warns that `CoUninitialize` can deadlock.

**`panel.rs`'s Windows body changed shape.** Its `catch_unwind` closure became a plain `unsafe` block, since `run_blocking` now catches a panic for every command. Nothing else about it changed.

Both Windows bodies were type-checked on the Mac against `x86_64-pc-windows-msvc`, copied into a scratch crate without Tauri on the same `windows` 0.62.2. Neither has been built with Tauri on Windows, and neither has run.

## What to check

- **A release build.** `pnpm compile` in `desktop`, which is the first time either body is compiled as part of fuji on Windows.
- **Thumbnails across idle gaps.** Open a folder of JPEGs and PNGs on the contact sheet, wait more than ten seconds so the pool retires its threads, then open another folder, a few times over. Every thumbnail should arrive, and the log should hold no `thumbnail:` errors that name COM or `CoCreateInstance`. If a render ever fails on a fresh thread and succeeds on a retry, the initialize is the first suspect. And compare the first thumbnail's `render` time after an idle gap with the ones that follow it, which is where reloading WIC would show; the site's thumbnail page has 65 ms for a 6000 by 4500 JPEG decoded alone, measured before this change.
- **The panel's resolution.** The List view, `MyList.vue`, shows what `panelResolution()` answers; it should be the display's resolution as before, not `0 × 0`.
- **A table over a filling sheet.** Nothing pauses a card any more, so a folder opened while a table is showing fills the sheet behind it. On Windows, BMP, WebP and AVIF take the page route, which draws on the same main thread the table uses. Flip through large pictures while a hidden card of those formats fills, and say whether the flips hitch.
