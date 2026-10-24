# Copper

Fuji's Rust that needs neither Tauri nor a window and cannot be safe Rust: every call Fuji makes into C and into the operating system's own libraries away from a window. Each function here is a plain function that a thin command in Fuji's Tauri crate wraps, so this crate builds and tests with nothing of Tauri around it, and a search for unsafe in it finds every such call Fuji makes outside a window.

It is a Rust crate of its own. Fuji's Rust depends on it by path, and nothing publishes it.

```
pnpm test    cargo test, the crate alone
```
