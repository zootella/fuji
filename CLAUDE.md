# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fuji is a multimedia file manager designed with privacy and precision in mind. It's a Tauri desktop application built with:
- **Backend**: Rust (src-tauri/)
- **Frontend**: Vue 3 + JavaScript (src/)
- **Build Tools**: Vite, Tailwind CSS
- **Package Manager**: pnpm

The application displays images in an infinite pannable/zoomable space with keyboard navigation, drag-and-drop support, and full-screen mode.

## Development Commands

### Setup
```bash
pnpm install
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

### Clean Build Environment
There are deliberately no cleanup scripts. The old `wash`/`upgrade-wash` pair were yarn-classic-era crutches — that ecosystem needed frequent clean reinstalls; pnpm's store does not. If a genuine mess ever needs clearing, delete `dist`, `node_modules`, or `src-tauri/target` by hand — and never delete the tracked lockfiles: pnpm-lock.yaml and Cargo.lock serve both mac and windows, and removing them to fix a problem is the anti-pattern that motivated the pnpm switch.

## Architecture

### Rust Backend (src-tauri/src/)

**Entry Point**: `main.rs` → `lib.rs::run()`

**Key Modules**:
- `disk.rs` - File I/O commands for JavaScript to invoke:
  - `disk_readdir()` - List directory contents (POSIX-like readdir)
  - `disk_stat()` - Get file metadata (POSIX-like stat)
  - `disk_read()` - Read entire file into memory
  - `disk_copy()` - Efficient file copying using kernel-space operations

- `panel.rs` - Hardware display resolution detection:
  - `panel_resolution()` - Returns physical pixel dimensions via platform-specific APIs
  - Platform implementations for Windows (Win32), macOS (CoreGraphics), and Linux (xrandr)

**Command Registration**: All Rust functions exposed to JavaScript must be registered in `lib.rs::run()` using `tauri::generate_handler![]`

**Important Architecture Notes**:
- File I/O uses synchronous operations; `disk_read()` loads entire files into memory (suitable for images, not large files)
- Comments in `disk.rs` extensively document memory efficiency tradeoffs between direct reads vs. streaming
- Platform-specific code uses `#[cfg(target_os = "...")]` attributes for Windows/macOS/Linux

### Frontend (src/)

**Entry Point**: `main.js` → `App.vue` → `Shell.vue` → `DiamondTable.vue`

`main.js` mounts the app and nothing else; `App.vue` renders the one view directly. Fuji has no router and no store library — shared state is an exported `ref` in a plain module. Read `architecture.md` before adding a view or a new home for state: it carries the layers, why each thing sits where it does, and the tests for when a router would earn its place.

**Key Components**:
- `DiamondTable.vue` - One of fuji's tables, showing a single image; the shell owns the window and hands it events:
  - Event handling (keyboard, mouse, wheel, drag-drop, resize)
  - Triad pattern: maintains 3 image elements (prev, current, next) for smooth navigation
  - Quiver system: maintains positioning/sizing state in three phases (A: desired, B: calculated styles, C: applied to DOM)
  - HUD overlays for help and information display

**JavaScript Modules**:
- `disk.js` - Thin wrapper exposing Rust commands to JavaScript:
  - `diskRead(path)`, `diskReadDir(path)`, `diskStat(path)`, `diskCopy(source, destination)`

- `panel.js` - Exposes hardware resolution command:
  - `panelResolution()`

- `library.js` - Pure utility functions:
  - `xy(a, o, b)` - Vector math for {x, y} arrows (add, subtract, multiply, divide, compare)
  - `forwardize(path)` / `backize(path)` - Path normalization for cross-platform compatibility
  - `listSiblings(path)` - List all image files in same directory
  - `readAndRenderImage(img, path)` - Load image file and decode into img element
  - `screenToViewport()` - Calculate viewport position accounting for CSS/backing/physical pixels
  - `sayGroupDigits(n)`, `saySize4(n)` - Format numbers for display

**Key Patterns**:
- All paths are "forwardized" on entry (backslashes → forward slashes) and "backized" for Windows display
- Images are loaded as data URLs via: disk → Rust bytes → Blob → data URL → img.src → decode()
- The "triad" maintains 3 img tags (prev/here/next) to enable instant flipping without loading delays
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
./src-tauri/target/release/bundle/macos/Fuji.app
./src-tauri/target/release/bundle/dmg/Fuji_0.1.0_aarch64.dmg
```

**Windows**:
```
./src-tauri/target/release/fuji.exe
./src-tauri/target/release/bundle/msi/Fuji_0.1.0_x64_en-US.msi
./src-tauri/target/release/bundle/nsis/Fuji_0.1.0_x64-setup.exe
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
