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

**Settings will be clobbered**, last writer wins, and that is accepted. Every instance reads `fuji.toml` at startup and writes it at exit, so window position is the obvious casualty. `instances.md` records this as a known cost, and explicitly as something that must not be allowed to steer the design. Do not build a guard as part of this work.

## What was conceded to get here

All of it is in `instances.md` and none of it changed the decision: more memory than one process with many windows, no shared decode cache, and the settings clobber above. What is bought is one architecture instead of two, and isolation the kernel enforces rather than a discipline every future feature has to keep.
