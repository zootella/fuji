# To the macOS session

A decision was taken on the Windows box on 2026-09-13 that changes what fuji does when a second picture is opened, and macOS is the side that has to change. Windows was already right by doing nothing.

**Read `instances.md` first.** It was a whiteboard and is now a decision, with the reasoning and the conceded costs. This letter says only what that document cannot: what to build, and the one piece of it that is easy to get wrong.

## The decision, in one line

One process per window, on both platforms. A second picture opened while fuji is running gets its own fuji — its own process, window, model, cache and log — sharing nothing with the first and unable to disturb it.

## Why macOS is the one that changes

Windows gets this from Explorer: a new process per double-click, always. Verified here, two live processes, and `tauri-plugin-single-instance` is not a dependency and must not become one.

macOS will not launch a second copy of a bundle for a double-click. LaunchServices sends the running instance an Apple event instead, which Tauri raises as `RunEvent::Opened`. So the running instance is the only thing that ever learns the user wants another picture, and **starting the second instance is therefore its job.** That inversion is the whole of the work: fuji is told about a request it is meant to hand to a copy of itself.

The three reasons for breaking the platform's grain here rather than Windows' are in `instances.md`. The short form: one codebase cannot hold two architectures, the document-centric model fits a folder viewer better than the application-centric one, and breaking macOS's grain costs one function where breaking Windows' would cost an architecture.

## What to build

**One behaviour change, in the `RunEvent::Opened` arm in `lib.rs`.** Today it calls `open::open_urls`, which holds the paths for the page to drain. That is correct at launch and wrong afterwards, and the two cases have to be told apart:

- **At launch**, the event arrives before the page exists and fuji is being started *for* that file. Keep exactly what happens now: hold the paths, let the page drain them when it mounts. `open.rs` already does this and needs no change.
- **While running**, the page has already drained and is showing something. Launch a new fuji for that file instead, and leave this instance untouched. Nothing is held, nothing is emitted, and the page never learns the event happened.

**Telling the two apart** is a flag set once — after the page's first drain, or after the window is revealed. `open.rs` holds the list and is the natural place for it. Whatever it is, it has to be true before a user could plausibly double-click again, and false during the launch that brought fuji up.

**Launching the second instance.** `open -n` is the documented way to force a new instance past LaunchServices, and it takes the file too, so the new process gets the picture the way any launch does:

```
open -n -a <path to Fuji.app> <the file>
```

`std::env::current_exe()` gives `Fuji.app/Contents/MacOS/fuji`, so the bundle is three levels up from it. Executing that inner binary directly also starts a separate process and skips LaunchServices — it is the fallback if `open` proves awkward, but `open -n` is the supported route and sets the new process up as a launched application rather than a child.

**Nothing else changes.** No window management, no shared state, no message passing between instances. The two processes never speak again after the second one starts.

## The two things that are easy to get wrong

**The at-launch case must keep working exactly as it does.** It is verified — a double-click after *Change All* launches fuji onto that picture with its folder behind it, flipping and the `c` key live from the first frame — and it is easy to break while adding the branch beside it. A regression here looks like fuji opening to an empty window when double-clicked cold.

**`LSMultipleInstancesProhibited` must stay unset.** Fuji does not set it today. Setting it would forbid the very thing being built, and it is the kind of key that gets added to an `Info.plist` because it sounds tidy.

## What to expect, and what nobody has seen yet

**The Dock is unobserved.** Whether several instances of one application show one tile or several is not known, and will be apparent the first time this runs. If it is one tile, a user may not have an obvious way to tell the windows apart from the Dock, which is worth noticing but is not a reason to change the design.

**Quit semantics differ from a normal Mac application.** ⌘Q will quit the instance that is frontmost, not every fuji. That follows from the decision rather than being a defect, but it is unusual on macOS and worth seeing before judging.

**Settings will be clobbered**, last writer wins, and that is accepted. Every instance reads `fuji.toml` at startup and writes it at exit, so the remembered size is the casualty — whichever window was resized last, or not at all, wins. `instances.md` records this as a known cost, and explicitly as something that must not be allowed to steer the design. Do not build a guard as part of this work.

## The window change, already made, which you should check on a Mac

Two instances opened in exactly the same place on Windows, pixel for pixel, because fuji restored the rectangle it had recorded — position as well as size. That is right for one window and wrong for several, so it was fixed here on 2026-09-13 and the fix is in the code you will pull:

- `window.remember`, `window.x` and `window.y` are **gone from the settings schema**. There is no way to turn either half on or off. A size is always remembered, a position never is.
- **Rust builds the window now**, in `setup()`, at the size it reads out of `fuji.toml`. `tauri.conf.json` declares no window at all. A new module, `settings.rs`, does that one read and nothing else — it never writes, and the page still owns the schema, the repair and the write-at-exit exactly as before.
- **The recorded size is in css pixels**, where it used to be Tauri's physical ones. Tauri has only two words, logical and physical, and its physical covers the backing bitmap on macOS and the panel's own pixels on Windows — so the number used to mean different things on different machines. `settings.rs` carries the reasoning.
- `revealWindow` no longer sizes anything; it shows the window and that is all. `onSomeMonitor`, the `onMoved` listener, `settingsWindowRect` and the desktop-fraction fallback are all deleted from the page. The fallback moved into Rust as `starting_size`.

**This matters to you beyond instances**, because the css-pixel change fixes a hazard that was mostly a Mac one: a window recorded on a Retina panel used to carry a number twice the size it looked, and restoring it on an attached 1× display gave a window twice as wide as intended. That cannot happen now.

**Two things could behave differently on a Mac, and neither is checked.**

**Does macOS cascade, or centre?** This is the one that matters. No window config names an `x` or `y`, so placement is the platform's. Windows cascades. If macOS instead centres every new window — and two separate processes each creating their first window is exactly the case where it might — then two fujis will stack again and none of this will have helped there. Watch it first, and if they stack, the answer is on the macOS side rather than in the shared code.

**Does macOS place a window that does not fit?** On Windows it does, which was an unwelcome surprise: the cascade walks a fixed staircase and never checks the window fits the work area, so a tall window two or three steps down sits with its bottom under the taskbar. Building at the true size does not help — that was tried on exactly this theory and Windows ignored it.

**That is fixed, in `window.rs`, and the fix is cross-platform already.** After the window is built, fuji asks which monitor it landed on, compares all four edges against that monitor's work area, and if any is outside, rolls a new position uniformly inside it — both axes, because the cascade moves in both at once and keeping a good axis would leave every corrected window in the same column. A window too big to fit is pinned to the work area's near corner and allowed to overhang the far one, never resized. All of it happens while the window is still hidden, so nothing flashes.

Nothing in that is Windows-specific: `work_area` resolves through `SPI_GETWORKAREA` on Windows and the frame that already excludes the menu bar and the Dock on macOS. So if macOS has the same fault it should already be corrected, and if it does not, the check finds the window inside and leaves it alone. Worth watching once to confirm which of those is happening, since neither leaves a visible trace.

## What was conceded to get here

All of it is in `instances.md` and none of it changed the decision: more memory than one process with many windows, no shared decode cache, and the settings clobber above. What is bought is one architecture instead of two, and isolation the kernel enforces rather than a discipline every future feature has to keep.
