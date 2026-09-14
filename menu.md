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

The Dock menu is not built from the menu bar and neither knows about the other. macOS fills the Dock menu by itself with the open windows, Options, Show All Windows, Hide and Quit. An application may add its own items to the top of it, through `applicationDockMenu:` on its application delegate, and that is a wholly separate piece of code from the menu bar. Finder's *New Finder Window* and Zed's *New Window* are apps doing exactly that. Applications usually put the same few commands in both places, which is why the two feel like one thing.

## Decided

### File → New Window, ⌘N

The item a Mac user reaches for first, and the one with nowhere to go today. It makes an empty window, the same thing clicking the Dock icon with no windows open already does.

**This is the one menu item Rust acts on itself,** because only Rust can build a window: the menu event arrives in `lib.rs`'s run-event closure and calls `window_build(app, vec![])`, which is the function `RunEvent::Reopen` already calls. Nothing new is needed underneath it.

### File → Open…, and it opens the way a dropped file opens

A file picker, and then the picture it chose arrives exactly as a dragged-in file arrives.

**That gives three doors onto the same road, and they must stay one road.** Dragging a file into a table was coded first. Double-clicking a picture in the shell came later. This picker is the third. All three end in the same call — `activeView()?.onDrop?.(path)` — which lists the folder, applies the sort, and stands on that picture. A fourth door later joins the same call or it is wrong.

The dialog plugin is registered in `lib.rs` and `dialog:allow-open` is already granted in `capabilities/default.json`, so the plumbing exists and has simply never had a caller. The page opens the picker, because the page owns orchestration; Rust only says that the user chose the menu item.

### View → Toggle Full Screen must become fuji's own fullscreen

**Fuji has two unrelated fullscreen modes today and they can both be on at once.** The menu item is `PredefinedMenuItem::fullscreen`, which is macOS's real fullscreen — the window moves to a Space of its own, with the system animation. Fuji's own is simple fullscreen, in `DiamondTable.vue`: `setSimpleFullscreen`, instant, no Space, with a black curtain over the transition and a repair that hands the keyboard back to the web view afterwards. The table keeps its own `fullscreenNow` flag because Tauri cannot report the simple mode.

The two states know nothing about each other. Entering both leaves the user peeling out of each in turn, which is what happened when this was found. **Decided: there is one fullscreen in fuji, and it is fuji's own.** The predefined item is replaced with an item of fuji's that does exactly what a double-click on the table does.

**Rust does not toggle anything.** The menu event tells the page that the user chose the item, and the page calls the same `toggleFullscreen` a double-click calls. That keeps one implementation rather than two, and it is the rule in `CLAUDE.md` about the two layers: the page decides, Rust carries the news.

One detail to get right: a menu applies to the frontmost window, so the event has to reach *that* window rather than all of them. Now that fuji can have several windows on macOS, sending it to the wrong one would toggle a window the user is not looking at.

**Changing the menu item does not by itself leave fuji with one fullscreen, and this is unresolved.** The green button in the title bar also enters macOS fullscreen, so a user can still reach it and still end up in two unrelated states at once. Two ways out, and neither is chosen yet.

*One fullscreen:* change the menu item and also turn off the window's native fullscreen, so the green button merely zooms. Whether Tauri exposes that switch — it is `NSWindowCollectionBehaviorFullScreenNone` underneath — is unchecked and has to be settled before committing to this. **The price is Split View**, macOS's arrangement of two fullscreen applications sharing one Space side by side, which simple fullscreen cannot do at all and which is the one real thing an advanced Mac user would lose. The animation nobody would miss; Split View they might.

*Both, not fighting:* the green button and ⌃⌘F stay macOS fullscreen for whoever wants Split View, while the double-click and the menu item are fuji's own. The work is making each refuse to engage while the other is on, so nobody has to peel out of two states in turn. More code, more native, and it keeps the advanced user.

The user's stated preference is one fullscreen, and fuji's own is the right default either way, because fuji's fullscreen is for checking a detail and coming straight back rather than for settling in.

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
