# Instances and windows

Whether fuji is one window or several, what happens when the user opens a second picture while the first is still on screen, and where shared state lives if there is more than one window looking at it.

**This is a whiteboard.** Goals, requirements, prior art worth reading, and questions. Nothing here is decided, and the one thing that is settled — what the first associations pass does — is settled only because that pass needed an answer and this document did not exist yet.

## The goal

**Fuji should feel document-centric even though it is one application.** A user who double-clicks a picture starts from the document, not from the program. If they already have fuji open on another picture, that window and the folder behind it are meaningful state — they may be in the middle of something — and the new picture must not blow it away.

**So: the appearance of a second instance, over the reality of one.** Another window, from the same process, sharing settings and state and the cache. The user sees two fujis side by side; the machine has one.

## What the platforms do, and why they differ

**macOS enforces one process per application.** Double-clicking a second picture does not launch a second copy; it sends the running one an Apple event, which Tauri surfaces as `RunEvent::Opened`. Whether that becomes a second window is entirely the application's choice, and a Mac application that does nothing gets the behaviour everyone complains about in VLC: the one window turns to the new file and the old one is gone.

**Windows launches a new process per double-click.** Explorer runs the executable with the path as an argument, every time, whether or not the application is already running. An application that does nothing gets separate windows for free, and pays for it with separate everything else.

**So the two platforms fail in opposite directions,** and the correction is opposite too: macOS needs a window made, Windows needs a process prevented. `tauri-plugin-single-instance` is the tool for the second half, handing a new process's arguments to the running one and exiting.

## What the first associations pass does, and why it is not the answer

`associations.md` had to pick something. It picks the two do-nothing behaviours: on macOS the running window reorients to the new picture, and on Windows a second process starts with its own window. Neither is right, both are shippable, and the reason for taking them is that the correct behaviour is this document's subject rather than that one's.

**The Windows side is closer to correct by accident.** Two windows is what the user wanted; two processes is the wrong way to get it. The macOS side is the VLC behaviour and is the one a user would report as a bug.

## The finding that makes this harder than it looks

**Two Tauri windows do not share a JavaScript heap.** Each window is its own webview running its own copy of the page, so `model.js`, `cache.js` and `settings.js` exist once per window and know nothing of each other. This needs verifying against Tauri's own documentation before it is treated as fact, but if it holds, then "the same state and the same cache" is not something two windows fall into — it is the whole engineering problem.

Three consequences follow if it is true:

**The cache stops working across windows.** Its entire purpose is that a picture is decoded once; two windows on the same folder would decode everything twice and hold two copies. `cache.md` is built on a single store in a single page.

**Settings writes race.** Both windows would read `fuji.toml` at startup and write it at exit, and the last one out wins. Window position is the obvious casualty and there would be others.

**The log is a per-run file.** `log.js` starts one file per launch and names it for the table and window it started with. Two windows in one process would either share a file and interleave, or need a name each.

None of these is a reason not to do it. They are the work.

## Prior art worth reading properly

**Preview** is the closest analogue on macOS and the one to study first: a document-centric application that opens each picture in its own window, and honours the system's *prefer tabs when opening documents* setting, which is a macOS-wide preference that decides window-versus-tab on the application's behalf.

**Finder** opens as many windows as asked and shares per-folder view state between them.

**Chrome and Safari** allow a tab to be torn off into a window and dragged back, over one profile and one set of settings.

**Zed** makes new windows freely and they do not interfere.

**VLC on macOS** is the failure case the user named: one window, and a second film replaces the first. Windows VLC does not behave that way, which is the platform difference above showing through as a user-visible inconsistency in one application.

The pattern across the good ones is the same: one process, many document windows, shared preferences, and state that belongs to a document living with that document rather than in a global.

## Questions

**Does fuji want multiple windows at all, or is one window with good history enough?** Cheapest answer first: a `back` that returns to the previous picture and folder might cover most of what a second window would have been for. It is worth knowing whether the requirement is really two windows or really "do not lose what I was looking at."

**Where does shared state live, given that Rust stays dumb?** `CLAUDE.md`'s rule says the Rust layer does not orchestrate, so a Rust-owned model is against the grain. The alternatives are one window owning the state and the others asking it, or genuinely independent windows that coordinate only where they must.

**What does the cache do?** Per window and duplicated, or one store somewhere both can reach. This is the expensive one and `cache.md` has the constraints it would have to meet.

**How do settings writes get serialised?** Whatever the answer, the user has said clearly that a worry about settings clobbering must not be allowed to steer the larger design.

**What does the window-position memory mean with several windows?** Today fuji records one rectangle. Several windows need either several, or a rule like cascade-from-the-last.

**Does a second window get its own log file, or a shared one with a window column?**

**What is the right trigger for a new window?** Every launch-with-a-file, or only when the running window is showing something else, or a modifier held during the double-click, or a preference.

## Scope

Not now, and not part of the first associations pass. What that pass must not do is make any of this harder, which is the one requirement this document places on it: nothing in the way a launched file reaches the page should assume there is exactly one window forever.
