use std::sync::Mutex;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

/*
Fuji's menu bar, which exists on macOS and nowhere else. Tauri applies a menu of its own under `#[cfg(target_os = "macos")]` when an application sets none, so fuji has always shown a menu it never wrote, and Windows and Linux have shown no menu at all. Setting one here has to stay gated the same way, because off the Mac a menu goes *inside* the window rather than along the top of the screen.

**Why the whole menu is spelled out rather than edited.** Fuji changes something in nearly every submenu, and Tauri's default can only be reached into by position — find the File submenu, insert at index one. That breaks silently on the day Tauri reorders its default, and the failure looks like an item in the wrong menu rather than an error. So fuji writes out every item it shows, and the price is that a future Tauri improvement to the default arrives here by hand.

**What differs from that default is what makes it a Mac menu, and nothing else does.** The application menu has About and Settings… where every Mac application keeps them, Settings… on ⌘, and both fuji's own items: About opens the settings panel with its About section in view instead of the panel macOS assembles from Info.plist, so every platform reads the same words in the same place. Its hiding group ends with Show All, as every Mac application menu's does and the default's does not. File gains New Window and Open…, and its close item says Close, the Mac's word, where muda's default says Close Window. View's system Toggle Full Screen is replaced by an item of fuji's own, which switches between the contact sheet in a window and the table in fuji's fullscreen, as a double-click does, rather than running macOS's, and the page retitles it to name where it goes next, through menu_text below. The Window submenu ends with Bring All to Front and has no close item, as a Mac Window menu does. Help holds Fuji Help, which opens fuji's help address in the system's browser, and not About, which is a Windows habit. Window and Help carry the ids Tauri gives its own Window and Help menus, so setting the menu registers both with AppKit, as it does the default's: macOS keeps the Window menu filled with the open windows, and puts its search field at the top of the Help menu. The Edit menu is copied across exactly: those are the platform's own items, muda has no Delete to add beside them, and macOS adds Dictation and Emoji on its own. Its items start gray, and the page lights each one while something in the window can answer it, through menu_enable below, because muda turns off AppKit's own asking and offers no switch for the system's items.

**Fuji has two fullscreens and the View menu shows both, which is deliberate and is not this file's subject.** macOS inserts its own *Enter Full Screen* into any menu titled "View", so one item is written here and two appear — worth knowing before somebody hunts for the second one in this file. The essay above `fullscreenSet` in Shell.vue is where that whole subject lives: why there are two, which is for what, and how they are kept from landing on top of each other.

**Rust does one of the six itself and hands the other five to the page.** Making a window is Rust's, because only Rust can make one. Choosing a file or a folder, switching views, showing the settings and About, and opening the site are the page's, because the page already does all of them — the open box answers with a path the page then shows, the fullscreen table is what a double-click already switches to, the settings panel is what s already shows and About is its last section, and the site is an address the page already asks Rust to open. Sending those five down means one implementation of each rather than two, which is the rule in CLAUDE.md about the two layers seen from the menu's side.

**The menu is application-wide and a menu item is not.** macOS shows one menu bar however many windows are open, so an item has to act on a window, and AppKit's own answer is that the window a command acts on need not be the one in front. Each window's page may say which of fuji's items it answers, through menu_answer below, and a window whose page never says answers them all. An item goes to the window in front when that window answers it; otherwise to the window most recently in front that does, brought forward first, which is how Settings… chosen over a video window turns the last contact sheet's window to the settings; and only when no window answers it does it go where it goes with no window open. Rust keeps the windows in the order they were last in front, from lib.rs's window events, and never learns what kind of window any of them is. When fuji is resident with no windows there is no page to send an item to. The View item then goes nowhere. Open… puts the open box up on its own, as a Mac application with no document open does, and makes a window only for what the user chooses, so a cancel leaves nothing behind; the window is made to answer Open… as the items below are, so its page opens the choice the way Open… opens one in a window already open, a picture on the light table rather than in the preview a double-click gets. Settings…, About and Fuji Help bring a window up, the way a click on the dock icon does: one that first appears showing the settings, or with the site opening in front of it. An event cannot reach that window, because its page is not listening yet, so the item waits here under the new window's label until the page asks for it as it mounts: open.rs's rule for a double-clicked picture, delivered to the window made for it, applied to a menu item.
*/

//the ids fuji's own items carry, and the contract between this file and Shell.vue, which matches on these same strings
pub const MENU_NEW_WINDOW: &str = "menu-new-window";
pub const MENU_OPEN: &str = "menu-open";
pub const MENU_FULLSCREEN: &str = "menu-fullscreen";
pub const MENU_ABOUT: &str = "menu-about";
pub const MENU_SETTINGS: &str = "menu-settings";
pub const MENU_HELP: &str = "menu-help";

const MENU_VIEW: &str = "View";//the submenu's title, named once because two things below have to agree on it: the menu fuji builds, and the lookup that finds it again afterwards
const MENU_EDIT: &str = "Edit";//the same, and the page names it too when it lights an item

static MENU_WAITING: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());//items chosen while no window was open, each beside the label of the window made to answer it; a handful at most, so a list rather than a map
static MENU_ANSWERS: Mutex<Vec<(String, Vec<String>)>> = Mutex::new(Vec::new());//the items each window's page said it answers, by label; a window with no entry answers every item
static MENU_FRONT: Mutex<Vec<String>> = Mutex::new(Vec::new());//every window's label, the one most recently in front first

fn menu_answers() -> std::sync::MutexGuard<'static, Vec<(String, Vec<String>)>> { MENU_ANSWERS.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) }//take the list even if a previous holder panicked
fn menu_front() -> std::sync::MutexGuard<'static, Vec<String>> { MENU_FRONT.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) }
fn menu_answered(label: &str, id: &str) -> bool {//whether this window answers this item: every item, unless its page has said which it answers
	match menu_answers().iter().find(|(l, _)| l == label) { Some((_, items)) => items.iter().any(|item| item == id), None => true }
}

/// The items of fuji's own this window answers, by their ids, in place of whatever it said before; a window that never says answers them all
#[tauri::command]
pub fn menu_answer(window: WebviewWindow, items: Vec<String>) {
	let mut answers = menu_answers();
	answers.retain(|(label, _)| label != window.label());
	answers.push((window.label().to_string(), items));
}

/// This window has come to the front, so it goes first in the order the menu bar looks through; lib.rs calls this from the window's focus event
pub fn menu_focused(label: &str) {
	let mut front = menu_front();
	front.retain(|l| l != label);
	front.insert(0, label.to_string());
}

/// This window is going, and answers nothing from here; lib.rs calls this as it closes
pub fn menu_closed(label: &str) {
	menu_front().retain(|l| l != label);
	menu_answers().retain(|(l, _)| l != label);
	MENU_WAITING.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).retain(|(l, _)| l != label);//an item a window was made for and never asked after, as a video window chosen in Open… never does
}

/// Build the menu bar and make it the application's; lib.rs calls this during setup, on macOS only
pub fn menu_set(app: &AppHandle) -> tauri::Result<()> {
	let menu = Menu::with_items(app, &[
		&Submenu::with_items(app, &app.package_info().name, true, &[//the application menu, named brandName, the product name in tauri.conf.json
			&MenuItem::with_id(app, MENU_ABOUT, format!("About {}", app.package_info().name), true, None::<&str>)?,//fuji's own About, the last section of the settings panel, in place of the panel macOS assembles from Info.plist; Apple's wording for the item, with brandName
			&PredefinedMenuItem::separator(app)?,
			&MenuItem::with_id(app, MENU_SETTINGS, "Settings…", true, Some("CmdOrCtrl+,"))?,//the Mac's own place and word for it since Ventura, with its shortcut, ahead of Services; the page shows the settings panel, which s already does
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::services(app, None)?,
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::hide(app, None)?,
			&PredefinedMenuItem::hide_others(app, None)?,
			&PredefinedMenuItem::show_all(app, None)?,//the hiding group's third, which Tauri's default leaves out; macOS runs all three itself
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::quit(app, None)?,
		])?,
		&Submenu::with_items(app, "File", true, &[
			&MenuItem::with_id(app, MENU_NEW_WINDOW, "New Window", true, Some("CmdOrCtrl+N"))?,
			&MenuItem::with_id(app, MENU_OPEN, "Open…", true, Some("CmdOrCtrl+O"))?,
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::close_window(app, Some("Close"))?,//the Mac's word; muda's default here says Close Window, which is a Windows menu's
		])?,
		&Submenu::with_items(app, MENU_EDIT, true, &[//tauri's default, kept on purpose. These are the standard macOS commands and they reach the web view, which implements all of them for a field that takes typing; one day Copy and Paste will mean files
			&PredefinedMenuItem::undo(app, None)?,
			&PredefinedMenuItem::redo(app, None)?,
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::cut(app, None)?,
			&PredefinedMenuItem::copy(app, None)?,
			&PredefinedMenuItem::paste(app, None)?,
			&PredefinedMenuItem::select_all(app, None)?,
		])?,
		&Submenu::with_items(app, MENU_VIEW, true, &[
			&MenuItem::with_id(app, MENU_FULLSCREEN, "Show Light Table", true, Some("CmdOrCtrl+Ctrl+F"))?,//the title the page sets at launch and on every change of view through menu_text below, Show Light Table from the sheet and Show Contact Sheet from the table, because a Mac item names where it takes you rather than saying toggle. It is fuji's own fullscreen, not the system's, and macOS adds a second item of its own to any menu called View — the essay above fullscreenSet in Shell.vue is the whole subject and worth reading before touching either. ⌃⌘F is the keystroke a mac user already knows, reaching the code a double-click reaches. Spelled this way because muda parses "Cmd" to Modifiers::META and its macos layer only turns Modifiers::SUPER into the command key — so "Ctrl+Cmd+F" silently loses the command and becomes ⌃F. The CmdOrCtrl family is the only spelling that produces command here, which is why the two items above use it
		])?,
		&Submenu::with_id_and_items(app, WINDOW_SUBMENU_ID, "Window", true, &[//tauri's own id for the Window menu, which set_menu below registers with AppKit, so macOS keeps it filled with every open window, named by its title bar: the picture on a table, the folder on the sheet
			&PredefinedMenuItem::minimize(app, None)?,
			&PredefinedMenuItem::maximize(app, None)?,
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::bring_all_to_front(app, None)?,//the item a Mac Window menu ends with, ahead of the window list macOS fills in; Close belongs to File alone
		])?,
		&Submenu::with_id_and_items(app, HELP_SUBMENU_ID, "Help", true, &[//and its id for the Help menu, which set_menu registers the same way, so macOS puts its search field at the top, the one that finds any menu item by name
			&MenuItem::with_id(app, MENU_HELP, format!("{} Help", app.package_info().name), true, None::<&str>)?,//the first item of every Mac Help menu, under the search field, and with no shortcut: with ⇧⌘/ set on it, the key opened this menu at its search field rather than reaching the item. The page opens the help address in the system's browser, fuji's manual; the help panel on h is separate. About belongs under the application menu alone
		])?,
	])?;

	app.set_menu(menu)?;//which registers the Window and Help menus with AppKit too, by the two ids above
	menu_validate_view();//after the menu is the application's, because it asks AppKit for the menu bar and would otherwise find nothing; the essay below is the subject
	if let Some(edit) = menu_submenu(MENU_EDIT) { for item in edit.itemArray().iter() { item.setEnabled(false) } }//every Edit item starts gray, until the page lights it
	Ok(())
}

fn menu_submenu(title: &str) -> Option<objc2::rc::Retained<objc2_app_kit::NSMenu>> {//a submenu of AppKit's own menu bar, by the title it shows; none before the menu is set, or off the main thread, where AppKit's menus live
	let marker = objc2::MainThreadMarker::new()?;
	let bar = objc2_app_kit::NSApplication::sharedApplication(marker).mainMenu()?;
	bar.itemWithTitle(&objc2_foundation::NSString::from_str(title))?.submenu()
}

/// Light or gray one item of the menu bar, by the titles the bar shows for its menu and for it; plain, because AppKit's menus live on the main thread
#[tauri::command]
pub fn menu_enable(menu: String, item: String, enabled: bool) -> Result<(), String> {
	let submenu = menu_submenu(&menu).ok_or(format!("no menu {menu}"))?;
	let found = submenu.itemWithTitle(&objc2_foundation::NSString::from_str(&item)).ok_or(format!("no item {item} in {menu}"))?;
	found.setEnabled(enabled);
	Ok(())
}

/*
Let AppKit keep its own menu item's label honest, which muda otherwise prevents.

macOS puts a second item into the View menu by itself — *Enter Full Screen*, on the Globe+F shortcut — so fuji writes one item there and two appear. That one is meant to retitle itself to *Exit Full Screen* while the window is in a Space, and in fuji it never did.

The retitling is not a mechanism of its own: it is part of menu validation. Before a menu opens, AppKit walks its items and asks the responder chain to validate each, and `NSWindow`'s answer for `toggleFullScreen:` both enables that item and rewrites its title. The walk only happens while the menu's `autoenablesItems` is on, and **muda turns it off on every menu and submenu it builds** — reasonably, since muda tracks each item's enabled state itself and automatic enabling would fight it. The collision exists only because macOS adds an item muda knows nothing about, and that is the one item needing validation. So this turns the flag back on, for the View submenu alone, by setting a property on an AppKit object muda handed over — which keeps it in the same safe category as `ns_window()` rather than the category a custom dock menu would need.

**Fuji's own item is not put at risk**, which was worth checking before writing this. AppKit leaves an item enabled when it has an explicit target that responds to its action and that target does not implement `validateMenuItem:`; muda sets each item's target to the item itself with an action it implements, and implements `validateMenuItem:` nowhere. So fuji's own View item stays enabled, and only the system's item — which has no target and so reaches `NSWindow` — is validated and retitled.

Every way this can fail leaves the label as wrong as it already was and nothing worse: a muda that stops disabling validation makes this a no-op, one that disables it after us puts the stale label back, and a View submenu that cannot be found means doing nothing.
*/
fn menu_validate_view() {
	if let Some(view) = menu_submenu(MENU_VIEW) { view.setAutoenablesItems(true) }
}

/// Act on a chosen menu item: make a window here, or tell a window's page about the ones it owns, the window in front when it answers the item and otherwise the one most recently in front that does
pub fn menu_chosen(app: &AppHandle, event: MenuEvent) {
	let id = event.id().as_ref();
	if id == MENU_NEW_WINDOW { crate::window::window_open(app, vec![]); return }//only rust can make a window, so this one never reaches the page
	if ![MENU_OPEN, MENU_FULLSCREEN, MENU_ABOUT, MENU_SETTINGS, MENU_HELP].contains(&id) { return }//a predefined item macos handles by itself, and not fuji's business

	let front = app.webview_windows().into_values().find(|w| w.is_focused().unwrap_or(false));
	if let Some(window) = front.filter(|w| menu_answered(w.label(), id)) { let _ = app.emit_to(window.label(), "menu", id); return }//the window in front answers it, which a contact sheet's window always does: to that window alone, because the menu bar is shared and the item is not
	let order = menu_front().clone();//copied, so the list is not held while a window comes forward
	if let Some(window) = order.iter().filter(|label| menu_answered(label, id)).find_map(|label| app.get_webview_window(label)) {//the window most recently in front that answers it, brought forward first, so the item is seen to act where it acts
		let _ = window.unminimize();
		let _ = window.set_focus();
		let _ = app.emit_to(window.label(), "menu", id);
		return
	}

	//no window answers it, which on macOS usually means fuji is resident with none open
	if id == MENU_OPEN {//the open box on its own, and a window only for what the user chooses, made to answer Open… as the items below make one to answer them; a cancel leaves no window at all
		let app = app.clone();
		crate::dialog::dialog_open_alone(move |path| { if !path.is_empty() { menu_window(&app, vec![path], MENU_OPEN) } });
		return
	}
	if id == MENU_FULLSCREEN { return }//the View item acts on a window, and there is none to act on
	menu_window(app, vec![], id)//a window, as a click on the dock icon makes, for the item to be answered in
}

//make a window to answer a menu item no window could, holding the item for its page and these files with it
fn menu_window(app: &AppHandle, paths: Vec<String>, id: &str) {
	match crate::window::window_build(app, paths, String::new()) {
		Ok(label) => MENU_WAITING.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).push((label, id.to_string())),//held after the window is built and still before its page can ask: menu events, the open box's answer and the page's calls all arrive on the main thread, and this has it until it returns
		Err(e) => crate::log::log(&format!("menu: could not make a window, {e}")),
	}
}

/// The item this window was made to answer, chosen while no window was open, taken away as it answers; blank for a window made any other way
#[tauri::command]
pub fn menu_waiting(window: WebviewWindow) -> String {
	let mut waiting = MENU_WAITING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());//take the list even if a previous holder panicked
	let Some(at) = waiting.iter().position(|(label, _)| label == window.label()) else { return String::new() };
	waiting.remove(at).1
}

/// Retitle one of fuji's own items; the page calls this as a window's view changes, because the menu bar cannot see what a window shows, and the View item names where it takes you
#[tauri::command]
pub fn menu_text(app: AppHandle, id: String, text: String) -> Result<(), String> {
	let menu = app.menu().ok_or("no menu bar")?;
	for kind in menu.items().map_err(|e| e.to_string())? {//the bar is submenus and each holds items, two levels, which is the whole of the menu above; tauri's get looks one level down and no further
		let Some(submenu) = kind.as_submenu() else { continue };
		if let Some(item) = submenu.get(id.as_str()).and_then(|k| k.as_menuitem().cloned()) { return item.set_text(text).map_err(|e| e.to_string()) }
	}
	Err(format!("no menu item {id}"))
}
