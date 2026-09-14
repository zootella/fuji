use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager};

/*
Fuji's menu bar, which exists on macOS and nowhere else. Tauri applies a menu of its own under `#[cfg(target_os = "macos")]` when an application sets none, so fuji has always shown a menu it never wrote, and Windows and Linux have shown no menu at all. Setting one here has to stay gated the same way, because off the Mac a menu goes *inside* the window rather than along the top of the screen.

**Why the whole menu is spelled out rather than edited.** Three items had to change, and Tauri's default can only be reached into by position — find the File submenu, insert at index one. That breaks silently on the day Tauri reorders its default, and the failure looks like an item in the wrong menu rather than an error. So fuji writes out every item it shows, and the price is that a future Tauri improvement to the default arrives here by hand.

**What differs from that default, and nothing else does.** File gains New Window and Open…. View's system Toggle Full Screen is replaced by fuji's own, because fuji has a fullscreen of its own and two must not coexist — `window_no_spaces` in window.rs shuts the other doors into the system one, and menu.md carries the reasoning. The Window submenu is registered with AppKit so macOS keeps it filled with the open windows, which Tauri's default never does. Everything else is copied across exactly, the Edit menu's unreachable Cut and Paste included: those are the platform's own items and removing them is a separate decision nobody has taken.

**Rust does one of the three itself and hands the other two to the page.** Making a window is Rust's, because only Rust can make one. Choosing a file and toggling fullscreen are the page's, because the page already does both — a picker ends in the same call a dropped file takes, and the table's fullscreen is a double-click away. Sending those two down means one implementation of each rather than two, which is the rule in CLAUDE.md about the two layers seen from the menu's side.

**The menu is application-wide and a menu item is not.** macOS shows one menu bar however many windows are open, so an item has to act on the window in front. The event below goes to the focused window alone, and to nowhere at all when fuji is resident with no windows.
*/

//the ids fuji's own items carry, and the contract between this file and Shell.vue, which matches on these same strings
pub const MENU_NEW_WINDOW: &str = "menu-new-window";
pub const MENU_OPEN: &str = "menu-open";
pub const MENU_FULLSCREEN: &str = "menu-fullscreen";

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
		&Submenu::with_items(app, "View", true, &[
			&MenuItem::with_id(app, MENU_FULLSCREEN, "Toggle Full Screen", true, Some("CmdOrCtrl+Ctrl+F"))?,//fuji's own fullscreen, not the system's, and macOS adds a second item of its own to any menu called View — the essay above toggleFullscreen in DiamondTable.vue is the whole subject and worth reading before touching either. ⌃⌘F is the keystroke a mac user already knows, reaching the code a double-click reaches. Spelled this way because muda parses "Cmd" to Modifiers::META and its macos layer only turns Modifiers::SUPER into the command key — so "Ctrl+Cmd+F" silently loses the command and becomes ⌃F. The CmdOrCtrl family is the only spelling that produces command here, which is why the two items above use it
		])?,
		&windows,
		&Submenu::with_items(app, "Help", true, &[
			&PredefinedMenuItem::about(app, None, None)?,
		])?,
	])?;

	app.set_menu(menu)?;
	windows.set_as_windows_menu_for_nsapp()?;//after the menu is the application's, and only there: muda finds this submenu by walking NSApp's own main menu, so called any earlier it resolves nothing and returns quietly, which is exactly how the window list came out empty the first time. macOS then keeps it filled with every open window, named by its title bar — the picture on a table, the folder on the sheet
	Ok(())
}

/// Act on a chosen menu item: make a window here, or tell the frontmost window's page about the ones it owns
pub fn menu_chosen(app: &AppHandle, event: MenuEvent) {
	let id = event.id().as_ref();
	if id == MENU_NEW_WINDOW { crate::window::window_open(app, vec![]); return }//only rust can make a window, so this one never reaches the page
	if id != MENU_OPEN && id != MENU_FULLSCREEN { return }//a predefined item macos handles by itself, and not fuji's business

	let Some(window) = app.webview_windows().into_values().find(|w| w.is_focused().unwrap_or(false)) else { return };//no window in front, which on macOS means fuji is resident with none open; the item has nothing to act on
	let _ = app.emit_to(window.label(), "menu", id);//to that window alone, because the menu bar is shared and the item is not
}
