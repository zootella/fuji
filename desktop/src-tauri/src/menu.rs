use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager};

/*
Fuji's menu bar, which exists on macOS and nowhere else. Tauri applies a menu of its own under `#[cfg(target_os = "macos")]` when an application sets none, so fuji has always shown a menu it never wrote, and Windows and Linux have shown no menu at all. Setting one here has to stay gated the same way, because off the Mac a menu goes *inside* the window rather than along the top of the screen.

**Why the whole menu is spelled out rather than edited.** Three items had to change, and Tauri's default can only be reached into by position — find the File submenu, insert at index one. That breaks silently on the day Tauri reorders its default, and the failure looks like an item in the wrong menu rather than an error. So fuji writes out every item it shows, and the price is that a future Tauri improvement to the default arrives here by hand.

**What differs from that default, and nothing else does.** File gains New Window and Open…. View's system Toggle Full Screen is replaced by an item of fuji's own, which runs fuji's fullscreen rather than macOS's. The Window submenu is registered with AppKit so macOS keeps it filled with the open windows, which Tauri's default never does. Everything else is copied across exactly, the Edit menu's unreachable Cut and Paste included: those are the platform's own items and removing them is a separate decision nobody has taken.

**Fuji has two fullscreens and the View menu shows both, which is deliberate and is not this file's subject.** macOS inserts its own *Enter Full Screen* into any menu titled "View", so one item is written here and two appear — worth knowing before somebody hunts for the second one in this file. The essay above `toggleFullscreen` in DiamondTable.vue is where that whole subject lives: why there are two, which is for what, and how they are kept from landing on top of each other.

**Rust does one of the three itself and hands the other two to the page.** Making a window is Rust's, because only Rust can make one. Choosing a file and toggling fullscreen are the page's, because the page already does both — a picker ends in the same call a dropped file takes, and the table's fullscreen is a double-click away. Sending those two down means one implementation of each rather than two, which is the rule in CLAUDE.md about the two layers seen from the menu's side.

**The menu is application-wide and a menu item is not.** macOS shows one menu bar however many windows are open, so an item has to act on the window in front. The event below goes to the focused window alone, and to nowhere at all when fuji is resident with no windows.
*/

//the ids fuji's own items carry, and the contract between this file and Shell.vue, which matches on these same strings
pub const MENU_NEW_WINDOW: &str = "menu-new-window";
pub const MENU_OPEN: &str = "menu-open";
pub const MENU_FULLSCREEN: &str = "menu-fullscreen";

const MENU_VIEW: &str = "View";//the submenu's title, named once because two things below have to agree on it: the menu fuji builds, and the lookup that finds it again afterwards

/// Build the menu bar and make it the application's; lib.rs calls this during setup, on macOS only
pub fn menu_set(app: &AppHandle) -> tauri::Result<()> {
	let windows = Submenu::with_items(app, "Window", true, &[
		&PredefinedMenuItem::minimize(app, None)?,
		&PredefinedMenuItem::maximize(app, None)?,
		&PredefinedMenuItem::separator(app)?,
		&PredefinedMenuItem::close_window(app, None)?,
	])?;

	let menu = Menu::with_items(app, &[
		&Submenu::with_items(app, "Fuji", true, &[
			&PredefinedMenuItem::about(app, None, None)?,//the panel macos assembles out of Info.plist, which already carries fuji's name and version
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::services(app, None)?,
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::hide(app, None)?,
			&PredefinedMenuItem::hide_others(app, None)?,
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::quit(app, None)?,
		])?,
		&Submenu::with_items(app, "File", true, &[
			&MenuItem::with_id(app, MENU_NEW_WINDOW, "New Window", true, Some("CmdOrCtrl+N"))?,
			&MenuItem::with_id(app, MENU_OPEN, "Open…", true, Some("CmdOrCtrl+O"))?,
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::close_window(app, None)?,
		])?,
		&Submenu::with_items(app, "Edit", true, &[//tauri's default, copied unchanged; menu.md says why it is left alone for now
			&PredefinedMenuItem::undo(app, None)?,
			&PredefinedMenuItem::redo(app, None)?,
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::cut(app, None)?,
			&PredefinedMenuItem::copy(app, None)?,
			&PredefinedMenuItem::paste(app, None)?,
			&PredefinedMenuItem::select_all(app, None)?,
		])?,
		&Submenu::with_items(app, MENU_VIEW, true, &[
			&MenuItem::with_id(app, MENU_FULLSCREEN, "Toggle Full Screen", true, Some("CmdOrCtrl+Ctrl+F"))?,//fuji's own fullscreen, not the system's, and macOS adds a second item of its own to any menu called View — the essay above toggleFullscreen in DiamondTable.vue is the whole subject and worth reading before touching either. ⌃⌘F is the keystroke a mac user already knows, reaching the code a double-click reaches. Spelled this way because muda parses "Cmd" to Modifiers::META and its macos layer only turns Modifiers::SUPER into the command key — so "Ctrl+Cmd+F" silently loses the command and becomes ⌃F. The CmdOrCtrl family is the only spelling that produces command here, which is why the two items above use it
		])?,
		&windows,
		&Submenu::with_items(app, "Help", true, &[
			&PredefinedMenuItem::about(app, None, None)?,
		])?,
	])?;

	app.set_menu(menu)?;
	menu_validate_view();//after the menu is the application's, because it asks AppKit for the menu bar and would otherwise find nothing; the essay below is the subject
	windows.set_as_windows_menu_for_nsapp()?;//after the menu is the application's, and only there: muda finds this submenu by walking NSApp's own main menu, so called any earlier it resolves nothing and returns quietly, which is exactly how the window list came out empty the first time. macOS then keeps it filled with every open window, named by its title bar — the picture on a table, the folder on the sheet
	Ok(())
}

/*
Let AppKit keep its own menu item's label honest, which muda otherwise prevents.

macOS puts a second item into the View menu by itself — *Enter Full Screen*, on the Globe+F shortcut — so fuji writes one item there and two appear. That one is meant to retitle itself to *Exit Full Screen* while the window is in a Space, and in fuji it never did.

The retitling is not a mechanism of its own: it is part of menu validation. Before a menu opens, AppKit walks its items and asks the responder chain to validate each, and `NSWindow`'s answer for `toggleFullScreen:` both enables that item and rewrites its title. The walk only happens while the menu's `autoenablesItems` is on, and **muda turns it off on every menu and submenu it builds** — reasonably, since muda tracks each item's enabled state itself and automatic enabling would fight it. The collision exists only because macOS adds an item muda knows nothing about, and that is the one item needing validation. So this turns the flag back on, for the View submenu alone, by setting a property on an AppKit object muda handed over — which keeps it in the same safe category as `ns_window()` rather than the category a custom dock menu would need.

**Fuji's own item is not put at risk**, which was worth checking before writing this. AppKit leaves an item enabled when it has an explicit target that responds to its action and that target does not implement `validateMenuItem:`; muda sets each item's target to the item itself with an action it implements, and implements `validateMenuItem:` nowhere. So fuji's Toggle Full Screen stays enabled, and only the system's item — which has no target and so reaches `NSWindow` — is validated and retitled.

Every way this can fail leaves the label as wrong as it already was and nothing worse: a muda that stops disabling validation makes this a no-op, one that disables it after us puts the stale label back, and a View submenu that cannot be found means doing nothing.
*/
fn menu_validate_view() {
	let Some(marker) = objc2::MainThreadMarker::new() else { return };//setup runs on the main thread, so this is a formality the type system asks for rather than a real question
	let application = objc2_app_kit::NSApplication::sharedApplication(marker);
	let Some(bar) = application.mainMenu() else { return };//no menu bar yet, which means this ran too early
	let title = objc2_foundation::NSString::from_str(MENU_VIEW);
	let Some(view) = bar.itemWithTitle(&title) else { return };
	let Some(submenu) = view.submenu() else { return };
	submenu.setAutoenablesItems(true);
}

/// Act on a chosen menu item: make a window here, or tell the frontmost window's page about the ones it owns
pub fn menu_chosen(app: &AppHandle, event: MenuEvent) {
	let id = event.id().as_ref();
	if id == MENU_NEW_WINDOW { crate::window::window_open(app, vec![]); return }//only rust can make a window, so this one never reaches the page
	if id != MENU_OPEN && id != MENU_FULLSCREEN { return }//a predefined item macos handles by itself, and not fuji's business

	let Some(window) = app.webview_windows().into_values().find(|w| w.is_focused().unwrap_or(false)) else { return };//no window in front, which on macOS means fuji is resident with none open; the item has nothing to act on
	let _ = app.emit_to(window.label(), "menu", id);//to that window alone, because the menu bar is shared and the item is not
}
