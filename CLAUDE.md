# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Working agreements

These are the rules a session works under here. They live in this file rather than in any one machine's memory, for the reason the rule about moving between machines gives.

**The user alone runs git commands that change anything.** A session uses git to look — `status`, `log`, `diff`, `show`, `blame` — and never to commit, add, push, checkout, restore, reset, merge, rebase, stash, or tag. Finish the work, leave it in the working tree, say plainly what changed and what a commit would cover, and hand over the command rather than running it. Every commit in this repository is the user's own, with a lowercase one-line subject and no body and no trailers, and it stays that way; a commit carrying a `Co-Authored-By` or a session link looks nothing like the rest of the history and has had to be rewritten once already. One consequence is worth stating on its own: **`git checkout <file>` is not an undo.** It discards every uncommitted change to that file, including work that has nothing to do with the mistake being fixed, and it has already cost a finished fix here once. Use the editing tools.

**A diff file is the user's too, and never a session's to write.** A session reads diffs freely — `git diff`, `git show`, and `diff.diff` at the root when it is there — and never creates, refreshes or overwrites one. That file is gitignored, and the user writes it when he wants a change set in front of him; a session that regenerates it has replaced his record with its own idea of what changed, which is the git rule above in a smaller form. And **"let's review the diff" means review the changes it describes**, never audit how it was made or whether it is current. Take the file as the statement of what to look at, and spend the whole turn on whether the code in it is complete and correct — which is the review that was wanted, and the one a session is actually useful for.

**The work moves between machines, and this repository is the only thing they share.** Fuji is cross-platform, and it is developed on several computers that pull and push through git rather than on one. A Mac mini is home base and where most of the work happens, though its display is old and sRGB. A MacBook Air is the Retina and Display P3 machine, visited when something has to be checked against a modern Apple panel. A Windows 10 box, itself old, a Linux desktop and a Raspberry Pi carry the desktop integration each of those systems needs.

**Two machines publish fuji, and linux is not one of them.** Windows builds and sends the exe. The mac sends the dmg it built natively and the four linux packages it built in docker containers, so every linux package comes from one machine against one base image and one lockfile. A linux box can still clone this repository and run `pnpm installer` in `desktop` to build for the machine it is sitting at — that is development and it works — but `scripts.js` refuses to stage or upload from there, and says why.

**So write for a reader who is somewhere else.** A Claude Code session keeps notes of its own, per project folder and per machine, and those do not travel: written on one computer they are invisible on the next, and the session that reads a file is almost never the session that wrote it. Weeks may have passed and many revisions may have landed in between. So anything worth knowing next time goes in a file here rather than in a note only one computer can see; a document says which machine a measurement came from, because the same code gives different numbers on different hardware; and it says what is settled against what is still assumed, since the reader cannot ask the session that found out. `contents.md` lists the design documents and says what each one owns, and separates them from letters, which are addressed to whoever comes next rather than settling a subject. `fidelity.md` is the worked example of all three habits.

**But a note to another machine has to earn its place.** Every change here touches code that runs where this session cannot look, so "untested on Windows" is true of nearly everything and is never by itself worth writing down. A letter or an open item that says only that turns the other computers into a queue of doubt: it costs attention on every visit, it is mostly noise, and it buries the one entry that actually mattered.

The bar is a **specific reason to believe something is likely to be wrong there**, and meeting it means naming the mechanism — a platform API that behaves differently, code behind a `#[cfg]` that has never been compiled for that target, an assumption read out of one platform's source and carried to another, a measurement that cannot be taken here. If no mechanism can be named, the work is simply done, and the next visit to that machine exercises it the way it exercises everything else.

Two habits lower the bar rather than clear it, and both come before writing anything down. **Exercise the other platform's path here wherever it is not gated** — `open_argv` runs on every platform, so launching the binary with a file as an argument on a Mac tests exactly what Explorer does on Windows. And **write for the general case so the other platform runs the same code in a degenerate form** rather than a branch of its own: per-window state with one window in the map needs no Windows testing that one-window-per-process state would have needed.

## Where the work ends up

**Everything this project learns has a permanent home, and a planning document is not one of them.** There are four. `style.md` governs the first three and is the authority on how to write them.

**The code, and the names in it.** The best home for a fact is a name that makes the fact obvious, and a shape that makes the wrong thing hard to write. Reach for that before reaching for prose.

**Comments.** At the end of a line, above a group of lines, above a function — the *why* that the *how* cannot show. They are dense here by design and `style.md` argues for that at length.

**Essays.** A `/* */` block in ordinary prose, for a mechanism that runs through several files where any one site reads as a contradiction on its own. The essay above `toggleFullscreen` in `DiamondTable.vue` is the worked example: fuji has two fullscreens on purpose, and every place that touches them looks like a mistake until you have read it.

**The site.** `site/docs/` is the fourth destination and the only one that is not code. It takes long-form writing where a subject carries real research or engineering — measurements, the alternatives weighed and why, pictures of the defect. `site/docs/thumbnail-pipeline.md` is the worked example: all of it is implemented in the code, but no comment could hold it in one place or with that depth. Use it rarely. Most work does not earn a page, and a page nobody needed is worse than none.

**These four must be short, correct, and well placed.** Everything below is scaffolding for getting there.

## Planning documents

**They live at the repository root, in markdown, and `contents.md` says what each one owns.** They are where a subject gets decided, and they are written for a reader on another machine who cannot ask the session that wrote them. They are scaffolding, not fixtures: their acreage is meant to be spent.

**A document has three acts, and which one it is in says how to edit it.**

**First act: a whiteboard.** Scraps, notes, half-formed requirements, design ideas, and questions — seeded as they arise, with far more asked than settled. This is where "we should look at the dock menu, is any of it free, and what would a Mac user expect?" goes the moment somebody says it. The document exists so that concern survives the conversation.

**Second act: research, results, and then a plan.** Work the questions and write the answers where the questions were. That usually rewrites the document completely, which is right — and nothing is lost, because the document is the record of what was asked. Then some code: a spike, an experiment in the field, enough to learn whether the ground is firm or the woods are dense. Somewhere in the middle of that, rewrite again, from exploration into a crisp plan of attack, and follow it incrementally. **That second rewrite usually makes the document shorter.** It grew while it was holding questions, possibilities and detailed measurements; those have now been spent on decisions, and all it has to carry is the plan.

**Third act: cleanup, canonization, minimization.** The work has landed. What is left is the loose ends — something else to test, something to decide, something to table, something to forward as a note to the machine that can answer it — and that usually means a little more code and another smoke test. Then the document is minimized. **Most of its acreage is simply deleted**, and what is worth keeping moves to one of the four destinations above. What remains in the file is only the part still open. `thumbnail-open.md` is what that boiling leaves behind, and `contents.md` records which documents became pages on the site instead.

**Roughly nineteen words in twenty go, and that is the expected shape rather than a failure.** The bulk is not being relocated — it is being spent. Almost none of it belongs in the code, because code has to stay tight and fast to read, and a comment earns its place only by making a reader faster. A planning document that has been "moved into comments" wholesale has ruined two things at once.

**But deleting is done with a check, thoroughly and every time, even though it usually finds nothing.** The test is never "this section is about menus, menus are done, delete it" — subject and staleness are different questions, and a finished subject can still be the only place some small thing is written down. Before a section goes, ask three questions of it.

*Is there a detail here that the code does not say and a comment should?* This is the common one, and it is small. Write the comment first, confirm it reads well where it sits, then delete the section.

*Is there a realistic corner case, an untested condition, or a concern nobody has met yet?* Move a note into a document that will survive, or up into a higher-level one that owns the subject, or out to the machine that can answer it. Then delete.

*Is there a path of research and exploration behind a decision, with enough substance that a reader would want the account?* That is the site's case. Propose a page, and delete only once it exists. The thumbnail work is the worked example: pages of measurements, alternatives and dead ends were removed from the planning documents, but only after `site/docs/thumbnail-pipeline.md` had a good account of them — informative and interesting to somebody arriving later, which is a different job from the notes that produced it.

**Delete the trail.** Not "first we thought this, so we tried that, but it did not work, and we were wrong about the other thing, which is how we got here." Only the working, finished implementation matters, nearly all of the time, and that lives in the code. Write what was decided and briefly why, where the work stands and briefly why, and what is next. A failed attempt earns its place only when the failure is the reason the design is what it is — `cache.md` keeps three of them because each names a constraint the store still has to meet.

**When there is genuinely too much to say, that is the site's job, not the planning document's.** A subject with real innovation behind it — where how we got here is worth an account with measurements and pictures — becomes a page in `site/docs/`. That is the one case where the long version survives, and it survives somewhere a reader would actually look for it.

**Keep them current as the work goes, and replace questions with answers.** A document that has learned something says the answer where it used to ask; it does not keep both. A finished item left sounding open costs a future session real time, because it reads as work to pick up.

**They are not precious the way code is.** A stray passive voice in a comment earns a correction, a review, and a push — the same sentence in a planning document is fine, and loose prose here costs nothing. What they do have to be is complete, correct, and verbose, because they are what a reader has instead of the session.

## Project Overview

Fuji is a multimedia file manager designed with privacy and precision in mind. It's a Tauri desktop application built with:
- **Backend**: Rust (`desktop/src-tauri/`)
- **Frontend**: Vue 3 + JavaScript (`desktop/src/`)
- **Build Tools**: Vite, Tailwind CSS
- **Package Manager**: pnpm

The application displays images in an infinite pannable/zoomable space with keyboard navigation, drag-and-drop support, and full-screen mode.

### The repository is a pnpm monorepo

The application lives in the `desktop` workspace. The planning documents and the repository's own files stay at the root, where they describe the project rather than belonging to one part of it. `pnpm-workspace.yaml` names the workspaces, and the rule is that a directory holding a `package.json` is one — `notes/`, which holds the raw material the planning documents were written from, has none and is therefore just a folder. The second workspace is `site`, the VitePress website and documentation for fujidesktop.app, built to static files that our own reverse proxy serves. The third is `linux`, which builds fuji's four linux packages in docker containers on the mac — it has no dependencies of its own, because everything it does is drive containers whose toolchains live inside them.

The root has no scripts, deliberately. `pnpm install` runs there and installs every workspace; everything else runs from inside the workspace it belongs to, so `cd desktop` comes first. **Throughout this document a path written `src/` or `src-tauri/` is relative to `desktop/`**, which is how the code refers to itself; only paths written from the root, like the build outputs below, carry the `desktop/` prefix.

**The planning documents are listed in `contents.md`**, which says what each one owns and which to read first. Read `structure.md` and `architecture.md` before changing anything structural, and `style.md` before the first edit.

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
pnpm installer    # build the installer, all the way through the app to the dmg
pnpm reveal       # open the file manager on that installer, to run it as a person would
pnpm hash         # stage and hash what is already built, building nothing
pnpm upload       # send what is already staged to the production server
```
**Each command stops where its name says**, which is the whole point of naming them this way: `compile` never makes an installer, `installer` never hashes, `hash` never builds, `upload` never builds. Publishing is `installer`, `hash`, `upload`, then a commit, since the sidecars are tracked. A name means the same thing on every machine while doing different work underneath, so nothing has to be remembered per platform.

Roughly what a release build costs, so a long one does not read as a hang: on the Mac mini about 20 seconds when only the frontend changed and a minute or so when Rust has to compile again; on the Windows 10 box 2m52s cold and about 45 seconds warm. The release profile shares nothing with the debug profile `pnpm local` uses, so the first release build after a stretch of dev work compiles everything over again.

**A session builds when there is a reason to, and picks the smallest build that gives it.** Not every turn, and not by habit at the end of a change.

**To know the code is valid**, `cargo check` and `pnpm vite-build` are the cheap answers and usually enough. **To prove the release profile compiles and links**, `pnpm compile` and nothing more — no app folder, no dmg. **To let the user smoke test something that has to be installed**, `pnpm installer`; they then run `pnpm reveal` and drag it in themselves, because installing is theirs. **Otherwise build nothing.**

Building the installer every turn is the habit to avoid: it is the slowest thing here, it produces a file nobody asked for, and it says nothing that `cargo check` did not already say.

### Frontend Only (for rapid UI iteration)
```bash
pnpm dev          # Run Vite dev server without Tauri
pnpm vite-build   # Build frontend only
```
These two are also Tauri's own before-commands, named in `tauri.conf.json`, which is why they keep those names instead of joining the list above.

### The Linux Workspace

```bash
cd linux
pnpm build        the four linux packages, and a check of the AUR recipe
pnpm hash         stage them and write the sidecars, building nothing
pnpm upload       send what is staged to the production server
```
**Three commands and no setup step**, because `build` brings its own toolchain images up to date every time — three seconds once they exist, and the reason nobody has to notice when a version inside a `Dockerfile` moves. Docker Desktop has to be installed and running; the first build on a new machine is much longer, since it makes four images first. `build-distro`, `build-flatpak`, `build-aur`, `build-images` and `stage` sit underneath for factoring and are rarely typed.

`linux/README.md` is the guide — the commands, and every filename on the way through. It makes four packages — a `.deb` for arm64, a `.deb` and an `.rpm` for x86_64, and a `.flatpak` for x86_64 — plus the AUR's PKGBUILD, which is a recipe rather than a package and so is validated here rather than published. An image is the toolchain and a container is one build: nothing survives between runs, so every build starts from the same known state, which is worth more than speed at a few releases a year.

**The base image is `debian:12-slim` and that is load-bearing.** A binary is compatible with its build machine's glibc and every later one, never an earlier one, so the base sets a floor on who can run the result. Debian 12's 2.36 reaches Ubuntu 24.04 LTS, Mint 22.x, Fedora 40 and up, and both generations of Raspberry Pi OS — verified by installing the built packages on clean Debian, Ubuntu 24.04 and Fedora images. Debian 13 would have moved that floor to 2.41 and shut out the current Ubuntu LTS.

### The Site Workspace
```bash
cd site
pnpm local        # VitePress dev server
pnpm build        # Static files into docs/.vitepress/dist/
pnpm upload       # Build, then ship dist/ to the server
```
`upload` means the installer in `desktop` and the site in `site`, deliberately — one word, and each workspace ships what it made. The site build never learns a hash, because the download page fetches each sidecar at runtime.

**The dev server answers a sidecar from this machine first and from production second**, which is a small Vite plugin plus a proxy in `docs/.vitepress/config.js`. The plugin reads `desktop/release/` and `linux/release/` at request time and serves whatever is staged there; anything it does not find falls through to the proxy and comes from fujidesktop.app. So `pnpm local` shows the packages you are about to publish alongside the ones already live — the exe, say, which is built on the Windows box and arrives from the server. A response carries `x-fuji-sidecar` when it came from this machine, so the two can be told apart without guessing.

That is the third arrangement, and the two it replaced are worth knowing. **Copying was first**, a `pnpm fixtures` script that put this machine's sidecars in `docs/public/` — which worked, but one left behind is baked into a build and served from the site's own directory, shadowing the real file and pinning the page to a stale hash. Those copies are gitignored, so a machine that ran it may still hold some and no other machine can tell; `upload-site` refuses to ship a `fuji.*.json` it finds in the build and names what to delete, which is the guard that script's `clear` mode used to be. **Proxying alone was second**, honest about what was live and unable to show you your own staged work at all. Neither the plugin nor the proxy can bake anything into a build, because `vitepress build` never runs `configureServer`.

### The Root Script

**One file at the monorepo root, `scripts.js`, holds the whole build pipeline**, reached by a verb: `reveal`, `hash`, `upload-installer`, `upload-site`, `icons-collect`. Nothing runs it directly — the package.json scripts above are the names a person types.

It was four scripts across both workspaces until September 2026; the essay at the top of `scripts.js` says why they became one. The `platforms` table there now holds the bundle folder, the filename suffix, the published name and the file-manager command together, and is the only place any of them is said.

Two rules keep it workable. **It imports node builtins and nothing else**, because the root `package.json` has no dependencies and `node_modules` belongs to the workspaces below it. And **every path is built from the file's own location**, never from the working directory, because both workspaces call it and each calls it from its own folder — which is a thing the old scripts each answered differently and only got away with because pnpm happened to run them from the right place.

### Regenerate the Icons
```bash
pnpm icons        # rebuild every platform's icons from src-tauri/icons/app-icon.svg
```
One run writes the macOS `.icns`, the Windows `.ico`, the Linux PNGs, the Store logos, and the mobile trees together, so no platform needs its own run. It reads three sources, not one — `app-icon.svg` for everything shared, `app-icon-mac.svg` for the dock, and `app-icon-tile.svg` for the Windows Start menu tile — because those three want the disc at different sizes and nothing else differs between them. These are committed artifacts, which means a Tauri CLI upgrade does not refresh them — re-run this after one. `icon.md` says why that matters and what it has already cost.

**Expect exactly two modified files afterwards, both `.icns`.** `tauri icon` writes that container's records in an unstable order, so `icon.icns` and `mac/icon.icns` come back reordered on every run, on every machine, with every image byte-identical — verified record by record. Committing them is lossless. Any *other* file appearing in `git status` after a run is a real change worth looking at.

### Clean Build Environment
There are deliberately no cleanup scripts. The old `wash`/`upgrade-wash` pair were yarn-classic-era crutches — that ecosystem needed frequent clean reinstalls; pnpm's store does not. If a genuine mess ever needs clearing, delete `desktop/dist`, `node_modules`, or `desktop/src-tauri/target` by hand — and never delete the tracked lockfiles: pnpm-lock.yaml and Cargo.lock serve both mac and windows, and removing them to fix a problem is the anti-pattern that motivated the pnpm switch.

## Architecture

**The two layers are not coworkers.** Fuji's Rust and its JavaScript do not confer, do not split a problem between them, and do not solve one together. Every Rust command is dumb, atomic and simple; all the orchestration — the sequencing, the routing, the deciding what to call and when — happens in the page. `SquareFlow.vue` is the worked example: it decides per file whether a thumbnail comes from the operating system or from the page, runs two loops at different widths, and stops them when a card goes away, while the Rust underneath does nothing but answer one question about one file.

**The Rust codebase grows for three reasons and no others: speed, operating-system proximity, and a permission or security necessity.** Never because a task is "mostly Rust" or "involves Rust." When a feature needs both layers, the question is what single dumb thing Rust has to do that JavaScript cannot — write that, and put the rest in the page.

**One codebase behaves differently on each platform in exactly one place, and three rules keep it that way.** *Ask the platform question once, where a thing is made, and never where it is used* — `window_build` is the only way a window comes into being, every caller is in `lib.rs`, and nothing downstream ever asks again. *Write for the plural and let the simpler platform be the degenerate case* — paths waiting for a page are keyed by window label everywhere, and Windows runs that map with one entry forever, needing no branch to do it. *Where a rule can be identical for free, make it identical* — settings and the log flush when the last window closes on every platform, not only at a quit. The alternative, per-process on one platform and per-window on another, is what makes every later feature ask which world it is in.

**The payoff is fewer commands and a smaller conversation across the boundary,** which is what keeps the whole reliable and easy to reason about. `disk.rs` is five atomic calls, `thumbnail.rs` is two, `panel.rs` answers one question, and `log.rs` and `desktop.rs` each hold some text and write it on the way out. None of them knows what it is part of. `lib.rs` already calls its handler list the whole of fuji's attack surface, so keeping that list short is this same discipline seen from the security side.

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

- `open.rs` - A file the operating system handed fuji, because the user double-clicked a picture:
  - `open_files()` - The paths handed over since the page last asked, emptying the list as it answers
  - Filled from `RunEvent::Opened` on macOS and from the command line on Windows and Linux, and held rather than delivered because at launch both arrive before the page exists

- `associate.rs` - What fuji has told the operating system it can open:
  - `associate_register(types)` - Write the Windows registry entries that offer fuji for a list of extensions; a no-op on macOS, on Linux, and in a debug build
  - Offers and never claims: the one value it does not write is the extension's own default, which is what would take a file type. macOS needs nothing here, since its declaration is `CFBundleDocumentTypes` in `src-tauri/Info.plist`, which Tauri merges into the bundle at build time
  - `associations.md` is the whole subject

- `log.rs` - Fuji's log, the half that holds the text and writes it:
  - `log(text)` - One line from any Rust code, appended to the run's log; a no-op unless the page started a log
  - `log_start(path)` - The page names the file, once, only when `log.record` is on
  - `log_append(text)` - The page's lines, a batch at a time
  - `log_write()` - Called from `RunEvent::Exit`, making the folder if needed; `src/log.js` is the page's half and carries the essay on why a file and not a console
  - Written from `RunEvent::Exit`, the one event every way of quitting reaches, making the file's folder first if it is missing

- `thumbnail.rs` - The operating system's thumbnailer, ImageIO on macOS and WIC on Windows, behind two commands:
  - `thumbnail_probe(paths)` - For each path, what its first bytes say it is and what its header says its size is, without decoding; one call per card. Refuses bytes fuji does not know and a header claiming a raster over half the machine's memory
  - `thumbnail_render(path, format, maximum, gamut)` - Decode the file scaled so its longer side is at most `maximum` pixels, oriented and color-converted, returning one buffer: a 12-byte header of width, height and whether the pixels are Display P3, then straight-alpha RGBA. Refuses a file whose bytes are not `format`. Runs on Tauri's thread pool. Rejects on Linux
  - `SquareFlow.vue` is the caller; the thumbnail pipeline document on the site says which files go here and which the page makes for itself

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
- `Card.vue` - A box of up to `card.images` thumbnails, all from one folder, handed to the flow; names the one flow there is, and a second one brings a register back with it
- `SquareFlow.vue` - The one flow, and the whole of how a path becomes a tile: probes a card's files in one call, lays every box out at its final size, then fills canvases from the operating system where the platform's list allows and from the page where it does not, with GIF and SVG as img tiles; waits while the sheet is hidden. `TagFlow.vue` and `CanvasFlow.vue` were the experiment it replaced and are deleted
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

- `open.js` - Exposes the files fuji was opened with:
  - `openFiles()`

- `associate.js` - Composes the type list out of `imageTypes` and hands it down:
  - `associateRegister()`

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
- A thumbnail is a canvas fuji sized, its pixels from the operating system through `thumbnail.rs` where the platform's allow list permits and from the page where not; a GIF or an SVG is an img. `SquareFlow.vue` is the code, and the thumbnail pipeline document on the site is why it is shaped that way
- A canvas is sized to its box in device pixels — whole CSS pixels times `devicePixelRatio` — and never to whatever size the thumbnail came back at. A canvas even one device pixel short of its box is resampled by the compositor on every row, and a fractional CSS size does not fix it. `flowSnap` and `flowEdge` in `SquareFlow.vue` hold this; `fidelity.md` has the measurement
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
./desktop/src-tauri/target/release/bundle/nsis/Fuji_0.1.0_x64-setup.exe
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

**Two different files are named `fuji.exe`, and their sizes tell them apart at a glance.** `src-tauri/target/release/fuji.exe` is the application itself — the binary the NSIS installer wraps, and the one that runs in place without installing. `release/fuji.exe` is the staged **installer**, a copy of `Fuji_0.1.0_x64-setup.exe` under its publishing name, and it is what the website offers for download. Measured on the Windows box 2026-09-11: the binary is 9,512,448 bytes and the installer 2,042,921, because NSIS compresses what it wraps.

`bundle.targets` names what a `tauri build` makes, rather than Tauri's default `"all"` — which also builds an `.msi` beside the NSIS installer and an `.AppImage` beside the Debian package, neither of which anything links to. One list serves every platform: a target that does not apply to the machine doing the build is skipped, and **the skipping is silent**, so a build producing one file is not evidence that anything went wrong.

The linux containers do not use that list. They pass `--bundles deb` or `--bundles deb,rpm` on the command line instead, which overrides it for that run only — so the linux specifics stay in the `linux` workspace and what the mac and windows builds are told to make never changed.

`pnpm hash` copies the bundle out from under its versioned, architecture-specific name into `release/` under a stable publishing name, and writes the sidecar beside it from the bytes that landed. The rename happens here rather than at upload time, which is what lets the site side copy known filenames from a known path with no rules about versions or architectures. The installers stay out of git; the sidecars are committed, so history keeps a dated record of what hash each release had.

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
