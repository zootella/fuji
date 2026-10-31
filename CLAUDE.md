# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**A new session reads the constitution and its amendments before anything else, and reads each one whole.** They are the project's rules for the conversation, the planning documents, the code and the site.

## Project Overview

Fuji is a multimedia file manager designed with privacy and precision in mind. It's a Tauri desktop application built with:
- **Backend**: Rust (`desktop/src-tauri/`)
- **Frontend**: Vue 3 + JavaScript (`desktop/src/`)
- **Build Tools**: Vite, Tailwind CSS
- **Package Manager**: pnpm

The application displays images in an infinite pannable/zoomable space with keyboard navigation, drag-and-drop support, and full-screen mode.

### The repository is a pnpm monorepo

The application lives in the `desktop` workspace. The planning documents and the repository's own files stay at the root, where they describe the project rather than belonging to one part of it. `pnpm-workspace.yaml` names the workspaces, and the rule is that a directory holding a `package.json` is one — `notes/`, which holds the raw material the planning documents were written from, has none and is therefore just a folder. `site` is the VitePress website and documentation for fujidesktop.app, built to static files that our own reverse proxy serves. `linux` builds fuji's linux packages in docker containers on the mac — its one dependency is ergo, for how its output says a size, and everything else it does is drive containers whose toolchains live inside them. `apple` is the mac release through Apple: everything that depends on Apple's developer program lives there, so `desktop` needs none of it. `steel` is fuji's safe Rust, the scout, its reader of a file's first bytes, and the walker of a folder tree, a crate fuji's Rust depends on by path; its `package.json` has no dependencies and names its cargo commands. `copper` is fuji's Rust that needs neither Tauri nor a window and cannot be safe Rust, every call into C and the operating system's libraries away from a window, which fuji's Rust also depends on by path. The Rust crates are one Cargo workspace: the root `Cargo.toml` names them, one `Cargo.lock` at the root pins every version for all of them, and `.cargo/config.toml` there keeps the target folder at `desktop/src-tauri/target`, where the pipeline expects it.

The root has no scripts, deliberately. `pnpm install` runs there and installs every workspace; everything else runs from inside the workspace it belongs to, so `cd desktop` comes first. **Throughout this document a path written `src/` or `src-tauri/` is relative to `desktop/`**, which is how the code refers to itself; only paths written from the root, like the build outputs below, carry the `desktop/` prefix.

**The planning documents live at the root under short names, and are found by name**: grep for the subject, or ask git which ones changed most recently. Read `structure.md` and `architecture.md` before changing anything structural.

## Development Commands

### Setup
```bash
pnpm install      # at the repository root; installs every workspace
cd desktop        # every command below runs from the workspace, not the root
```

### The Desktop Workspace
```bash
pnpm local        # run Fuji here, in development mode with hot reload
pnpm compile      # build the binary in release mode, and stop there
pnpm installer    # build the installer, all the way through the app to the dmg or the exe
pnpm reveal       # open the file manager on that installer, to run it as a person would
pnpm hash         # stage and hash what is already built, building nothing
pnpm upload       # send what is already staged to the production server
pnpm installer-reveal  # the installer and then reveal, to try a build on this machine without publishing it
pnpm installer-hash-upload-reveal  # the four in a row, stopping at the first that fails, for a build you're sure of
pnpm certificate  # find, make, or import the Mac's signing identity
pnpm notarize     # build the installer signed with the Developer ID and notarized by Apple; the apple workspace below
pnpm notarize-hash-upload-reveal  # the same four with notarize first, the Mac's release in a row
pnpm library      # build the media library from its pinned release sources, on the Mac and on Windows, and write its entry in the ledger; nothing elsewhere
```
**Each command stops where its name says**, which is the whole point of naming them this way: `compile` never makes an installer, `installer` never hashes, `hash` never builds, `upload` never builds. Publishing is `installer`, `hash`, `upload`, then a commit, since the sidecars are tracked. That is how Windows publishes, and how the Mac publishes only as the fallback; a Mac release is `notarize`, `hash`, `upload`, through the apple workspace below. The two chained names that publish run each of those in a row with `reveal` last, stopping at the first step that fails, for a build you're sure of: `installer-hash-upload-reveal` on Windows, and `notarize-hash-upload-reveal` on the Mac, since `notarize` refuses on any other machine. `installer` and `notarize` both leave their dmg in the bundle folder, and `hash` stages whichever was made last. A name means the same thing on every machine while doing different work underneath, so nothing has to be remembered per platform.

Roughly what a release build costs, so a long one does not read as a hang: on the Mac mini about 20 seconds when only the frontend changed and a minute or so when Rust has to compile again; on the Windows 10 box 2m52s cold and about 45 seconds warm, and `pnpm installer` adds about six seconds there for the setup program. The release profile shares nothing with the debug profile `pnpm local` uses, so the first release build after a stretch of dev work compiles everything over again.

### Frontend Only (for rapid UI iteration)
```bash
pnpm dev          # Run Vite dev server without Tauri
pnpm vite-build   # Build frontend only
```
These two are also Tauri's own before-commands, named in `tauri.conf.json`, which is why they keep those names instead of joining the list above.

### The Linux Workspace

```bash
cd linux
pnpm library      the media library for both architectures, into linux/cache and the ledger
pnpm build        the four linux packages, from the library in linux/cache
pnpm hash         stage them and write the sidecars, building nothing
pnpm upload       send what is staged to the production server
pnpm build-hash-upload  the three in a row, stopping at the first that fails, for a build you're sure of
pnpm walk         the sandbox process walked in containers of several distributions, from one debug build
```
**Six commands and no setup step**, because `build` and `library` bring their own toolchain images up to date every time — three seconds once they exist, and the reason nobody has to notice when a version inside a `Dockerfile` moves. Docker Desktop has to be installed and running; the first build on a new machine is much longer, since it makes three images first. `pnpm library` runs `desktop/libmpv/build.py` in each tauri container, about three minutes on arm64 and twelve on the emulated amd64, and keeps the result in `linux/cache`; `pnpm build` takes the library from there, checked by the recipe's hash and the library's, refuses without one, refuses a library that needs a distribution package the deb's or the rpm's dependency line in `tauri.linux.conf.json` does not declare, and runs the arm64 and amd64 containers at the same time, the lines of each prefixed with its architecture and a failure in one stopping the other, in about ten minutes, 10m34s on the Mac mini on 2026-10-20; the tauri image carries the recipe's tools, meson from PyPI at a pinned version since Debian 12's is too old for mpv, and the headers for the handful of libraries the Linux library shares with the distribution. `build-distro`, `build-flatpak`, `build-images` and `stage` sit underneath for factoring and are rarely typed.

`linux/README.md` is the guide — the commands, and every filename on the way through. It makes four packages — a `.deb` for arm64, a `.deb` and an `.rpm` for x86_64, and a `.flatpak` for x86_64. An image is the toolchain and a container is one build: nothing survives between runs but the compiled media library in `linux/cache`, which `pnpm library` compiles and `pnpm build` uses, kept under the recipe's hash and a hash of the image's layers and checked against both before it is used, so every build starts from the same known state, which is worth more than speed at a few releases a year, and the twelve minutes of FFmpeg under emulation are paid once per recipe.

**The base image is `debian:12-slim` and that is load-bearing.** A binary is compatible with its build machine's glibc and every later one, never an earlier one, so the base sets a floor on who can run the result. Debian 12's 2.36 reaches Ubuntu 24.04 LTS, Mint 22.x, Fedora 40 and up, and both generations of Raspberry Pi OS — verified by installing the built packages on clean Debian, Ubuntu 24.04 and Fedora images. Debian 13 would have moved that floor to 2.41 and shut out the current Ubuntu LTS.

### The Apple Workspace

```bash
cd apple
pnpm waiting-room # build, notarize, hash and upload the mac release, a key each
```
**The published dmg is signed with a Developer ID and notarized, and `pnpm installer` still signs with Fuji Desktop, which is the fallback** for a release that has to go out without Apple. `apple/README.md` is the guide: the keys, the statuses, the folder each build is kept in, and setting up a Mac. `apple/notarize.js` does the work, and `pnpm notarize` in desktop is its build and notarize without the waiting room, leaving the notarized dmg in the bundle folder where `pnpm installer` leaves its own.

An application is one signed build, kept in a folder of its own under `apple/applications/`, gitignored. **Where each stands is read every time from things that outlast the waiting room**: its folder, Apple's history, which files every submission under its application's name, and the dmg sidecars staged and live. So quitting during a notarization and coming back is safe, and n never sends the same thing twice. Every step is also a line in the build log as it starts and as it ends, and the waiting room looks at that every second, so it shows a `pnpm notarize` in another terminal as it happens. `log.txt` in each folder is that application's history, its steps' lines with what every program printed between them.

The identity is named by its kind, `Developer ID Application`, rather than by the certificate's holder, and the API key's three values live in `.env`, so no tracked file carries a legal name, a team, or a key ID.

### The Site Workspace
```bash
cd site
pnpm local        # VitePress dev server
pnpm build        # Static files into docs/.vitepress/dist/
pnpm upload       # Build, then ship dist/ to the server
```
`upload` means the installer in `desktop` and the site in `site`, deliberately — one word, and each workspace ships what it made. The site build never learns a hash, because the download page fetches each sidecar at runtime.

**The dev server answers a sidecar from this machine first and from production second**, which is a small Vite plugin plus a proxy in `docs/.vitepress/config.js`. The plugin reads `desktop/release/` and `linux/release/` at request time and serves whatever is staged there; anything it does not find falls through to the proxy and comes from fujidesktop.app. So `pnpm local` shows the packages you are about to publish alongside the ones already live — the exe, say, which is built on the Windows box and arrives from the server. A response carries `x-fuji-sidecar` when it came from this machine, so the two can be told apart without guessing.

**The site's settings are in `docs/.vitepress/config.js`, each with its reason beside it.** Search is VitePress's local provider, an index built at build time and searched in the browser, with no outside service. The sitemap and each page's last-updated date are made at build time, the date from git, so the build has to run inside the checkout. Every page carries a link to its source on GitHub, and Open Graph tags from a hook in the config, so a pasted link shows a card. Two theme components are placed by the layout rather than by markdown: `LastUpdated.vue`, swapped in for VitePress's own through an alias in the config so the date reads one way everywhere, the download page's way, and `CreativeCommons.vue`, the license line under every article, which draws Creative Commons' two marks in the page so they take the line's ink. The stylesheet's rules for blockquotes and for the foot of a page carry their reasoning as comments, as every rule there does.

That is the third arrangement, and the two it replaced are worth knowing. **Copying was first**, a `pnpm fixtures` script that put this machine's sidecars in `docs/public/` — which worked, but one left behind is baked into a build and served from the site's own directory, shadowing the real file and pinning the page to a stale hash. Those copies are gitignored, so a machine that ran it may still hold some and no other machine can tell; `upload-site` refuses to ship a `fuji.*.json` it finds in the build and names what to delete, which is the guard that script's `clear` mode used to be. **Proxying alone was second**, honest about what was live and unable to show you your own staged work at all. Neither the plugin nor the proxy can bake anything into a build, because `vitepress build` never runs `configureServer`.

### The Steel Workspace
```bash
cd steel
pnpm test         # cargo test: the scout's tests, pinned to the files in samples/, the walker's and find's on trees they build, the disk's in a folder of their own, the fits' against the numbers fit.js gives, the reads', and the lock's on files made in memory
pnpm fuzz         # libFuzzer on the reader, seeded from samples/, until stopped; pnpm fuzz -- -max_total_time=60 runs a minute
pnpm fit-table    # write fit-table.txt, fitSize from fit.js over a grid, which the fits' test reads; run it when the arithmetic changes in both files at once
```
`steel/README.md` is the guide: what the crate is, its files, and how fuji uses it.

### The Copper Workspace
```bash
cd copper
pnpm test         # cargo test: the crate alone, with nothing of Tauri compiled; the registry's test runs only on Windows, and the Mac's launch services' only there
pnpm walk         # the sandbox process walked on this machine through a built fuji, each step saying what it saw; pnpm walk crash runs one step
```
`copper/README.md` is the guide.

### The Root Script

**One file at the monorepo root, `scripts.js`, holds the whole build pipeline**, reached by a verb: `reveal`, `hash`, `upload-installer`, `upload-site`, `icons-collect`, `certificate`. Nothing runs it directly — the package.json scripts above are the names a person types.

**Its one companion, `build-log.js` beside it, keeps the build log, `build-log.txt` at the root and gitignored**: a line for every step any workspace takes on this machine, as it starts and as it ends, with the commit the checkout was at, structured fields first and words for people after, which the essay there describes. It also says times one way for the whole pipeline, like `2026-Oct-9 Fri 2:45 PM`, while a time that's kept is UTC, in a form of ergo's that a person reads and a program reads back. It is a file of its own so that `dmg.js`, `win-setup.js`, `linux/build.js`, `scripts.js` and the apple workspace can all import it, while `scripts.js` stays something only ever run and never imported. It runs nothing, follows the same two rules below, and is the one other file at the root that is code.

It was four scripts across both workspaces until September 2026; the essay at the top of `scripts.js` says why they became one. The `platforms` table there now holds the bundle folder, the filename suffix, the published name and the file-manager command together, and is the only place any of them is said.

Two rules keep it workable. **It takes packages from the root `package.json`**, like any script, because the pipeline should be the shortest and plainest code in the repository and a package that already does the work keeps it that way; `pnpm install` at the root installs them along with every workspace's, and none reaches a workspace's bundle, which holds only what that workspace's own code imports. And **every path is built from the file's own location**, never from the working directory, because every workspace calls it, each from its own folder — which is a thing the old scripts each answered differently and only got away with because pnpm happened to run them from the right place.

### Regenerate the Icons
```bash
pnpm icons        # rebuild every platform's icons from the sources in src-tauri/icons
```
One run writes the macOS `.icns`, the Windows `.ico`, the Linux PNGs, the Start menu tile and, on a Mac with Xcode, the Tahoe icon's `Assets.car`, so no platform needs its own run. These are committed artifacts, which means a Tauri CLI upgrade does not refresh them — re-run this after one. `src-tauri/icons/README.md` has every source, every file a run makes, how each platform's build takes them, and the sizes.

**Expect two modified files afterwards, both `.icns`, and a third, `mac/Assets.car`, on a Mac with Xcode.** `tauri icon` writes that container's records in an unstable order, so `icon.icns` and `mac/icon.icns` come back reordered on every run, on every machine, with every image byte-identical — verified record by record. `actool` writes a new timestamp and new generated names into `Assets.car` on every run. Committing them is lossless. Any *other* file appearing in `git status` after a run is a real change worth looking at.

### Clean Build Environment
There are deliberately no cleanup scripts. The old `wash`/`upgrade-wash` pair were yarn-classic-era crutches — that ecosystem needed frequent clean reinstalls; pnpm's store does not. If a genuine mess ever needs clearing, delete `desktop/dist`, `node_modules`, or `desktop/src-tauri/target` by hand — and never delete the tracked lockfiles: pnpm-lock.yaml and Cargo.lock serve both mac and windows, and removing them to fix a problem is the anti-pattern that motivated the pnpm switch.

## Architecture

**Code lives in four places, and `architecture.md` at the root says what each is, what it demands of the code in it, and how to decide where something new goes.** Plastic is the page in `desktop/src`, which is the application and decides everything: state, orchestration, what to ask of the machine and when. Brass is the crate Tauri builds the application from in `desktop/src-tauri`, holding only what is tied to a window, the event loop or Tauri's own types; its handler list is the whole of what the page may ask and so the whole of fuji's attack surface. Copper is the crate for every call into C and the operating system's libraries away from a window, where a search for unsafe is the whole list. Steel is the safe crate, which the compiler holds to safe Rust and which reads the bytes nobody vouched for. The metals never decide, hold application state or sequence work, and none knows it is part of fuji; work goes below plastic for speed, for reaching the operating system or a library, or for a security wall, and for nothing else. The sorting is done: steel holds the scout, the walker, the reads, the disk, find, the fits and the lock; copper holds the media library, the inventory's loop, the sandbox process's start and walls, the thumbnailers, the registry, launch services, the display, the memory counters and the process's start; brass holds the commands, the windows, the menus, the dock, the trackpad, the dialogs, the log, the players with their render contexts, and the web engine's process ids.

### Rust Backend (desktop/src-tauri/src/)

**Entry point**: `main.rs` → `lib.rs::run()`. **The handler list in `lib.rs` is the complete list of commands**, and the whole of what the page can ask Rust to do. No plugin is granted to the page: `capabilities/default.json` holds Tauri's core set and the calls the page makes on its own window, and anything past the window, an address opened in the browser or the system's file dialog, is a command here. Each module opens with an essay on what it offers and why, and each command carries a doc comment, so the code is where to read what a command takes and answers; this file does not repeat it.

The modules, by what each one answers: the disk (`disk.rs`), the folder tree in find's order, many folders per call (`find.rs`), fuji's windows and the native rectangle a video draws in (`window.rs`), the files the operating system handed fuji (`open.rs`), the display (`panel.rs`), where this copy of the program is (`paths.rs`), the Windows registry (`registry.rs`), other programs, handed a file or an address the way a double-click would (`process.rs`), the system's own dialog boxes (`dialog.rs`), the Mac's record of which application opens which kind of file (`launch.rs`), the operating system's thumbnailer (`thumbnail.rs`), the machine's memory and what fuji's processes take of it (`memory.rs`), how long this process has been running (`uptime.rs`), the media library, libmpv, declared by hand, which plays a file and answers what one holds (`media.rs`), a file's first bytes, read in safe Rust for what they are and what their format keeps in its header by the scout in steel, the safe crate at the root (`scout.rs`), an inventory of a folder's whole tree, every file in it hashed and scouted into a text file of lines (`inventory.rs`), the log and the text written at exit (`log.rs`, `desktop.rs`), and the Mac's menu bar, dock and trackpad (`menu.rs`, `dock.rs`, `touch.rs`).

**Important Architecture Notes**:
- File I/O uses synchronous `std::fs` calls inside `#[command(async)]`, so they run on Tauri's thread pool rather than the thread that runs the window; `disk_read()` loads entire files into memory (suitable for images, not large files). `lib.rs` has the rule for which commands are async
- `disk.rs` holds no guard on paths, deliberately: its opening essay is the contract, and the walls are outside the file, argued once in `lib.rs`
- Platform-specific code uses `#[cfg(target_os = "...")]` attributes for Windows/macOS/Linux

### Frontend (desktop/src/)

**Entry point**: `main.js` → `App.vue` → `Shell.vue` for a browsing window, which shows one view at a time, or `VideoWindow.vue` for a video window.

`main.js` titles the page and mounts the app, and does nothing else; `App.vue` picks the window's root once, as the page mounts, from the note the window was made with and the files it was opened with. Fuji has no router and no store library — shared state is an exported `ref` in a plain module. Read `architecture.md` before adding a view, a part of one, or a new home for state: it carries the layers, the views, why each thing sits where it does, and the tests for when a router would earn its place.

**Where to read before changing something.** Every component in `src/components/` and every module in `src/` opens with a comment saying what it is, and the ones that carry a subject open with an essay on it:
- `Shell.vue` — a browsing window, which view is showing, and fuji's two fullscreens
- `VideoWindow.vue` — a video window, a native window of its own beside a browsing window or without one
- `TestFlow.vue` — how a path becomes a thumbnail
- `DiamondTable.vue` — the quiver, and why a flip shows first and loads last
- `model.js`, `settings.js`, `cache.js` — what the user is looking at, the one schema for `fuji.toml`, and the store of pictures
- `pager.js` — every image on the volume as one list, never built, and the sheet's page as a window onto it
- `associate.js` — file associations, starting from the values the design follows

**Each Rust module with commands has a JavaScript file of the same name in `src/`** that wraps them for the page, one function per command, forwardizing a path where it crosses. `components/library.js` holds the small pure helpers, `forwardize`, `backize` and `xy` among them. `fileTypes.js` is the one table of every extension fuji knows and what each part of fuji does with it on each platform.

**Key Patterns**:
- All paths are "forwardized" on entry (backslashes → forward slashes) and "backized" for Windows display
- Images reach the screen as: disk → Rust bytes → Blob → object url → `<img>` → `decode()`, held by `cache.js`, and the url is kept until the entry is freed. A picture only the operating system's imaging library reads on this platform, a TIFF on Windows or a Photoshop file on the Mac, reaches it instead as the library's pixels at the picture's own size, painted once onto a canvas the store holds in the img's place; `fileTypesTable` in `fileTypes.js` says which
- A table shows the store's own element rather than pointing one of its own at the same picture, which was measured to cost the whole decode again
- A flip shows first and asks the store for anything new last, because a read or decode started before the paint blocks the frame it was meant to help. `DiamondTable.vue` carries the essay
- A thumbnail is a canvas fuji sized, its pixels from the operating system through `thumbnail.rs` where the platform's allow list permits and from the page where not; a GIF or an SVG is an img. `TestFlow.vue` is the code, and the Image Fidelity page on the site is why it is shaped that way
- A canvas is sized to its box in backing pixels — whole CSS pixels times `devicePixelRatio` — and never to whatever size the thumbnail came back at. A canvas even one backing pixel short of its box is resampled by the compositor on every row, and a fractional CSS size does not fix it. `flowSnap` and `flowEdge` in `TestFlow.vue` hold this; the Image Fidelity page on the site has the measurement
- The "quiver" system separates state (A), calculation (B), and rendering (C) for efficient DOM updates

### Styling

**The look and the CSS are decided together, and simple, correct CSS wins a tie.** When a look would take complex or brittle styles, say so and offer a look CSS gives cleanly; the essay at the top of `style.css`, fuji's one stylesheet, is the whole of it.

- Tailwind CSS 4.x with Vite plugin, kept for its reset, which evens out three engines' defaults, and for layout that reads in the template
- **Tailwind classes in the template** for layout and one-off values; **a named class in CSS** when other code finds the element by it — `myTile` for the gamma filter, `myImage` reached with `:deep()` because the template did not make that element — or when the style is one utilities say badly, like a gradient pattern or a layered shadow. A named class starts with `my`, so none can ever be mistaken for a utility, and lives in its component's `<style scoped>` unless several components share it, when it goes in `style.css`
- **Nothing resizes a picture unless fuji says so.** `style.css` cancels the reset's `img { max-width: 100% }`, which shrinks and resamples any picture wider than its container without asking, so every img sizes itself
- **Two kinds of text, and whole pixels between them.** The root is set in the platform's own menu font, which is what rem measures from and what fuji's future toolbars and menus inherit; fuji's own character-mode type, every hud and panel today, is one rule in `style.css` in CSS pixels. One setting chooses the faces for both: system, the platform's own; bundled, the two fuji carries, Inter and IBM Plex Mono, which look the same everywhere; or retro, Verdana as Windows and the Mac install it, with Plex again. The root never changes with the choice; Inter is 13 css pixels and Verdana 12, the same on every platform, each on the root's line. Those two short lists replace Tailwind's own, so every element names the same ones. Tailwind's spacing is pinned to whole CSS pixels, so no platform's text size moves a box
- **Light and dark from one palette.** Every color fuji paints is named in `style.css`, used by its name and never as a hex value, and the window's theme, set from one setting, picks between light and dark. Every gray is Zed's, from One Light and One Dark; one accent, the user's, paints a list's selected row and a menu's row on every platform; and fuji's own colors are never gray. The essay above the palette says where each came from and how its name says so

## Build Output Locations

**macOS**:
```
./desktop/src-tauri/target/release/bundle/macos/Fuji.app
./desktop/src-tauri/target/release/bundle/dmg/Fuji_0.1.0_aarch64.dmg
```

**Windows**:
```
./desktop/src-tauri/target/release/fuji.exe
./desktop/src-tauri/target/release/bundle/win-setup/Fuji_0.1.0_x64-setup.exe
```

**Linux**:
```
./linux/release/Fuji_0.1.0_arm64.deb
./linux/release/Fuji_0.1.0_amd64.deb
./linux/release/Fuji-0.1.0-1.x86_64.rpm
./linux/release/Fuji_0.1.0_x86_64.flatpak
```

**The staged release**, written by `pnpm hash` on whichever machine built it. No published name carries a version, so a link anyone shares keeps pointing at the current build; every linux package names its architecture, because linux is where architectures multiply and a bare name would read as the default while being the rarer one:
```
./desktop/release/fuji.dmg              ./desktop/release/fuji.dmg.json
./desktop/release/fuji.exe              ./desktop/release/fuji.exe.json

./linux/release/fuji.arm64.deb          ./linux/release/fuji.arm64.deb.json
./linux/release/fuji.amd64.deb          ./linux/release/fuji.amd64.deb.json
./linux/release/fuji.x86_64.rpm         ./linux/release/fuji.x86_64.rpm.json
./linux/release/fuji.x86_64.flatpak     ./linux/release/fuji.x86_64.flatpak.json
```

**Two different files are named `fuji.exe`, and their sizes tell them apart at a glance.** `src-tauri/target/release/fuji.exe` is the application itself — the binary the installer carries, and the one that runs in place without installing. `release/fuji.exe` is the staged **installer**, a copy of `Fuji_0.1.0_x64-setup.exe` under its publishing name, and it is what the website offers for download. Measured on the Windows box 2026-10-03: the binary is 11,437,568 bytes and the installer 3,858,061, because the installer compresses what it carries.

`bundle.targets` names what a `tauri build` makes, rather than Tauri's default `"all"` — which also builds an NSIS installer and an `.msi` on Windows and an `.AppImage` beside the Debian package, none of which anything links to. One list serves every platform: a target that does not apply to the machine doing the build is skipped, and **the skipping is silent**, so a build producing one file is not evidence that anything went wrong.

The linux containers do not use that list. They pass `--bundles deb` or `--bundles deb,rpm` on the command line instead, which overrides it for that run only — so the linux specifics stay in the `linux` workspace and what the mac and windows builds are told to make never changed.

`bundle.macOS.signingIdentity` is `Fuji Desktop`, a self-signed certificate of fuji's own held in the login keychain of the Mac that publishes, which `pnpm certificate` finds, makes once, or imports from its backup. It is what `pnpm installer` signs with, and the fallback: the published dmg is signed with a Developer ID by the apple workspace, which overrides this identity with `APPLE_SIGNING_IDENTITY` for its own build and leaves the file alone. To Gatekeeper Fuji Desktop reads as an ad-hoc seal did, "could not verify" with an Open Anyway button rather than "damaged" with no way through; what it adds is a designated requirement every build meets, so a privacy permission the user grants survives an update. `hardenedRuntime` is off beside it, because the hardened runtime's library validation loads only libraries signed under the app's own Team ID, which a self-signed identity has none of, so a fallback build carrying libmpv's tree died at launch with it on; the apple workspace turns it back on for its own build, where the Developer ID gives the app and every library one Team ID and notarization requires it. The essay under the targets table in `scripts.js` says why it exists, and `apple/README.md` why both do. Windows and the linux containers ignore it. Every `pnpm installer` on the mac tries to notarize after signing, finds no credentials, and says so, which is expected, since notarizing is the apple workspace's job. `pnpm hash` and `pnpm upload` warn loudly at a dmg that is not notarized and never refuse one, because the fallback ships exactly that.

**Every platform carries libmpv as one library fuji builds itself, from one recipe.** `pnpm library` runs `desktop/libmpv/build.py`: the same release sources at the same versions on each platform, each fetched from its own project and checked against the hash written in the script, built static in order into one shared library with nothing in it from anywhere else, so the About section names the recipe's numbers and every package carries the same work. On Windows it is `libmpv-2.dll`, from twelve sources inside MSYS2's ucrt64 toolchain, which `libmpv/libmpv.js` installs and starts the script under, and MSYS2 itself is one `winget install --id MSYS2.MSYS2`; 48 MB, with Direct3D 11, both hardware decoders, AV1 and ICC profiles inside, which the installer's cabinet holds in about 16. On the Mac it is `libmpv.2.dylib`, arm64, from ten of the twelve, since OpenGL wants no shader compiler: VideoToolbox, CoreAudio and the OpenGL output inside, over Apple's own frameworks and nothing else, 25.7 MB, built in about two and a half minutes on the Mac mini, with Homebrew providing the tools alone, `brew install meson ninja pkgconf`, which the launcher names when one is missing, over Apple's clang and Swift. Both land untracked in `src-tauri/binaries/`, each named by a tracked platform configuration file Tauri merges on that platform alone: `tauri.windows.conf.json` as a resource beside the executable, and `tauri.macos.conf.json` as a framework, which Tauri copies into `Contents/Frameworks` and signs. The Mac binary links against the library in `binaries/`, whose install name is `@rpath/libmpv.2.dylib`, resolved by the rpath Tauri gives the app, so a `tauri build` before the recipe has run fails to link and says so. Mapping the library at launch costs nothing a collector could notice: warm launches to the first frame stayed within the run-to-run noise of launches without it, measured on the Mac mini on 2026-10-13 and the Windows box on 2026-10-12, which is why it is linked rather than loaded on the first play. On Linux the containers run the same recipe before Tauri, and the four packages carry `libmpv.so.2` as a resource under `usr/lib/Fuji`, found by the rpath `build.rs` gives the binary, with the distribution's fontconfig, X11, OpenGL and sound libraries named as dependencies in the tracked `tauri.linux.conf.json`; `media.rs` links it on all three. The essay atop `build.py` says what is in each library and what was left out. **Beside the library the recipe writes `libmpv.json`, the record of what it made**: the library's hash and size, the recipe's own hash, the tools, and every source with its hash. `pnpm hash` copies that record into the release's sidecar, so the tracked sidecars say which library each release carried and where every part of it came from, and `hash` refuses a library the recipe as checked out did not build, a library in `binaries/` that is not the one its record describes, or an installer built before the library was. The containers write the record beside the packages under each architecture's name, and the rpm and the flatpak take amd64's, which is the library they carry. **The ledger is four tracked files at the root, `library-mac.txt`, `library-win.txt`, `library-amd.txt` and `library-arm.txt`**: every run of the recipe that compiles anything appends the same record as an entry, so git says when each library was built and from what, and `hash` also refuses a library with no entry in its file.

**Fuji makes its own dmg, and Tauri makes only the app inside it.** `bundle.targets` leaves `dmg` out, and `pnpm installer` runs `desktop/dmg.js` after `tauri build`, which hands the layout to dmgbuild through `uvx` at a pinned version — so the Mac that builds the installer needs uv, and no other machine does. dmgbuild writes Finder's `.DS_Store` directly into an image Finder never sees, so a build steals no focus, takes about five seconds, and comes out the same whatever Finder is showing; the essay atop `dmg.js` says what this replaced. The window a Mac user meets is the jinbocho street scene from `src-tauri/dmg/jinbocho.jpg`, 800 CSS pixels square and shown whole, with Fuji.app and the Applications link centered in its two halves; the picture is a 1600-pixel JPEG tagged at 144 dpi, which is how one file serves an sRGB panel and a Retina one alike, and its source and every candidate that lost live outside the repository. The numbers in `dmg.js` were measured on the Sequoia Mac mini against a checkerboard in the picture's place.

**Fuji makes its own Windows installer too, and Tauri makes only the executable.** `bundle.targets` leaves NSIS out, and `pnpm installer` runs `desktop/win-setup/win-setup.js` last: it stages the executable and everything in `bundle.resources`, with `tauri.windows.conf.json` merged over `tauri.conf.json` the way Tauri merges it, packs them with Windows' own `makecab`, compiles `setup.c` with the product's names stamped in, and appends the cabinet, in about six seconds on the Windows box, forty with the media library's dll in the cabinet, using the C compiler from the Visual Studio Build Tools that Rust already needs. The setup program installs per user with no interface, is its own uninstaller, and never uninstalls first, so an install over an existing copy keeps every file type fuji registered and every choice the user saved in Windows' own screens. Every file in `win-setup/` but `registry.js` names no product and reads every name from `tauri.conf.json` and `Cargo.toml`, so no change to fuji needs a change there; the README's section The Windows installer says the rest.

`pnpm hash` copies the bundle out from under its versioned, architecture-specific name into `release/` under a stable publishing name, and writes the sidecar beside it from the bytes that landed. The rename happens here rather than at upload time, which is what lets the site side copy known filenames from a known path with no rules about versions or architectures. The installers stay out of git; the sidecars are committed, so history keeps a dated record of what hash each release had.

## Path Handling

Always use the path normalization functions from `library.js`:
- Call `forwardize(path)` on all paths entering the system (e.g., from drag-drop events)
- Use forward slashes internally throughout the codebase
- Call `backize(path)` only when displaying paths to Windows users in the UI

## Pixel Units

**Three units, and one name each.** The code, its comments, the planning documents and the site all use these three names and no others.

- **CSS** pixels are the page's unit and the user's: what CSS lays out in, what the user sizes things in, and what settings record. macOS calls them points and lists them in its display settings as the "looks like" resolution; Tauri calls them logical.
- **backing** pixels are the grid fuji paints into. On the Mac that is the backing bitmap. On Windows, which has no separate backing store, it is the display resolution. Tauri calls these physical, in `PhysicalSize` and `PhysicalPosition`, and so is every rectangle Win32 hands fuji, which is DPI aware.
- **panel** pixels are the display's own lights. On the Mac they are always the physical panel, and macOS resamples backing onto them whenever the two differ. On Windows they are the physical panel only while the display runs at its native resolution. Many users lower the resolution to make things bigger, because that setting is older and better known than "Change the size of text, apps, and other items"; the monitor or the graphics card then stretches backing onto the lights, invisibly to every API, and fuji neither detects that nor adjusts for it.

**A ratio always says which one it is**, because there are several. Backing per CSS is the one `devicePixelRatio` answers in the page, Tauri's `scale_factor` answers in Rust, and Windows calls its scale setting: only ever 1 or 2 on the Mac, and on Windows 1.25 at 125 percent, 1.5 at 150, and so on, so often fractional. Prose calls it by whichever of those names its reader already knows, or says it outright — each CSS pixel two backing pixels across — and never leaves a bare *the ratio* for the reader to resolve. Panel per backing is the Mac's resample. A picture's own pixels are not a screen unit at all, and a picture's size on screen is CSS per image pixel. In code, backing per CSS is held as `backingPerCss` in the page and `backing_per_css` in Rust, never as a bare `ratio` or `scale`, and any other ratio names both of its units where it is declared.

**Five words stay out of the unit names.** *Physical*, because Tauri means backing by it and this codebase used it for panel. *Display*, because the Mac's display settings list CSS resolutions and Windows' list backing ones. *Device pixel*, *native* and *hardware*, which were three more names for units that already have one.

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
