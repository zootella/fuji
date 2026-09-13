# Instances and windows

Whether fuji is one process or several, and what happens when the user opens a second picture while the first is still on screen.

**This document had been a whiteboard and now carries a decision, taken 2026-09-13.** What is written below is the shape fuji is being built to, not a set of options.

## The decision

**One process per window, on both platforms.** A second picture opened while fuji is running gets its own fuji: its own process, its own window, its own model and cache and log, sharing nothing with the first and unable to disturb it.

Two double-clicks give two fujis the way two double-clicks give two Notepads. If the user is in the middle of something in the first window, the second cannot reach it.

**The first reason is one codebase, and it is the largest of them.** Going with the grain of each platform sounds like the respectful choice and is not available: it would mean separate single-window processes on Windows and one process holding many windows on macOS — two different architectures, with two different answers to where state lives, what a window is, and who owns the model, produced from the same source. Every feature after that would be written twice and reasoned about twice. So one grain has to be broken, and the only question is which.

**Windows' grain is the one to keep.** Apple's model is application-centric: the application is the thing that runs, and documents are opened inside it. Microsoft's was document-centric — the document is the thing, and opening one gets you an instance of whatever shows it. For a viewer that shows one folder at a time, the document-centric model is simply the better description of what the user is doing, and it is the one that survives contact with the simpler implementation. Breaking macOS's grain costs one function; breaking Windows' would cost an architecture.

**The second reason is that the isolation becomes the operating system's guarantee rather than ours.** An application that keeps several windows inside one process has to be careful, in every feature it ever writes, that window two does not change what window one is showing — and that care is unbounded, because it applies to code not yet written. Separate processes make the question stop existing. There is no shared state to be careful about.

## What each platform needs

**The two platforms fail in opposite directions, and only one of them needs code.**

**Windows already does this, and the work is to leave it alone.** Explorer runs the executable with the path as an argument, a new process every time, whether or not fuji is already running. Verified on the Windows 10 box on 2026-09-13: two double-clicks, two processes, two windows, both live at once. `tauri-plugin-single-instance` is not a dependency, is not registered, and is not in `Cargo.lock` — and that plugin exists to *prevent* second instances, not to permit them. So the correct Windows implementation is the absence of one, and the thing to guard against is somebody adding that plugin later believing it to be an improvement.

**macOS enforces one process and has to be talked out of it.** LaunchServices will not launch a second copy of a bundle for a double-click; it sends the running instance an Apple event, which Tauri raises as `RunEvent::Opened`. Multiple instances are permitted — `open -n` starts one, and `LSMultipleInstancesProhibited` is the key that would forbid them, which fuji does not set — but the *gesture* cannot be made to produce one. So the running instance is the only thing that learns the user wants another picture, and starting the second instance is therefore its job.

That is the one piece of code this decision costs: on receiving an open event while already running, fuji launches another fuji for that file instead of showing it. Everything after that is two ordinary processes that never speak again.

## What this concedes

Stated plainly, because each one is real and none of them changed the decision.

**More memory than one process with several windows.** Each instance carries its own Rust host and its own webview. On Windows one fuji already means seven processes — the host plus six WebView2 processes, measured 2026-09-13 — so a second window doubles that rather than adding a window to an existing tree.

**No shared cache.** Two windows on the same folder decode everything twice. The operating system's page cache makes the second *read* free, since it holds the file's bytes, but it cannot hold a decoded raster, so the decode is paid again. `thumbnail.rs` puts that at 65 ms for a 27-megapixel JPEG on an ordinary Windows box, which is the size of the thing being conceded.

**Settings can be clobbered.** Every instance reads `fuji.toml` at startup and writes it at exit, so the last one out wins and window position is the obvious casualty. This is now a known cost rather than an open worry: last-writer-wins is acceptable while fuji has one user, and a guard is a later question, not a blocker. What must not happen is this concern quietly steering the larger design, which the user has said directly.

**The log is per process, which is the right answer by accident.** `log.js` names one file per run, so two instances write two logs rather than interleaving into one. Nothing needs to change.

## What it buys

**No application logic for managing windows from a single core.** No array of renderers the Rust side tracks, no routing a command to the right window, no deciding which window a newly opened file belongs to, no lifecycle for a window that closes while work is in flight. None of that gets written, and none of it gets maintained.

**The simplicity fuji already has, kept.** `model.js`, `cache.js` and `settings.js` are module-level state in a page, and they stay that way, because there is exactly one page per process. The shape that made them simple is the shape that survives.

**A guarantee rather than a discipline.** Isolation enforced by the kernel does not regress when somebody adds a feature.

## What this is not

**It is not a claim that one process with several windows would be wrong.** It would use less memory and could share a decode cache, and a mature fuji may want it. It is a claim that the complexity is not worth paying now, and quite possibly not ever, for an application that shows one folder at a time.

**It is not a crash-isolation argument, or not mostly.** The page already runs out of process on both platforms — WebView2 spawns a renderer, WKWebView spawns a WebContent process — so the risky work, decoding a stranger's file into hundreds of megabytes of canvas, is already isolated from the host today. What separate instances add is isolation from a panic in fuji's own Rust, and that is five small modules that each answer one question. The isolation that matters here is from *ourselves* — from a future feature reaching across windows — rather than from crashes.

## Prior art, and where fuji sits in it

**Notepad** is the model: one process per document, nothing shared, and a second file never disturbs the first. Simple applications on Windows have worked this way for thirty years.

**Preview and Finder** are the macOS counter-model: one process, many document windows, honouring the system-wide *prefer tabs when opening documents* setting. That is the platform's convention and fuji is deliberately not following it.

**VLC on macOS** is the failure case the user named: one window, and a second film replaces the first. Windows VLC does not behave that way, which is the same platform split showing through as an inconsistency inside one application. Fuji's decision removes that split by making both platforms behave like the Windows one.

**Chrome and Zed** make new windows freely, and Chrome does it from one process tree with tear-off tabs. That is the sophisticated version, and it is sophisticated — it is what fuji is choosing not to build.

## Still open

**Whether two Tauri windows share a JavaScript heap** is no longer load-bearing, since fuji will not be putting two windows in one process. It stays worth knowing if the question is ever revisited.

**What the Dock does with several instances of one macOS application** — one tile or several — is unobserved and will be apparent the first time it runs.

**Whether a settings guard is ever worth building**, and what it would be: a lock, a last-writer-wins that merges rather than replaces, or writing only the keys a window actually changed.
