# Menus

What fuji offers in the macOS menu bar and in the Dock icon's right-click menu: what the platform gives for free, what fuji has to build, and what is deliberately left alone.

**This is a macOS-only subject.** Tauri applies its default menu under `#[cfg(target_os = "macos")]` and fuji sets no menu of its own, so Windows and Linux have no menu bar at all and nothing planned here changes them. That is worth knowing before starting: a menu item added here cannot put a menu bar inside the Windows window, and a Windows or Linux user reaches every one of these commands another way or not at all.

**Where this came from.** The user is a Windows user and does not know what a Mac user expects from an application. This document is the list, the reasoning, and the decisions taken 2026-09-14.

## What fuji has today

Read out of `tauri-2.11.5/src/menu/menu.rs` on 2026-09-14 rather than observed, so the order on screen may differ slightly:

- **Fuji** — About Fuji, Services, Hide, Hide Others, Quit
- **File** — Close Window
- **Edit** — Undo, Redo, Cut, Copy, Paste, Select All
- **View** — Toggle Full Screen
- **Window** — Minimize, Zoom, Close Window
- **Help** — About Fuji

All of it is Tauri's default. Fuji has never written a menu.

**Two things already work and were never built.** The Window menu lists every open window by name, because muda registers that submenu with AppKit as the application's windows menu and macOS fills the list itself. The Dock icon's right-click menu lists the open windows the same way. Both read well only because the window title is the picture's filename, so the title work pays off twice in places nobody aimed at.

## The Dock menu and the menu bar are two different things

Worth stating plainly, because they look like one feature.

The Dock menu is not built from the menu bar and neither knows about the other. macOS fills the Dock menu by itself with the open windows, Options, Show All Windows, Hide and Quit.

**That window list works today, confirmed on the Mac mini 2026-09-14** with two windows open — both named in the Dock menu, with no code of fuji's involved. Worth weighing before building anything here, because the list is the main reason a person opens that menu and it already does the job. It also stands in contrast to the menu bar's own Window submenu, which needed fuji to register it with AppKit before macOS would fill it in the same way. An application may add its own items to the top of it, through `applicationDockMenu:` on its application delegate, and that is a wholly separate piece of code from the menu bar. Finder's *New Finder Window* and Zed's *New Window* are apps doing exactly that. Applications usually put the same few commands in both places, which is why the two feel like one thing.

## Decided

### File → New Window, ⌘N

The item a Mac user reaches for first, and the one with nowhere to go today. It makes an empty window, the same thing clicking the Dock icon with no windows open already does.

**This is the one menu item Rust acts on itself,** because only Rust can build a window: the menu event arrives in `lib.rs`'s run-event closure and calls `window_build(app, vec![])`, which is the function `RunEvent::Reopen` already calls. Nothing new is needed underneath it.

### File → Open…, and it opens the way a dropped file opens

A file picker, and then the picture it chose arrives exactly as a dragged-in file arrives.

**That gives three doors onto the same road, and they must stay one road.** Dragging a file into a table was coded first. Double-clicking a picture in the shell came later. This picker is the third. All three end in the same call — `activeView()?.onDrop?.(path)` — which lists the folder, applies the sort, and stands on that picture. A fourth door later joins the same call or it is wrong.

The dialog plugin is registered in `lib.rs` and `dialog:allow-open` is already granted in `capabilities/default.json`, so the plumbing exists and has simply never had a caller. The page opens the picker, because the page owns orchestration; Rust only says that the user chose the menu item.

**The picker shows every file rather than only the ten fuji knows, decided 2026-09-14.** A folder is easier to recognise by everything in it than by a filtered subset, and a filtered list is harder to read at a glance. Choosing something fuji cannot show costs nothing: the model lists the folder and stands on the first picture in it, which is the same thing a dropped non-image already does.

**One reason for this does not survive contact with the rest of fuji, and is worth knowing rather than rediscovering.** A picture saved without an extension is now visible in the picker, but fuji still cannot open it: `listFolder` keeps only the extensions in `imageTypes`, so such a file is not in the folder listing and choosing it stands the user on some other picture instead. Fuji identifies files by extension everywhere except `thumbnail_probe`, which reads the first bytes. Closing that gap is a real subject — it would mean the folder listing asking Rust what each unknown file actually is — and it belongs to whoever takes on the model rather than to the menu.

### Two fullscreens, side by side, decided 2026-09-14

**Fuji keeps both kinds and offers both to the user.** They are genuinely different things and each is right for a different moment.

*Fuji's own* is simple fullscreen, in `DiamondTable.vue`: `setSimpleFullscreen`, instant, in place, no Space and no animation, with a black curtain over the transition and a repair that hands the keyboard back to the web view afterwards. It is for checking a detail of a picture and coming straight back, which is most of what a viewer is for. The table keeps its own `fullscreenNow` flag because Tauri cannot report the simple mode.

*macOS's own* moves the window to a Space of its own with the system animation, and is what Split View is built on. It is for settling in. Taking it away would cost an advanced Mac user something real, so fuji does not.

**The words keep them apart in the View menu.** Fuji's item says **Toggle** Full Screen; the system's says **Enter**, and becomes Exit once you are in it. The second item is not fuji's — macOS inserts it automatically into any menu titled "View", which came as a surprise and is worth knowing before anyone goes looking for it in `menu.rs`.

**The shortcuts each tell the truth about themselves.** ⌃⌘F is the legacy spelling of the system's fullscreen and macOS no longer advertises it, so fuji takes it for its own item; the system's item keeps Globe+F, which is what macOS shows today. Each label's shortcut does what that label says.

**An earlier decision, on the same day, went the other way and was reversed.** Fuji briefly set `NSWindowCollectionBehaviorFullScreenNone`, which shut every door into the system fullscreen — the green button, the keystroke and the menu item together. It worked, and it cost Split View and any use of a fuji window as a Space of its own, for a confusion that turned out to be fixable instead. Recorded because the flag is the obvious answer to "two fullscreens at once" and someone will reach for it again.

**What makes them coexist is one rule and one repair,** since the whole objection to having both was a user stuck peeling out of two states in turn.

*Toggle means leave, whichever kind you are in.* When fuji's toggle finds the window already in a macOS Space, it leaves the Space rather than laying its own fullscreen on top. Tauri's `isFullscreen()` reports the system fullscreen and deliberately does not report the simple mode, so fuji can always tell the two apart.

*And the other direction is repaired rather than refused,* because fuji cannot intercept the system's own menu item. If macOS takes the window into a Space while fuji's fullscreen is on, the resize handler notices and lets fuji's state go. Whether macOS will even do that to a window whose title bar style mask is cleared is unknown, so this may prove to be a guard against something impossible.

**Rust toggles nothing.** The menu event tells the page which item was chosen and the page calls the same `toggleFullscreen` a double-click calls — one implementation rather than two, and the rule in `CLAUDE.md` about the two layers seen from the menu's side. A menu applies to the frontmost window, so the event goes to that window alone; with several windows on macOS, sending it to all of them made every window answer at once, which is exactly what happened the first time.

## Where the work stands

**Tested and working on the Mac mini, 2026-09-14.** A double-click opens one window. The File menu holds New Window and Open…; ⌘N makes a window; ⌘O raises one picker in the focused window alone, listing every file rather than only pictures. The Window submenu lists the open windows and switches between them. Both fullscreens are reachable and neither lands on top of the other: the green traffic light offers the system's, View offers both, ⌃⌘F runs fuji's, and fuji's toggle leaves a Space when it finds itself in one.

**One bug came out of this work and is fixed.** Entering a Space could leave a view stopping partway down the screen with bare page below it. It was never a menu problem: WebKit keeps a cached viewport for `vh` units and that cache lags, so `h-screen` — which every view used — could be built on a stale number. `index.css` now chains a percentage from the document element instead and forbids the page to scroll, and its essay carries the measurements. Logged 28 disagreements in one session, `vh` wrong every time and alone every time, worst error 156 pixels.

**Gained along the way, unplanned:** the Window submenu is now registered with AppKit, so macOS keeps it filled with the open windows. Tauri's default menu never did this, so that list had always been empty rather than working as this document first assumed.

## Left alone for now

**The Edit menu.** It offers Undo, Redo, Cut, Copy, Paste and Select All, and none of them do anything in fuji. Dead items read as a promise, so trimming the menu was proposed and the user declined for now. It stays exactly as Tauri built it until there is a reason to touch it — most likely a Copy that copies the picture.

## Open

### The Dock menu's own items

Adding *New Window* to the Dock icon's right-click menu, the way Finder and Zed do.

**Tauri cannot do this.** There is no Dock menu API anywhere in the stack: `tauri` 2.11.5, `muda` and `tao` 0.35.3 contain no mention of `applicationDockMenu`, `dock_menu` or `setDockMenu` at all. So this is fuji writing AppKit calls itself.

**It would be Rust, not Objective-C or Swift.** No `.m` or `.swift` file, and no new toolchain: the `objc2` crates are the Rust bindings to the Objective-C runtime, and fuji already has them beneath it — `objc2` 0.6.4 and `objc2-app-kit` 0.3.2 are in `Cargo.lock`, pulled in by tao. The Xcode command line tools are installed on the Mac mini (clang 17), which any Rust build on macOS needs anyway. So the tools are all present and no dependency has to be added.

**What makes it hairy is the delegate, not the language.** `applicationDockMenu:` is a method on the application's delegate object, and tao owns that delegate and installs it before fuji's code runs. Fuji would have to reach into a class tao defines and add a method to it at runtime, then return an `NSMenu` fuji builds. That is a documented Objective-C runtime technique and it is also exactly the kind of code that breaks quietly on a tao upgrade, because it depends on tao's internals rather than on tao's API.

**Estimate, honestly rough:** the AppKit side is small, perhaps forty to sixty lines in a module shaped like `panel.rs`. The risk is not the size. It is that fuji would own a piece of tao's delegate, with no compiler error to warn it when tao changes.

**Not started, and not obviously worth it yet.** The Dock menu already lists the open windows, which is most of what a user goes there for. This would add one item.

### ⌘W after the window lifecycle changed

Close Window is already in two menus and has never been tried since `instances.md`'s decision landed on 2026-09-14. On macOS, closing the last window now leaves fuji resident; on Windows it ends the process. Both should be what ⌘W does, and neither has been checked by hand.

## Not now

**Open Recent.** A File menu staple on the Mac, and it needs fuji to remember a list of folders across launches. That is a settings change and a schema addition before it is a menu, so it is a feature rather than a tidy-up. Named here so it is not mistaken for an oversight.
