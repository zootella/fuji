# Menus

What fuji offers in the macOS menu bar and in the Dock icon's right-click menu: what the platform gives for free, what fuji has to build, and what is deliberately left alone.

**This is a macOS-only subject.** Tauri applies its default menu under `#[cfg(target_os = "macos")]` and fuji sets no menu of its own, so Windows and Linux have no menu bar at all and nothing planned here changes them. That is worth knowing before starting: a menu item added here cannot put a menu bar inside the Windows window, and a Windows or Linux user reaches every one of these commands another way or not at all.

**Where this came from.** The user is a Windows user and does not know what a Mac user expects from an application. This document is the list, the reasoning, and the decisions taken 2026-09-14.

## What fuji has today

Fuji writes its whole menu, in `menu.rs`. As of 2026-09-15:

- **Fuji** — About Fuji, Services, Hide, Hide Others, Quit
- **File** — New Window ⌘N, Open… ⌘O, Close Window ⌘W
- **Edit** — Undo, Redo, Cut, Copy, Paste, Select All, all of them stubs
- **View** — Toggle Full Screen ⌃⌘F, and macOS's own Enter Full Screen beside it
- **Window** — Minimize, Zoom, Close Window, and every open window listed by name
- **Help** — About Fuji

And on the Dock icon's own right-click menu, **New Window**, above everything macOS puts there by itself.

**The Dock icon's right-click menu lists the open windows and fuji built none of it** — macOS fills that list itself. The menu bar's Window submenu looks the same but is not free: Tauri's default menu never registers that submenu with AppKit, so the list was empty until `menu.rs` registered it. Both read well only because the window title is the picture's filename, so the title work pays off twice in places nobody aimed at.

## The Dock menu and the menu bar are two different things

Worth stating plainly, because they look like one feature.

The Dock menu is not built from the menu bar and neither knows about the other. macOS fills the Dock menu by itself with the open windows, Options, Show All Windows, Hide and Quit.

**That window list is free, confirmed on the Mac mini 2026-09-14** with two windows open — both named in the Dock menu, with no code of fuji's involved. It stands in contrast to the menu bar's own Window submenu, which needed fuji to register it with AppKit before macOS would fill it the same way.

An application may add items above all that, through `applicationDockMenu:` on its application delegate, which is a wholly separate piece of code from the menu bar. Finder's *New Finder Window* and Zed's *New Window* are apps doing exactly that, and so is fuji's New Window now. Applications usually put the same few commands in both places, which is why the two feel like one thing.

## What the menu does, and why

**All of this is built and working**, checked on the Mac mini on 2026-09-14 and 2026-09-15. It is written here rather than only in the code because each item carries a reason that the code cannot show on its own.


### File → New Window, ⌘N

The item a Mac user reaches for first, and the one with nowhere to go today. It makes an empty window, the same thing clicking the Dock icon with no windows open already does.

**This is the one menu item Rust acts on itself,** because only Rust can build a window: the menu event arrives in `lib.rs`'s run-event closure and calls `window_build(app, vec![])`, which is the function `RunEvent::Reopen` already calls. Nothing new is needed underneath it.

### File → Open…, and it opens the way a dropped file opens

A file picker, and then the picture it chose arrives exactly as a dragged-in file arrives.

**That gives three doors onto the same road, and they must stay one road.** Dragging a file into a table was coded first. Double-clicking a picture in the shell came later. This picker is the third. All three end in the same call — `activeView()?.onDrop?.(path)` — which lists the folder, applies the sort, and stands on that picture. A fourth door later joins the same call or it is wrong.

The dialog plugin is registered in `lib.rs` and `dialog:allow-open` is already granted in `capabilities/default.json`, so the plumbing exists and has simply never had a caller. The page opens the picker, because the page owns orchestration; Rust only says that the user chose the menu item.

**The picker shows every file rather than only the ten fuji knows, decided 2026-09-14.** A folder is easier to recognise by everything in it than by a filtered subset, and a filtered list is harder to read at a glance. Choosing something fuji cannot show costs nothing: the model lists the folder and stands on the first picture in it, which is the same thing a dropped non-image already does.

**One reason for this does not survive contact with the rest of fuji.** A picture saved without an extension is now visible in the picker, but fuji still cannot open it — `listFolder` keeps only the extensions in `imageTypes`, so choosing one stands the user on some other picture instead. That is a real subject and `security.md` owns it, under what fuji thinks a file is; it is not the menu's to solve.

### Two fullscreens, side by side, decided 2026-09-14

**Fuji keeps both kinds and offers both to the user.** They are genuinely different things and each is right for a different moment.

*Fuji's own* is simple fullscreen, in `DiamondTable.vue`: `setSimpleFullscreen`, instant, in place, no Space and no animation, with a black curtain over the transition and a repair that hands the keyboard back to the web view afterwards. It is for checking a detail of a picture and coming straight back, which is most of what a viewer is for. The table keeps its own `fullscreenNow` flag because Tauri cannot report the simple mode.

*macOS's own* moves the window to a Space of its own with the system animation, and is what Split View is built on. It is for settling in. Taking it away would cost an advanced Mac user something real, so fuji does not.

**The words keep them apart in the View menu.** Fuji's item says **Toggle** Full Screen; the system's says **Enter**, and becomes Exit once you are in it. The second item is not fuji's — macOS puts it into the View menu by itself, which came as a surprise and is worth knowing before anyone goes looking for it in `menu.rs`.

**The shortcuts each tell the truth about themselves.** ⌃⌘F is the legacy spelling of the system's fullscreen and macOS no longer advertises it, so fuji takes it for its own item; the system's item keeps Globe+F, which is what macOS shows today. Each label's shortcut does what that label says.

**An earlier decision, on the same day, went the other way and was reversed.** Fuji briefly set `NSWindowCollectionBehaviorFullScreenNone`, which shut every door into the system fullscreen — the green button, the keystroke and the menu item together. It worked, and it cost Split View and any use of a fuji window as a Space of its own, for a confusion that turned out to be fixable instead. Recorded because the flag is the obvious answer to "two fullscreens at once" and someone will reach for it again.

**What makes them coexist is one rule and one repair,** since the whole objection to having both was a user stuck peeling out of two states in turn.

*Toggle means leave, whichever kind you are in.* When fuji's toggle finds the window already in a macOS Space, it leaves the Space rather than laying its own fullscreen on top. Tauri's `isFullscreen()` reports the system fullscreen and deliberately does not report the simple mode, so fuji can always tell the two apart.

*And the other direction is repaired rather than refused,* because fuji cannot intercept the system's own menu item. If macOS takes the window into a Space while fuji's fullscreen is on, the resize handler notices and lets fuji's state go. Whether macOS will even do that to a window whose title bar style mask is cleared is unknown, so this may prove to be a guard against something impossible.

**Rust toggles nothing.** The menu event tells the page which item was chosen and the page calls the same `toggleFullscreen` a double-click calls — one implementation rather than two, and the rule in `CLAUDE.md` about the two layers seen from the menu's side. A menu applies to the frontmost window, so the event goes to that window alone; with several windows on macOS, sending it to all of them made every window answer at once, which is exactly what happened the first time.

### The Dock icon's own menu, built 2026-09-16

**One item: New Window.** `dock.rs` is the record of how — the delegate method AppKit requires, why Tauri cannot do it, and why fuji subclasses the delegate rather than modifying it. What belongs here is the *what*.

**A Dock menu is for starting something when the application does not have your attention** — when it is not frontmost, or not running at all. That is why the convention is so narrow. Finder offers New Finder Window, Safari offers New Window and New Private Window, Terminal offers New Window and New Command. All of them are ways to begin, and nothing else.

**New Window is the only command fuji has of that kind**, so it is the only one here.

**Open… was the near miss and is deliberately left off.** It is also a way to begin, but choosing it from the Dock raises a file picker over whatever the user was doing, with no fuji window on screen to give it any context. New Window and then ⌘O is one more gesture and never surprises.

**Everything else fuji can do acts on the window in front**, and the Dock menu is used precisely when there is no window in front. Toggle Full Screen, switching view, anything about the current picture — none of them have a subject at that moment.

**And the item people actually go there for is one fuji did not build.** macOS lists every open window by name, from the window titles, for nothing.

## Where the work stands

**Tested and working on the Mac mini, 2026-09-14.** A double-click opens one window. The File menu holds New Window and Open…; ⌘N makes a window; ⌘O raises one picker in the focused window alone, listing every file rather than only pictures. The Window submenu lists the open windows and switches between them. ⌘N and ⌘W make and close windows one at a time, leaving the others untouched, and closing the last leaves fuji in the Dock — checked 2026-09-15. Both fullscreens are reachable and neither lands on top of the other: the green traffic light offers the system's, View offers both, ⌃⌘F runs fuji's, and fuji's toggle leaves a Space when it finds itself in one. The Dock icon's own menu offers New Window and it opens one — checked 2026-09-16.

**One bug came out of this work and is fixed.** Entering a Space could leave a view stopping partway down the screen with bare page below it. It was never a menu problem: WebKit keeps a cached viewport for `vh` units and that cache lags, so `h-screen` — which every view used — could be built on a stale number. `index.css` now chains a percentage from the document element instead and forbids the page to scroll, and its essay carries the measurements. Logged 28 disagreements in one session, `vh` wrong every time and alone every time, worst error 156 pixels.

**Gained along the way, unplanned:** the Window submenu is now registered with AppKit, so macOS keeps it filled with the open windows. Tauri's default menu never did this, so that list had always been empty rather than working as this document first assumed.

## Left alone for now

**The Edit menu, reviewed on 2026-09-15 and deliberately kept.** Undo, Redo, Cut, Copy, Paste and Select All are all there and none of them does anything.

**They are not disconnected, though — they have nothing to act on.** These are the standard macOS commands, which travel the responder chain to the WKWebView, and a web view implements every one of them. Fuji's pages simply give them no work: every view carries `select-none` and there is no text field or editable region anywhere, so nothing is selectable to copy and nothing is editable to cut into.

**Kept because a file manager will want them meaning files.** Copy and Paste over *files* is a real destination for fuji, and these are the right names already sitting in the right menu. Removing them now to add them back later is churn.

**And when the dead look bothers somebody, it is a one-line fix rather than a decision.** They appear enabled only because muda turns AppKit's menu validation off. With it on, AppKit asks the responder whether each command applies and greys out the ones that do not, so Copy would dim itself whenever nothing is selected, with no code and no view about what Copy should eventually do. `menu_validate_view` in `menu.rs` already does exactly this for the View submenu and is the worked example.

## Decided against

**Open Recent, decided against on 2026-09-15.** It is a File menu staple on the Mac and fuji is not going to have one. This is a decision rather than a deferral, so nobody needs to cost it again.

**The metaphor is borrowed from documents and does not survive the move.** Recent works because a person has a handful of documents in play over a few days, so the last ten are genuinely the ones they want. Pictures arrive in thousands. The last ten are an arbitrary slice of a folder the user can already open, so the menu is not a shortcut to anything.

**And it would leak.** Picture files are frequently private in a way a spreadsheet is not, and an Open Recent menu does not merely reopen them — it *names* them, on screen, to anyone who glances over, in a screen share, or in a photograph of a desk. A feature that offers nothing and discloses something is an easy decision.

**Nothing is populating such a list today, which is measured rather than assumed.** An application contributes to its own recent documents by calling `noteNewRecentDocumentURL:`, and `tao`, `tauri` and `muda` contain no mention of it, of `NSRecentDocuments`, or of `LSSharedFileList` anywhere. So there is no menu to remove and no list to clear.

**The declarative opt-out is one line and worth taking anyway.** `NSRecentDocumentsLimit` set to `0` in `Info.plist` tells macOS this application keeps no recent documents. It changes no behaviour now; it is insurance against a future code path, or a Tauri change, quietly starting to populate a list fuji has decided not to have.

**One trace does exist, and File → Open… introduced it on 2026-09-15.** Fuji's preferences file, `~/Library/Preferences/com.zootella.fuji.plist`, now holds three keys written by AppKit's open panel: its size, its position, and `NSOSPLastRootDirectory` — 684 bytes of bookmark data recording the last folder browsed. Not a list of files, but the same class of thing: a quiet note of where someone's pictures are. Two ways to answer it, neither taken yet — give the picker an explicit starting folder every time so the remembered one is never consulted, which leaves the key written but inert; or clear those keys on the way out, which `desktop.rs` is already shaped for and which means fighting AppKit over its own preferences on every launch.

**One cost worth naming.** A Dock menu is exactly where Open Recent would have shone — it is the item a person reaches for when the application is not in front of them, and it is why other apps put recent documents there. Deciding against Open Recent is therefore also deciding that fuji's Dock menu stays a single item. That is the right trade for the reasons above, and it is the only place the decision costs anything.

**What fuji cannot reach.** Whether double-clicking a picture lands it in the system's own Apple menu → Recent Items list. That list belongs to Finder and LaunchServices, is populated when *they* open a document, and fuji has no say in it. Worth confirming by looking once, and then documenting as a limit rather than trying to code around.
