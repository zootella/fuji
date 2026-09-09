# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fuji is a multimedia file manager designed with privacy and precision in mind. It's a Tauri desktop application built with:
- **Backend**: Rust (`desktop/src-tauri/`)
- **Frontend**: Vue 3 + JavaScript (`desktop/src/`)
- **Build Tools**: Vite, Tailwind CSS
- **Package Manager**: pnpm

The application displays images in an infinite pannable/zoomable space with keyboard navigation, drag-and-drop support, and full-screen mode.

### The repository is a pnpm monorepo

The application lives in the `desktop` workspace. The planning documents and the repository's own files stay at the root, where they describe the project rather than belonging to one part of it. `pnpm-workspace.yaml` names the workspaces, and the rule is that a directory holding a `package.json` is one — `notes/`, which holds the raw material the planning documents were written from, has none and is therefore just a folder. A second workspace, `site`, holding the VitePress website for fujidesktop.app, arrives from the `fuji-site` repository.

The root has no scripts, deliberately. `pnpm install` runs there and installs every workspace; everything else runs from inside the workspace it belongs to, so `cd desktop` comes first. **Throughout this document a path written `src/` or `src-tauri/` is relative to `desktop/`**, which is how the code refers to itself; only paths written from the root, like the build outputs below, carry the `desktop/` prefix.

**The planning documents are listed in `contents.md`**, which says what each one owns and which to read first. Read `structure.md` and `architecture.md` before changing anything structural, and `style.md` before the first edit.

## Development Commands

### Setup
```bash
pnpm install      # at the repository root; installs every workspace
cd desktop        # every command below runs from the workspace, not the root
```

### Run Development Mode
```bash
pnpm local        # Run Tauri in dev mode with hot reload
```

### Build
```bash
pnpm build-binary # Quickest proof the release build compiles and links; no bundles
pnpm build-app    # Also bundle the runnable app; skips the dmg and its finder theatrics
pnpm build-dmg    # Everything, including the dmg installer
pnpm build        # Same as build-dmg
pnpm app          # Launch the built mac app
pnpm win          # Launch the built windows exe
```

### Frontend Only (for rapid UI iteration)
```bash
pnpm dev          # Run Vite dev server without Tauri
pnpm vite-build   # Build frontend only
```

### Regenerate the Icons
```bash
pnpm icons        # rebuild every platform's icons from src-tauri/icons/app-icon.svg
```
One run writes the macOS `.icns`, the Windows `.ico`, the Linux PNGs, the Store logos, and the mobile trees together, so no platform needs its own run. These are committed artifacts, which means a Tauri CLI upgrade does not refresh them — re-run this after one. `icon.md` says why that matters and what it has already cost.

### Clean Build Environment
There are deliberately no cleanup scripts. The old `wash`/`upgrade-wash` pair were yarn-classic-era crutches — that ecosystem needed frequent clean reinstalls; pnpm's store does not. If a genuine mess ever needs clearing, delete `desktop/dist`, `node_modules`, or `desktop/src-tauri/target` by hand — and never delete the tracked lockfiles: pnpm-lock.yaml and Cargo.lock serve both mac and windows, and removing them to fix a problem is the anti-pattern that motivated the pnpm switch.

## Architecture

### Rust Backend (desktop/src-tauri/src/)

**Entry Point**: `main.rs` → `lib.rs::run()`

**Key Modules**:
- `disk.rs` - File I/O commands for JavaScript to invoke:
  - `disk_readdir()` - List directory contents (POSIX-like readdir), shallow, skipping entries it cannot stat
  - `disk_stat()` - Get file metadata (POSIX-like stat), describing a symlink rather than following it
  - `disk_read()` - Read an entire file, returning `tauri::ipc::Response` so the bytes cross as an ArrayBuffer rather than a JSON array of numbers
  - `disk_write()` - Create or truncate a file and write bytes
  - `disk_copy()` - Efficient file copying using kernel-space operations; overwrites the destination

- `panel.rs` - Hardware display resolution detection:
  - `panel_resolution()` - Returns physical pixel dimensions via platform-specific APIs
  - Platform implementations for Windows (Win32), macOS (CoreGraphics), and Linux (xrandr)
  - Answers about the primary display only, and asks every mode the display offers because no API reports the native one

- `desktop.rs` - The one thing only Rust can do, because only Rust sees a quit coming:
  - `desktop_exit_hold()` - Replace the text to write to a path when the application exits

- `log.rs` - Fuji's log, the half that holds the text and writes it:
  - `log(text)` - One line from any Rust code, appended to the run's log; a no-op unless the page started a log
  - `log_start(path)` - The page names the file, once, only when `log.record` is on
  - `log_append(text)` - The page's lines, a batch at a time
  - `log_write()` - Called from `RunEvent::Exit`, making the folder if needed; `src/log.js` is the page's half and carries the essay on why a file and not a console
  - Written from `RunEvent::Exit`, the one event every way of quitting reaches, making the file's folder first if it is missing

- `thumbnail.rs` - The operating system's thumbnailer, ImageIO on macOS and WIC on Windows, behind two commands:
  - `thumbnail_probe(paths)` - For each path, what its first bytes say it is and what its header says its size is, without decoding; one call per card. Refuses bytes fuji does not know and a header claiming a raster over half the machine's memory
  - `thumbnail_render(path, format, maximum, gamut)` - Decode the file scaled so its longer side is at most `maximum` pixels, oriented and color-converted, returning one buffer: a 12-byte header of width, height and whether the pixels are Display P3, then straight-alpha RGBA. Refuses a file whose bytes are not `format`. Runs on Tauri's thread pool. Rejects on Linux
  - `SquareFlow.vue` is the caller; `thumbnail-plan.md` says which files go here and which the page makes for itself

**Command Registration**: All Rust functions exposed to JavaScript must be registered in `lib.rs::run()` using `tauri::generate_handler![]`

**Important Architecture Notes**:
- File I/O uses synchronous operations; `disk_read()` loads entire files into memory (suitable for images, not large files)
- `disk.rs` holds no guard on paths, deliberately: its opening essay is the contract, and the walls are outside the file
- Comments in `disk.rs` extensively document memory efficiency tradeoffs between direct reads vs. streaming
- Platform-specific code uses `#[cfg(target_os = "...")]` attributes for Windows/macOS/Linux

### Frontend (desktop/src/)

**Entry Point**: `main.js` → `App.vue` → `Shell.vue` → `Sheet.vue` or `DiamondTable.vue`

`main.js` mounts the app and nothing else; `App.vue` renders the one view directly. Fuji has no router and no store library — shared state is an exported `ref` in a plain module. Read `architecture.md` before adding a view or a new home for state: it carries the layers, why each thing sits where it does, and the tests for when a router would earn its place.

**Key Components**:
- `Shell.vue` - Owns the window and none of the pixels: reads settings, sizes and reveals the window, records where the user puts it, holds the one listener for each window event and hands it to the view that is showing, and starts the performance log. Adding a table is one entry in its `tables` object
- `Sheet.vue` - The contact sheet: one folder seen whole, as a top-to-bottom scroll over a stack of cards
- `Card.vue` - A box of up to `card.images` thumbnails, all from one folder, handed to a flow; holds the register of flows
- `TagFlow.vue` - The first flow: plain img tags sized inside the chosen `thumbnail` square, wrapped like words, everything else left to the renderer
- `CanvasFlow.vue` - The other flow: reads a few images at a time, paints each into a canvas at the display's backing resolution, and releases the original, so the store holds nothing once a card is drawn
- `SquareFlow.vue` - The flow that replaces both, per `thumbnail-plan.md`: probes a card's files in one call, lays every box out at its final size, then fills canvases from the operating system where the platform's list allows and from the page where it does not, with GIF and SVG as img tiles; waits while the sheet is hidden
- `DiamondTable.vue` - One of fuji's tables, showing one image sized to an invisible diamond on an infinite pannable plane:
  - Handles the events the shell hands it, plus wheel, pointer, and double-click on its own element
  - Quiver system: maintains positioning/sizing state in three phases (A: desired, B: calculated styles, C: applied to DOM)
  - Shows the cache's own `<img>` element, adopted into its card — never one of its own pointed at the same picture
  - HUD overlays for help and information display
- `ComicTable.vue` - Another table, a stub. One image full width, read down a vertical scroll

**Model**:
- `model.js` - What the user is looking at, and no view owns it: the folder, the sort, the ordered list, the current path, and which of the sheet and a table is showing. The position is a path rather than an index, so changing the sort leaves the user on the same picture
- `AlphabetSort.js` - The first sort, and the plainest: javascript's own `sort()`. A sort returns the order rather than a comparator, so a shuffle can be one too

**Image layer**:
- `cache.js` - A store, not a strategy: `cacheNeed(path, holder)` and `cacheRelease(path, holder)` with labelled reference counts, and `cacheNeed(path, holder, {decode: false})` for a caller that wants the bytes and the url without an element. Holds a blob, one object url, and a decoded `<img>` per path. No queue, no eviction policy, nothing freed except on command
- `flipCache.js` - The diamond table's policy over that store: hold a window of `flip.back` and `flip.forward` images around the current one, release what falls out
- `log.js` - The performance log, off unless `log.record` says otherwise. Records every load and every flip, touches no disk during the session, and hands rows to Rust to write at exit
- `settings.js` - `fuji.toml`: one schema is the only place a setting is defined, and the file repairs itself on every launch

**JavaScript Modules**:
- `disk.js` - Thin wrapper exposing Rust commands to JavaScript:
  - `diskRead(path)`, `diskWrite(path, data)`, `diskReadDir(path)`, `diskStat(path)`, `diskCopy(source, destination)`

- `desktop.js` - Exposes the exit-write commands:
  - `desktopExitHold(path, text)`

- `panel.js` - Exposes hardware resolution command:
  - `panelResolution()`

- `thumbnail.js` - Exposes the operating system thumbnailer:
  - `thumbnailRender(path, maximum, gamut)` - One ArrayBuffer, header then pixels
  - `thumbnailUnpack(buffer)` - `{width, height, pixels}` shaped for `new ImageData()`

- `library.js` - Pure utility functions:
  - `xy(a, o, b)` - Vector math for {x, y} arrows (add, subtract, multiply, divide, compare)
  - `forwardize(path)` / `backize(path)` - Path normalization for cross-platform compatibility
  - `listSiblings(path)` - List all image files in same directory
  - `revealWindow(rect)` - Size the hidden window and show it; the window is created invisible so it never appears at one size and jumps
  - `readAndRenderImage(img, path)` - Load a file into an img element as a data url; the retired experiment components are its only callers
  - `screenToViewport()` - Calculate viewport position accounting for CSS/backing/physical pixels
  - `sayGroupDigits(n)`, `saySize4(n)` - Format numbers for display

**Key Patterns**:
- All paths are "forwardized" on entry (backslashes → forward slashes) and "backized" for Windows display
- Images reach the screen as: disk → Rust bytes → Blob → object url → `<img>` → `decode()`, held by `cache.js`, and the url is kept until the entry is freed
- A table shows the store's own element rather than pointing one of its own at the same picture, which was measured to cost the whole decode again
- A flip shows first and asks the store for anything new last, because a read or decode started before the paint blocks the frame it was meant to help. `DiamondTable.vue` carries the essay
- A thumbnail is a canvas fuji sized, its pixels from the operating system through `thumbnail.rs` where the platform's allow list permits and from the page where not; a GIF or an SVG is an img. `thumbnail-plan.md` is the plan and `SquareFlow.vue` is it built
- The "quiver" system separates state (A), calculation (B), and rendering (C) for efficient DOM updates

### Styling

- Tailwind CSS 4.x with Vite plugin
- Custom classes defined in `<style scoped>` sections:
  - `.myDots` - Repeating dot background pattern
  - `.myHud` - Semi-transparent overlay styling
  - `.myDry` - Disables pointer events and text selection
  - `.myWillChangeTransform` - Performance hint for animations

## Build Output Locations

**macOS**:
```
./desktop/src-tauri/target/release/bundle/macos/Fuji.app
./desktop/src-tauri/target/release/bundle/dmg/Fuji_0.1.0_aarch64.dmg
```

**Windows**:
```
./desktop/src-tauri/target/release/fuji.exe
./desktop/src-tauri/target/release/bundle/msi/Fuji_0.1.0_x64_en-US.msi
./desktop/src-tauri/target/release/bundle/nsis/Fuji_0.1.0_x64-setup.exe
```

`fuji.exe` is the application itself — the same binary both installers wrap and install. `pnpm win` launches it in place, without installing.

## Path Handling

Always use the path normalization functions from `library.js`:
- Call `forwardize(path)` on all paths entering the system (e.g., from drag-drop events)
- Use forward slashes internally throughout the codebase
- Call `backize(path)` only when displaying paths to Windows users in the UI

## Adding New Rust Commands

1. Define function in appropriate module (e.g., `disk.rs`) with `#[tauri::command]` attribute
2. Add to `generate_handler![]` in `lib.rs::run()`
3. Create JavaScript wrapper in corresponding JS file (e.g., `disk.js`)
4. Import and use in Vue components

## Platform-Specific Code

When writing platform-specific Rust code:
- Use `#[cfg(target_os = "windows")]`, `#[cfg(target_os = "macos")]`, `#[cfg(target_os = "linux")]`
- Provide fallback implementation with `#[cfg(not(any(...)))]`
- See `panel.rs` for examples of Windows (Win32), macOS (FFI), and Linux (xrandr) implementations
- To type-check windows-only code from a mac: `cargo check --target x86_64-pc-windows-msvc` fails inside tauri's build script (tauri-winres wants llvm-rc), so copy the module into a scratch crate without tauri and check that against the target instead
