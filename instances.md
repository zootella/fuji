# Instances and windows

Whether fuji is one process or several, how many windows a process holds, and what happens when the user opens a second picture while the first is still on screen.

**This document carries a decision, taken 2026-09-14.** What is written below is the shape fuji is built to, and most of it is running.

## The decision

**One process per double-click on Windows and Linux, and one process holding as many windows as the user wants on macOS.** Each window is a world of its own either way: its own page, its own model, its own cache, its own pictures. What differs is only whether that window arrived in a process of its own, and the platforms decide that rather than fuji.

On Windows and Linux the shell starts fuji again for every file opened, so there is nothing to write and nothing to prevent. On macOS the shell refuses to start a second copy and tells the running one instead, so the running one makes another window.

**Why not one process per window everywhere, which is simpler.** Because the macOS Dock will not have it. Measured on the Mac mini, macOS 15.7.4, 2026-09-13: two instances of one bundle appear to LaunchServices as two foreground applications — separate serial numbers under a single bundle identifier — and the Dock draws a tile per foreground application rather than per bundle. Three fujis are three identical mint discs, clicking one raises only its own window, and ⌘Tab lists them separately. Nothing merges them; the only key that changes fuji's tile at all is `LSUIElement`, which removes the tile and the menu bar together.

**Windows and Linux hide exactly the same architecture, which is why they keep it.** The Windows taskbar combines buttons for one executable by default, so several fujis sit under one button that expands to each window. GNOME's dash and KDE's task manager match windows to a `.desktop` entry by `WM_CLASS` and group them the same way. macOS is the only shell of the three that refuses, so it is the only platform that needs code.

## What each platform does

**macOS is the one that needs code.** LaunchServices will not launch a second copy of a bundle for a double-click; it sends the running instance an Apple event, which Tauri raises as `RunEvent::Opened`. So the running process is the only thing that ever learns the user wants another picture, and making the window is its job. `LSMultipleInstancesProhibited` stays unset — not because fuji wants second instances, but because nothing should be asserted that has not been thought about.

**Windows already does this and the work is to leave it alone.** Explorer runs the executable with the path as an argument, a new process every time. Verified on the Windows 10 box 2026-09-13: two double-clicks, two processes, two windows, both live. `tauri-plugin-single-instance` is not a dependency and must not become one — it exists to hand a new process's arguments to the running one and exit, which is the one behaviour this design rejects on that platform.

**Linux does what Windows does,** through a `.desktop` file's `Exec` line. Unverified on hardware; it is the same mechanism as Windows and needs the same nothing.

## Why this does not fork the codebase

The fear worth naming, because it is the reason the earlier decision went the other way: one codebase behaving differently per platform becomes a codebase nobody can reason about. Three rules keep it from happening, and they are worth more than the decision itself.

**Ask the platform question once, where a fuji is made, and never where one is used.** `window_build(app, paths)` in `window.rs` is the only way a window comes into being, and every caller of it is in `lib.rs`. Nothing downstream ever asks again: a window is a window, and every piece of code that handles one handles it identically everywhere.

**Write for the plural and let Windows be the degenerate case.** The paths waiting for a page are keyed by window label in `open.rs` on every platform. Windows and Linux run that map with one entry forever and need no branch to do it. The alternative — per-process on one platform, per-window on the other — is what makes every later feature ask which world it is in.

**Where a rule can be identical for free, make it identical.** Settings and the log flush when the last window closes, on every platform, rather than only at a quit.

**What actually forks, counted.** Nothing in the page: `Shell.vue`, `Sheet.vue`, `Card.vue`, `SquareFlow.vue`, `DiamondTable.vue`, `model.js`, `cache.js`, `settings.js` and `log.js` carry no platform knowledge at all. Nothing in the stateless Rust: `disk.rs`, `panel.rs`, `thumbnail.rs` and `associate.rs` never knew how many windows exist. What is left is two `#[cfg]` arms in `lib.rs`'s run-event closure and one function in `window.rs`, and the macOS-only code is *smaller* than the separate-process version it replaced.

## How long fuji outlives its last window

**On Windows and Linux, closing the window closes fuji,** which is what those desktops mean by closing a window.

**On macOS fuji stays,** with its Dock tile and the dot under it. Verified on the Mac mini 2026-09-14: closing every window leaves fuji running, clicking the tile brings a new empty window back through `RunEvent::Reopen`, and Quit from the tile's menu ends it.

That is going with the grain rather than against it. On macOS an application is a place the user is in rather than a window they have open — the persistent tile is how they reopen it, how they learn they can keep it there, and how they decide what to quit when the machine is busy.

**The 1990s reason for this convention has expired and a better one replaced it.** Keeping an application resident was a cache for launch cost, back when launching meant paging a large binary off a slow disk. Fuji's entire frontend is about 150 KB of JavaScript and CSS, smaller than most pictures it opens, and cold launches are quick. What makes residency right now is the convention itself, and what makes it cheap is that a fuji with no windows has destroyed its webviews and is the Rust host alone — and that host is dumb, so it is doing nothing.

**A debug build answers no on macOS as well.** `pnpm local` runs the binary out of `target/debug` rather than a bundle, so there is no Dock tile to click for a window back, and closing the window is how a development run is meant to end. `associate.rs` excuses itself from a debug build for its own reasons and this is the second such case.

## What this concedes

**Settings are last-modified-wins, and the loser's change is gone rather than merged.** Every window reads `fuji.toml` at its own mount, keeps its own complete view, and hands the whole file down whenever something in it moves; Rust holds one text per path and writes it when the last window closes. So the window you most recently changed something in is the one whose entire view survives. This is the same on all three platforms and is accepted while fuji has one user.

One sharp edge here is worth remembering rather than rediscovering: the page must never hand down a *blank* to mean "nothing to write", because blank tells `desktop.rs` to forget that path, and with several windows in one process that lets a window which changed nothing erase a change another window is waiting to write.

**No shared decode cache, and one process does not buy one.** Each Tauri window is its own webview with its own JavaScript heap, so `cache.js` is per window and two windows on the same folder decode everything twice. Observed on the Mac mini 2026-09-14 rather than reasoned: two windows opened on different pictures showed different pictures, which module-level state shared between them could not do. The operating system's page cache makes the second *read* free; it cannot hold a decoded raster.

**Memory is per window either way,** since the webview is what costs. What macOS saves by holding them in one process is the Rust hosts, which were never the expensive part.

## What it buys

**One Dock tile on macOS, and the behaviour a Mac user expects from it.**

**A shared Rust core on macOS:** one `thumbnail.rs` thread pool serving every window, one settings read at startup and one write at the end, one log file for the run. The cross-process settings clobber stops existing there — it remains on Windows and Linux, where the processes genuinely cannot see each other.

**Window placement can finally be solved rather than worked around.** `window.rs` rolls a random position for a window the manager misplaced, because separate processes could not know where their siblings were. One process can. A real cascade is available on macOS whenever it is wanted.

## Where a window opens, and how big

**Fuji remembers a size and never a position.** There is no way to turn either half on or off, because neither is a preference: several windows returning to one remembered rectangle would land on top of each other.

**What places a new window is not the same on the two platforms, and the obvious assumption is wrong.** On Windows the window manager does it, cascading down a staircase of its own. On macOS nothing does: tao centres any window it is given no position for — `if attrs.position.is_none() { ns_window.center() }`, read out of tao 0.35.3 — so two windows of one size on one screen centre to the identical rectangle and stack perfectly. Observed on the Mac mini 2026-09-14. A plain NSWindow sits exactly where its frame says, and cascading on macOS is an AppKit convenience that document-based applications opt into and Tauri does not use. So fuji staggers its own windows, stepping a new one clear of any sibling standing where it landed. That code needs no platform test: it asks whether another window of this process is at this spot, and on Windows and Linux there is never another window of this process.

**The size is read before the window is built, which is why `settings.rs` exists** — Rust takes two numbers out of `fuji.toml` before any page exists, so the window is built once at the size it will keep rather than built wrong and corrected. The size is recorded in css pixels rather than Tauri's physical ones, because Tauri's word covers the backing bitmap on macOS and the panel's own pixels on Windows, and only css pixels mean the same thing on every screen. Every window asks the file again rather than copying its siblings, so a window opened after another was resized comes back at the newer size.

**Windows places a window that does not fit, and fuji moves it.** The cascade walks a fixed staircase and never checks the window against the work area — measured 2026-09-13, three instances 1062 pixels tall on a work area 1160 deep, cascaded to 52, 104 and 138, the last two overhanging by 6 and 40. Building the window at its true size does not help; that was tried first and Windows does not care. So `window.rs` looks at where the window actually landed and, if any edge is outside the work area, rolls a new position uniformly inside it — both axes, because the cascade moves in both at once and keeping a good axis would leave every corrected window in the same column. A window too big to fit is pinned to the work area's near corner and allowed to overhang the far one, never resized. The window is created hidden, so all of it happens before anyone is looking. Verified on Windows 2026-09-13 with five instances.

## How many windows a launch makes

**No window is built during setup. Every window comes from the event loop, under one rule: if fuji has no window when it becomes ready, make one.** `window_first` in `window.rs` is that rule, and `Ready` in `lib.rs` is where it is asked.

Walk the three cases and none of them needs a second rule. A double-click on macOS delivers the picture as an Apple event, which builds a window for it; `Ready` then finds a window and does nothing. A launch with nothing to show reaches `Ready` with no window and gets one there. On Windows and Linux the Apple event does not exist, so `Ready` is always the one that makes the window, out of whatever the command line carried.

**Building the window during setup instead is what made a double-click open two,** one holding the picture and one blank. Recorded because it is the reason the rule is shaped this way, and because it hid for a day: the two windows were built at the same size and centred to the same rectangle, so the blank one sat exactly behind the picture and looked like a single correct window. The cascade pulled them apart and made it visible.

**The event order this rests on, measured on the Mac mini 2026-09-14** with a cold launch through LaunchServices: `Opened` arrives 39 milliseconds before `setup()` runs and 52 before `Ready`. Only the second of those matters — `Ready` must come last — but the first is worth knowing, because it means a double-click builds fuji's window before fuji has finished setting itself up.

`Ready` reaches every platform: it is tao's `StartCause::Init`, emitted in `app_state.rs` at `applicationDidFinishLaunching` on macOS, at `linux/event_loop.rs:236`, and at `windows/event_loop/runner.rs:377` when the runner leaves its uninitialised state.

## Where the work stands

**Built and verified on the Mac mini, 2026-09-14:** two pictures opened while fuji runs give two windows in one process under one Dock tile; the command-line route Explorer uses still lands; residency, `Reopen` and Quit all behave; the two windows are genuinely independent; and a cold double-click now creates one webview where it created two, counted rather than looked at.

**Not yet verified:** that the single window a double-click now makes is the one with the picture in it, which wants a person's eyes. And everything on Windows, including that it compiles for the MSVC target — nothing there changes in kind, since that platform runs the one-entry case of the same code, but nothing there is proven either.

## Still open

**A window menu, and ⌘N.** Out of scope deliberately: the goal was that a shell double-click produces the window the user expects. But a Mac user looking at a resident fuji with no windows has only the Dock tile to click, and ⌘N is the gesture they will try first. The obvious next thing.

**Whether a settings guard is ever worth building**, and what it would be: a lock, a merge rather than a replace, or writing only the keys a window actually changed. Now a Windows and Linux question rather than a general one.
