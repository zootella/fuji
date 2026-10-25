# Copper

Fuji's Rust that needs neither Tauri nor a window and cannot be safe Rust: every call Fuji makes into C and into the operating system's own libraries away from a window. Each function here is a plain function that a thin command in Fuji's Tauri crate wraps, so this crate builds and tests with nothing of Tauri around it, and a search for unsafe in it finds every such call Fuji makes outside a window.

It is a Rust crate of its own. Fuji's Rust depends on it by path, and nothing publishes it.

```
pnpm test    cargo test, the crate alone
pnpm walk    the sandbox process walked through a built fuji, on whichever machine runs it
```

## What it holds

The media library, libmpv, its declarations and the questions asked through it without a window, the probe and the frame; the inventory's loop, which walks a folder, reads every file and writes a line each, in the main process or in the sandbox process; the sandbox process's start and its walls; the operating system's thumbnailer; the Windows registry; the Mac's launch services; the display's size; the machine's memory and what fuji's processes take; and when the operating system started this process. Each file says what it offers at its top, and the commands over them are in fuji's Tauri crate under the same names.
