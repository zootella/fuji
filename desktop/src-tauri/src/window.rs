use std::fs::{File, OpenOptions, TryLockError};
use std::sync::{Mutex, OnceLock};
#[cfg(target_os = "macos")]
use std::ffi::c_void;//the GL context and the layer, which are pointers to the Mac and addresses to fuji
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::open;

/*
Making fuji's windows, saying where one is, and deciding how long fuji outlives them. This is here rather than in tauri.conf.json because a window is no longer a thing there is exactly one of: each is made when it is asked for, under a label counted for it.

**How many, and who asks for one.** A window is made for the pictures it will show, and `window_build` is the only way one comes into being. Every launch on every platform makes one through `window_first`, when the application loop is ready, for whatever the command line handed over — unless a double-click on the mac has already made one, which `window_first` explains. On macOS a window is made again whenever the running application is told to open something, and again when the user clicks a dock icon with no windows behind it. Windows and Linux never make a second one for the operating system, because their shells start another process instead. On every platform the page can ask for another window through `window_new`, handing it a note its own page reads as it mounts, which is how a video window comes to sit beside the contact sheet: so a process on Windows and Linux holds one window for a launch and as many more as the page asks for, a process on the Mac holds any number, and the same function serves them all without knowing which.

**Why the platforms differ at all, since a process per window everywhere would be simpler.** Because the macOS Dock will not have it. Measured on the Mac mini, macOS 15.7.4, 2026-09-13: two running copies of one bundle appear to LaunchServices as two foreground applications — separate serial numbers under a single bundle identifier — and the Dock draws a tile per foreground application rather than per bundle. So several fujis would be a row of identical icons with nothing able to merge them. Windows and Linux hide the same architecture — the Windows taskbar combines buttons for one executable, and GNOME and KDE group by `WM_CLASS` — so macOS is the only shell of the three that needs code.

**What it is called.** Each window gets a label of its own, counted from one, and `capabilities/default.json` grants permissions to the pattern rather than to a name. A window whose label the capability does not match is built and then silently has no permissions at all, which looks like a page that loads and cannot do anything, so the two have to be kept in step.

**Where, and how big, is the page's to decide.** Every window is built hidden at whatever size Tauri defaults to, and its page places it before the reveal: the shell fits a preview around its picture, or places the contact sheet at the size the user last gave one or a preset portion of the work area, somewhere at random inside it, by the rule library.js keeps, and a video window places itself at its own size by the same rule. Rust reads none of it: the page has the settings file before it places anything. The window is hidden while this happens, so nothing flashes and nothing jumps.

Fuji places its windows itself because neither platform does it well. Windows cascades new ones down a fixed staircase without checking that they fit — measured on 2026-09-13, three windows 1062 tall on a work area 1160 deep, the last two hanging 6 and 40 under the taskbar. A work area 1160 deep on that 1200-row screen is what 100 percent leaves, so css and backing pixels were one and the numbers need no unit. macOS places nothing at all, and tao centers every window it is given no position for, so two windows of one size stack exactly on top of each other.

**How long fuji outlives its last window** is the one place fuji deliberately behaves differently on each platform, and `window_stays_resident` below is the whole of it. On Windows and Linux, closing a process's last window closes fuji, which is what those desktops mean by closing a window. On macOS an application is a place the user is in rather than a window they have open: the dock icon stays, with its dot, which is how a Mac user reopens it, learns they can keep it there, and decides what to quit when the machine is busy. Going with that grain costs almost nothing here, because a fuji with no windows has destroyed its webviews and is the Rust host alone — and the Rust is dumb, so it is doing nothing.
*/

/*
Copies of fuji that start together.

Windows and Linux start a copy of fuji for each picture opened, and Explorer starts one for every picture a user selects and opens with Enter, all within a fraction of a second. On Windows those copies race: WebView2 lets a second process into the data folder they share only once the first has its browser running, and refuses one that asks sooner. Measured on the Windows box on 2026-09-28, four copies started in the same millisecond made one window between them, as two copies of Microsoft's own sample app do. And the refusal was silent. Tauri answers a request for a window before trying it, then drops the half-made window without a word fuji can hear, so the copies that lost sat idle forever, with no window to close and nothing to end them.

Three rules answer it. A copy's first window takes a turn, holding a lock on a file until its page begins to load, when the web engine is up and the next copy can join it. A flurry opens one window: the copy that takes the turn stamps the file with its launch moment, as the file's modified time, which other copies can read through the lock, and a copy launched within WINDOW_FLURRY of the stamp leaves before making anything. A user who opens a folder's worth of pictures has one to look at, and the rest a flip or a contact sheet away; a copy launched later is a deliberate second launch, and waits its turn for a window of its own. And a copy whose first page never begins to load exits after WINDOW_ARRIVAL_WAIT, so whatever goes wrong, nothing is left running without a window. The one failure that last rule cannot see is a page that begins to load and then never reveals its window, since arriving is all it checks; none has been seen. Tauri's page load was seen reaching the watchdog with WebView2 on the Windows box and with WKWebView on the Mac mini on 2026-09-30, where a development window stayed past the twenty seconds; WebKitGTK is still to be seen, and win2linux.md asks.

The mac opens a selection as one window already, since Finder hands it over in one event: seen on the Mac mini on 2026-09-30, five pictures opened together making one window whether fuji was running or not. A process's later windows never take a turn, on the Mac or anywhere a page asks for one, because they join a web engine already running in the same process.
*/
const WINDOW_TURN_FILE: &str = "window-turn.lock";//empty, in the app's local data folder beside the web engine's own; its modified time is the stamp
const WINDOW_TURN_WAIT: Duration = Duration::from_secs(4);//the longest a deliberate launch waits behind another copy's turn before going ahead regardless: meant to outlast a cold start on a slow machine, which is reasoned rather than measured, and harmless when long, since a waiting copy goes the moment the turn is free
const WINDOW_FLURRY: Duration = Duration::from_millis(200);//how close two launches must be to count as one flurry. Explorer started five copies within a tenth of a second, and fifteen across about 430 ms, which made two windows and is fine; a person's second double-click comes later than this
const WINDOW_TURN_POLL: Duration = Duration::from_millis(20);//how often a waiting copy looks at the stamp and tries the lock, about a frame, so a flurry clears and a turn passes on unseen
const WINDOW_ARRIVAL_WAIT: Duration = Duration::from_secs(20);//how long a copy's first page has to begin loading before the copy takes its window to be lost and exits: long, because it ends a process, and a cold start on a slow machine can take seconds

static WINDOW_COUNT: AtomicUsize = AtomicUsize::new(0);//how many windows this process has ever made, which is where the next label comes from
static WINDOW_TURN: Mutex<Option<File>> = Mutex::new(None);//the locked file while this copy has the turn; dropping it lets the next copy go
static WINDOW_ARRIVED: AtomicBool = AtomicBool::new(false);//whether a page in this process has begun to load
static WINDOW_LAUNCHED: OnceLock<SystemTime> = OnceLock::new();//when this copy started, which window_flurry compares with the stamp
static WINDOW_NOTES: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());//the note each window was made with, by label, until its page takes it; a handful at most, so a list rather than a map
fn window_label() -> String { format!("window-{}", WINDOW_COUNT.fetch_add(1, Ordering::Relaxed) + 1) }//counted rather than reused, so a label never names two windows even after one closes; capabilities/default.json grants to window-*
fn window_notes() -> std::sync::MutexGuard<'static, Vec<(String, String)>> { WINDOW_NOTES.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) }//take the list even if a previous holder panicked

/// Make a hidden window for these pictures, holding this note for its page, for the page to place and reveal, and answer its label, for a caller with something else to hold for it
pub fn window_build(app: &AppHandle, paths: Vec<String>, note: String) -> tauri::Result<String> {
	let label = window_label();
	open::open_hold(app, &label, paths);//before the window exists, so its page finds them the moment it mounts and asks
	if !note.is_empty() { window_notes().push((label.clone(), note)) }//the same, for the note
	let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::default())
		.title(&app.package_info().name)//brandName, the product name from tauri.conf.json, until the page titles the window for what it shows
		.visible(false)//the page places it and then shows it, once it has something to draw
		.fullscreen(false)
		.on_page_load(|_, _| window_arrived())//the one sign a window really came, since build answers before anything is tried
		.build()?;
	window_browser_keys_off(&window);
	Ok(label)
}

/// Make fuji's first window, unless something has already made one, which is how a launch produces exactly one window however it was started
pub fn window_first(app: &AppHandle, paths: Vec<String>) {
	if !app.webview_windows().is_empty() { return }//a picture opened by double-click has already been given a window: measured on the Mac mini 2026-09-14, macOS delivers that event 39 milliseconds before setup() even runs and 52 before the one that calls this, so by now it is done. Only the second number is load-bearing; the first is worth knowing because a double-click builds fuji's first window before fuji has finished setting itself up, which is the opposite of what anyone adding to setup() would assume. Making a window in setup instead of here is what used to open two, one of them blank
	let app = app.clone();
	std::thread::spawn(move || {//nothing has, so this is an ordinary launch: empty from the dock, or on the pictures the command line carried, which is how windows and linux hand over a double-click. On a thread of its own, because taking a turn can mean waiting, and the main thread has to stay free to make the window
		match window_turn(&app) {//the essay above says why
			WindowTurn::Flurry => { app.exit(0); return }//launched with the copy that won, which is already opening the one window, so leave having made nothing
			WindowTurn::Taken(file) => *window_turn_held() = Some(file),//held until the first page begins to load
			WindowTurn::Without => {}
		}
		window_open(&app, paths);
		std::thread::sleep(WINDOW_ARRIVAL_WAIT);
		if !WINDOW_ARRIVED.load(Ordering::Relaxed) { window_lost(&app) }//whichever way it failed, including the way tauri reports to nobody
	});
}

//what waiting for a turn came to
enum WindowTurn {
	Taken(File),//this copy holds the turn, and has stamped the file with its launch moment
	Flurry,//a copy launched with this one holds the turn, or held it a moment ago, and is opening the one window
	Without,//no turn to be had, after WINDOW_TURN_WAIT or for want of the file, so the copy goes ahead regardless
}

//wait for this copy's turn to bring its web engine up, looking at the stamp before every try, so a copy in a flurry leaves the moment it can tell rather than when the winner is done
fn window_turn(app: &AppHandle) -> WindowTurn {
	let Some(file) = window_turn_file(app) else { return WindowTurn::Without };
	let began = Instant::now();
	loop {
		if window_flurry(&file) { return WindowTurn::Flurry }
		match file.try_lock() {
			Ok(()) => {
				if window_flurry(&file) { return WindowTurn::Flurry }//the winner can stamp and let go between the look above and this lock
				let _ = file.set_modified(window_launched());//the stamp is this copy's now; a copy that cannot stamp still opens its window, and only a flurry behind it goes unrecognized
				return WindowTurn::Taken(file)
			}
			Err(TryLockError::WouldBlock) if began.elapsed() < WINDOW_TURN_WAIT => std::thread::sleep(WINDOW_TURN_POLL),
			Err(_) => return WindowTurn::Without,//the whole wait gone by, or the lock itself refused
		}
	}
}

//the turn file, opened to be locked and stamped. One made new is stamped back to 1970 at once, since its own creation time would read as a fresh stamp and the copy that made it would leave as if another had won
fn window_turn_file(app: &AppHandle) -> Option<File> {
	let folder = app.path().app_local_data_dir().ok()?;
	std::fs::create_dir_all(&folder).ok()?;//a first launch arrives before the web engine has made it
	let path = folder.join(WINDOW_TURN_FILE);
	match OpenOptions::new().write(true).create_new(true).open(&path) {
		Ok(file) => { let _ = file.set_modified(UNIX_EPOCH); Some(file) }
		Err(_) => OpenOptions::new().write(true).open(&path).ok(),//there already, which is every launch but the first
	}
}
fn window_turn_held() -> std::sync::MutexGuard<'static, Option<File>> { WINDOW_TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) }//take the turn even if a previous holder panicked, as log.rs does

//whether the copy that last took the turn was launched within WINDOW_FLURRY of this one, either side, since the copy that won need not be the one that started first. The stamp is the file's modified time rather than anything written in it, because windows lets no other process read a locked file's contents but lets any process read its times, which the Windows box confirmed
fn window_flurry(file: &File) -> bool {
	let Ok(stamp) = file.metadata().and_then(|m| m.modified()) else { return false };
	let gap = window_launched().duration_since(stamp).unwrap_or_else(|earlier| earlier.duration());
	gap <= WINDOW_FLURRY
}

/// Note the moment this copy started; the run calls this before anything else, so a machine busy starting fifteen copies still records when each began rather than when tauri got round to it
pub fn window_launch() { window_launched(); }
pub(crate) fn window_launched() -> SystemTime { *WINDOW_LAUNCHED.get_or_init(SystemTime::now) }//pub for uptime.rs, which reads it as main's own clock

//a page has begun to load, so the web engine is up: the first window came, and the next copy can take its turn
fn window_arrived() {
	WINDOW_ARRIVED.store(true, Ordering::Relaxed);
	window_turn_held().take();//dropping the file lets go of the lock
}

//the first window is not coming: let the next copy take its turn, and leave, where a copy with no window has nothing to show and nothing that would ever end it
fn window_lost(app: &AppHandle) {
	eprintln!("window: the first window's page never began to load, so this copy is leaving");//for a terminal running fuji; the log belongs to a page, and no page came
	window_turn_held().take();
	if !window_stays_resident() { app.exit(1) }//the mac keeps its process resident on purpose, window or not
}

/// Make a window and report trouble to the log rather than to a caller who has nowhere to put it; the run event closure is that caller
pub fn window_open(app: &AppHandle, paths: Vec<String>) {
	if let Err(e) = window_build(app, paths, String::new()) { crate::log::log(&format!("window: could not make a window, {e}")) }//a window that never appears is worth a line, and there is nobody above to tell
}

/// Make another window of this application, holding this note for its page to take as it mounts: the page that asks chooses the word and the page that mounts reads it, and nothing here looks at it. Async, because Tauri's documentation warns that building a window from a plain command deadlocks on Windows, and the body runs on the blocking pool like every command that waits
#[command]
pub async fn window_new(app: AppHandle, note: String) -> Result<(), String> {
	crate::run_blocking(move || window_build(&app, vec![], note).map(|_| ()).map_err(|e| e.to_string())).await
}

/// The note this window was made with, taken away as it answers; blank for a window made without one, which is every window the operating system, the dock or the menu bar asks for
#[command]
pub fn window_note(window: WebviewWindow) -> String {
	let mut notes = window_notes();
	let Some(at) = notes.iter().position(|(label, _)| label == window.label()) else { return String::new() };
	notes.remove(at).1
}

/// Does fuji keep running once its last window has closed; the essay above is why the platforms answer differently
#[cfg(all(target_os = "macos", not(debug_assertions)))]
pub fn window_stays_resident() -> bool { true }
#[cfg(not(all(target_os = "macos", not(debug_assertions))))]
pub fn window_stays_resident() -> bool { false }//and a debug build answers no on the mac as well. pnpm local runs the binary out of target/debug rather than a bundle, so there is no dock tile a user could click to ask for a window back, and closing the window is how a development run is meant to end

/*
A browser's shortcut keys, which no window of fuji answers.

A web view is a browser, and on Windows WebView2 keeps a browser's keys: F5 and Ctrl+R reload the page, Ctrl+P prints it, Ctrl+F opens a find bar, and Alt+Left goes back. Neither wry nor Tauri turns them off. None of them belongs in a desktop application, and a reload is worse than useless here: it runs the shell's startup again in a window already placed and showing, and the pictures that window was made for are gone, since open.rs hands them over once. So as each window is made, in a release build, a handler goes on WebView2's key event, which every key the browser might take for its own passes through first, and tells the browser to skip each one. The page still receives every key, and which keys count stays Microsoft's list rather than one kept here.

WebView2 also has a single setting that turns them all off, and it would be simpler, but it takes effect only at the next navigation. By the time Tauri hands the web view over the first page is already loading, and fuji never navigates again, so its windows would keep the keys. A development build keeps them on purpose, for reloading and the developer tools. The web views on macOS and Linux have no such keys. The right-click menu is a browser's on every platform, and the shell turns it away, with the same code everywhere.
*/
#[cfg(target_os = "windows")]
fn window_browser_keys_off(window: &WebviewWindow) {
	use webview2_com::AcceleratorKeyPressedEventHandler;
	use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2AcceleratorKeyPressedEventArgs2;
	use webview2_core::Interface;//for cast, from the windows-core webview2-com is built on rather than fuji's own, under the name Cargo.toml gives it and explains

	if cfg!(debug_assertions) { return }//a development build keeps them, for reloading the page and opening the developer tools
	let _ = window.with_webview(|webview| unsafe {//on the main thread, where the web view lives
		let handler = AcceleratorKeyPressedEventHandler::create(Box::new(|_, args| {//every key the browser might take for its own comes through here first
			if let Some(args) = args.and_then(|args| args.cast::<ICoreWebView2AcceleratorKeyPressedEventArgs2>().ok()) { args.SetIsBrowserAcceleratorKeyEnabled(false)?; }//skip the browser's handling of this key; the page still gets it, and editing keys like ctrl+c were never the browser's. The newer interface arrived in webview2 1.0.2210, and on a runtime older than that the cast fails and the key stays the browser's
			Ok(())
		}));
		let mut token = 0;//what removing the handler would take; it stays for the life of the window
		let _ = webview.controller().add_AcceleratorKeyPressed(&handler, &mut token);//a failure leaves the keys on, the way they started
	});
}
#[cfg(not(target_os = "windows"))]
fn window_browser_keys_off(_window: &WebviewWindow) {}//the web views on macOS and Linux have no browser shortcut keys to turn off

/// The version of the web engine that draws fuji's windows, as the platform reports it: WebKit's on the Mac, the WebView2 runtime's on Windows, and WebKitGTK's on Linux. The page names the engine, since which one it is never changes on a platform
#[command]
pub fn window_engine() -> Result<String, String> { tauri::webview_version().map_err(|e| e.to_string()) }//the error is wry's rather than tauri's, so it is turned into text here as the other modules' are

/// Stop drawing this window's title in its title bar, which stays, with its buttons; the title itself stays set, for the menus and taskbars that list windows by it. The Mac's alone, where a title bar can keep its buttons without its title, and where tao and tauri hide a title only as a window is built; nothing elsewhere
#[command]
pub fn window_title_hide(window: WebviewWindow) -> tauri::Result<()> {
	#[cfg(target_os = "macos")] {
		let ns_window: &objc2_app_kit::NSWindow = unsafe { &*(window.ns_window()? as *const objc2_app_kit::NSWindow) };//the NSWindow tauri built, read on the main thread, where a plain command runs and AppKit wants it
		ns_window.setTitleVisibility(objc2_app_kit::NSWindowTitleVisibility::Hidden);
	}
	#[cfg(not(target_os = "macos"))]
	let _ = window;
	Ok(())
}

/*
Past the edges of the screen, on the Mac.

AppKit keeps a window at the normal level clear of the menu bar, and cuts a resizable window with a title bar to the screen's height, in NSWindow's constrainFrameRect:toScreen:, which it calls as a window is shown, moved and resized, and which Apple's AppKit notes for 10.9 invite a subclass to override "to adjust or prevent this constraining". tao's window class, TaoWindow, has no override, so the first window_constraint_lift adds one to that class at runtime, which gives every window NSWindow's own answer but those on the list the command keeps. Giving one window a subclass of fuji's own is the usual road and is closed here: tao's sendEvent: finds its superclass from the window's class at runtime, and WebKit has already swapped that class for key-value observing. A window on the list also loses AppKit's rescue when its display goes away.

What the override frees is a window fuji places itself. The Mac's own drag still stops at the menu bar with the override in place, seen on the Mac mini on 2026-10-21, because WindowServer runs that drag and holds the window there by itself, and AppKit takes WindowServer's frame as it is while a drag runs, read in a disassembly of AppKit on Sequoia. The first refusals of a run go to the log, as the override's evidence that AppKit asked.
*/
#[cfg(target_os = "macos")]
static WINDOW_UNCONSTRAINED: Mutex<Vec<(String, usize)>> = Mutex::new(Vec::new());//each window the override answers for, by label, and its NSWindow's address, which is all the override is handed
#[cfg(target_os = "macos")]
fn window_unconstrained() -> std::sync::MutexGuard<'static, Vec<(String, usize)>> { WINDOW_UNCONSTRAINED.lock().unwrap_or_else(|p| p.into_inner()) }

/// Let this window go anywhere and be any size, past every edge of the screen and over the menu bar, with AppKit's constraint lifted for it alone; the Mac's alone, nothing elsewhere
#[command]
pub fn window_constraint_lift(window: WebviewWindow) -> Result<(), String> {
	#[cfg(target_os = "macos")] {
		static ADDED: OnceLock<bool> = OnceLock::new();
		if !*ADDED.get_or_init(window_constraint_add) { return Err("the override could not be added to tao's window class".into()) }
		let address = window.ns_window().map_err(|e| e.to_string())? as usize;//the NSWindow tauri built, on the main thread, where a plain command runs
		let mut list = window_unconstrained();
		if !list.iter().any(|(label, _)| label == window.label()) { list.push((window.label().to_string(), address)) }
	}
	#[cfg(not(target_os = "macos"))]
	let _ = window;
	Ok(())
}

/// The window with this label is closing: off the override's list, so a window made later at the same address is constrained as usual
#[cfg(target_os = "macos")]
pub fn window_constraint_closed(label: &str) { window_unconstrained().retain(|(l, _)| l != label) }
#[cfg(not(target_os = "macos"))]
pub fn window_constraint_closed(_label: &str) {}

#[cfg(target_os = "macos")]
fn window_constraint_add() -> bool {//the override, added to TaoWindow under NSWindow's own type encoding for the method; false where the class is missing or already has one, which a later tao might add
	use objc2::{sel, ClassType};
	use objc2::ffi::{class_addMethod, class_getInstanceMethod, method_getTypeEncoding, objc_getClass};
	let selector = sel!(constrainFrameRect:toScreen:);
	unsafe {
		let tao = objc_getClass(c"TaoWindow".as_ptr());
		let method = class_getInstanceMethod(objc2_app_kit::NSWindow::class(), selector);
		if tao.is_null() || method.is_null() { return false }
		let imp: objc2::runtime::Imp = std::mem::transmute(window_constrain as extern "C-unwind" fn(_, _, _, _) -> _);
		class_addMethod(tao as *mut _, selector, imp, method_getTypeEncoding(method)).as_bool()
	}
}

#[cfg(target_os = "macos")]
extern "C-unwind" fn window_constrain(this: &objc2::runtime::AnyObject, _command: objc2::runtime::Sel, frame: objc2_foundation::NSRect, screen: *mut objc2::runtime::AnyObject) -> objc2_foundation::NSRect {//AppKit asking where a window may go: exactly where it was asked to, for a window on the list, and NSWindow's own answer for every other, through NSWindow by name rather than by the window's class, which WebKit swapped
	use objc2::{msg_send, ClassType};
	let own: objc2_foundation::NSRect = unsafe { msg_send![super(this, objc2_app_kit::NSWindow::class()), constrainFrameRect: frame, toScreen: screen] };
	if !window_unconstrained().iter().any(|(_, address)| *address == this as *const _ as usize) { return own }
	if own != frame { window_constraint_said(frame, own) }
	frame
}

#[cfg(target_os = "macos")]
fn window_constraint_said(asked: objc2_foundation::NSRect, own: objc2_foundation::NSRect) {//a line in the log for each of the first twenty times in a run the override left a window where AppKit would have moved it
	static SAID: AtomicUsize = AtomicUsize::new(0);
	if SAID.fetch_add(1, Ordering::Relaxed) >= 20 { return }
	let said = |r: objc2_foundation::NSRect| format!("{},{} {}x{}", r.origin.x, r.origin.y, r.size.width, r.size.height);
	crate::log::log(&format!("window: AppKit would have moved a window from {} to {}, in points from the bottom left, and the override left it", said(asked), said(own)));
}

/*
A drag fuji runs itself.

window_drag moves a window with the pointer from the moment the page asks, while the left button is down, until it comes up, placing the window itself on every move rather than handing the drag to the platform. That is what lets a window go past the top of the screen: the Mac's own drag is WindowServer's, which holds a window below the menu bar whatever AppKit allows, and Windows' own move loop keeps a window's top on the screen, while a window its application places goes where it is put, on the Mac once window_constraint_lift has freed it. It also leaves out what the platforms' drags bring, Snap and shake on Windows, and tiling and the move into another Space on the Mac. Keeping the window on the screen instead, with its contents rolling up inside it at the top, was weighed and set aside, since it needs a drag of fuji's own as well and breaking past the top worked; the one cost expected of a drag fuji runs, that the window would trail the pointer, did not come, and on the Mac mini it was as smooth as the Mac's own.

On the Mac a monitor of the application's mouse events, installed with the first drag and kept for the life of the process as touch.rs keeps its own, follows each drag event with AppKit's own reading of where the pointer is, which a window moving under it cannot throw off, and moves the window's frame at once on the main thread, the way Chromium drags a tab; a window in a macOS Space stays where it is, as under the Mac's own drag. On Windows the window takes the mouse, and a subclass on its window procedure places it on every move, until the button comes up or another window takes the mouse; a maximized window takes Windows' own drag instead, which restores it as the drag begins. Linux takes the desktop's own drag always, since Wayland lets no application place its own window.
*/
#[cfg(target_os = "macos")]
static WINDOW_DRAG: Mutex<Option<(WebviewWindow, (f64, f64), (f64, f64))>> = Mutex::new(None);//the window being dragged, and where the pointer and the window's corner were as the drag began, in points from the bottom left of the main screen; none between drags
#[cfg(target_os = "windows")]
static WINDOW_DRAG: Mutex<Option<((i32, i32), (i32, i32))>> = Mutex::new(None);//where the pointer and the dragged window's outer corner were as the drag began, in backing pixels; none between drags, and the window is the one the subclass is on
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn window_drag_held<T>(drag: &'static Mutex<T>) -> std::sync::MutexGuard<'static, T> { drag.lock().unwrap_or_else(|p| p.into_inner()) }//take the drag even if a previous holder panicked

/// Move this window with the pointer until the left button comes up, fuji placing it on every move, so it goes past the screen's edges and the menu bar where the platform's own drag would stop it; on Linux, the desktop's own drag
#[cfg(target_os = "macos")]
#[command]
pub fn window_drag(window: WebviewWindow) -> Result<(), String> {
	use objc2_app_kit::{NSEvent, NSWindow};
	static MONITOR: OnceLock<()> = OnceLock::new();
	MONITOR.get_or_init(window_drag_monitor);
	if NSEvent::pressedMouseButtons() & 1 == 0 { return Ok(()) }//the button already up, so there is no drag to follow
	let ns_window: &NSWindow = unsafe { &*(window.ns_window().map_err(|e| e.to_string())? as *const NSWindow) };//the NSWindow tauri built, on the main thread, where a plain command runs
	if ns_window.styleMask().contains(objc2_app_kit::NSWindowStyleMask::FullScreen) { return Ok(()) }//in a macOS Space, which a drag leaves where it is, as the Mac's own drag does
	let (pointer, corner) = (NSEvent::mouseLocation(), ns_window.frame().origin);
	*window_drag_held(&WINDOW_DRAG) = Some((window, (pointer.x, pointer.y), (corner.x, corner.y)));
	Ok(())
}
#[cfg(target_os = "macos")]
fn window_drag_monitor() {//every left drag and release in the application, before any window sees it, passed on unchanged: a drag moves the window being dragged, if there is one, and a release ends it
	use block2::RcBlock;
	use objc2_app_kit::{NSEvent, NSEventMask, NSEventType, NSWindow};
	use objc2_foundation::NSPoint;
	let block = RcBlock::new(|event: std::ptr::NonNull<NSEvent>| -> *mut NSEvent {//appkit calls this on the main thread
		let kind = unsafe { event.as_ref() }.r#type();
		if kind == NSEventType::LeftMouseUp { window_drag_held(&WINDOW_DRAG).take(); }
		if kind == NSEventType::LeftMouseDragged {
			let drag = window_drag_held(&WINDOW_DRAG).clone();//let go of the lock before appkit moves anything
			if let Some((window, from, corner)) = drag {
				if let Ok(address) = window.ns_window() {//none once the window has gone, and then nothing moves
					let ns_window: &NSWindow = unsafe { &*(address as *const NSWindow) };
					let pointer = NSEvent::mouseLocation();
					ns_window.setFrameOrigin(NSPoint::new(corner.0 + pointer.x - from.0, corner.1 + pointer.y - from.1));//at once, since this is the main thread, where tao's own set_position would wait a turn
				}
			}
		}
		event.as_ptr()
	});
	let token = unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::LeftMouseDragged | NSEventMask::LeftMouseUp, &block) };
	std::mem::forget(token); std::mem::forget(block);//the monitor lives as long as the process, as touch.rs's does
}

#[cfg(target_os = "windows")]
const WINDOW_DRAG_SUBCLASS: usize = 0x6472_6167;//the id the drag's subclass goes on the window under, its own, so taking it off touches no other
#[cfg(target_os = "windows")]
#[command]
pub fn window_drag(window: WebviewWindow) -> Result<(), String> {
	use windows::Win32::Foundation::{HWND, POINT, RECT};
	use windows::Win32::UI::Input::KeyboardAndMouse::SetCapture;
	use windows::Win32::UI::Shell::SetWindowSubclass;
	use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetWindowRect, IsZoomed};
	let h = HWND(window.hwnd().map_err(|e| e.to_string())?.0);
	if unsafe { IsZoomed(h) }.as_bool() { return window.start_dragging().map_err(|e| e.to_string()) }//maximized: Windows' own drag, which restores the window as it begins, as a title bar's does
	let (mut pointer, mut outer) = (POINT::default(), RECT::default());
	unsafe { GetCursorPos(&mut pointer) }.map_err(|e| e.to_string())?;
	unsafe { GetWindowRect(h, &mut outer) }.map_err(|e| e.to_string())?;//the outer rectangle, invisible border and all, which the drag moves by the pointer's distance and so needs no visible frame
	*window_drag_held(&WINDOW_DRAG) = Some(((pointer.x, pointer.y), (outer.left, outer.top)));
	if !unsafe { SetWindowSubclass(h, Some(window_drag_proc), WINDOW_DRAG_SUBCLASS, 0) }.as_bool() { window_drag_held(&WINDOW_DRAG).take(); return Err("the drag could not watch the window".into()) }
	unsafe { SetCapture(h) };//the mouse is this window's until the button comes up, wherever the pointer goes; a button already up shows on the first move, which ends the drag
	Ok(())
}
#[cfg(target_os = "windows")]
unsafe extern "system" fn window_drag_proc(h: windows::Win32::Foundation::HWND, message: u32, w: windows::Win32::Foundation::WPARAM, l: windows::Win32::Foundation::LPARAM, _id: usize, _data: usize) -> windows::Win32::Foundation::LRESULT {//the dragged window's messages while the drag lasts, each passed on after
	use windows::Win32::Foundation::POINT;
	use windows::Win32::System::SystemServices::MK_LBUTTON;
	use windows::Win32::UI::Shell::DefSubclassProc;
	use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetWindowPos, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, WM_CAPTURECHANGED, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCDESTROY};
	let held = w.0 & MK_LBUTTON.0 as usize != 0;//the logical left button, which follows the user's choice to swap the buttons
	match message {
		WM_MOUSEMOVE if held => {
			let drag = *window_drag_held(&WINDOW_DRAG);
			let mut pointer = POINT::default();
			if let (Some((from, corner)), Ok(())) = (drag, unsafe { GetCursorPos(&mut pointer) }) {
				let _ = unsafe { SetWindowPos(h, None, corner.0 + pointer.x - from.0, corner.1 + pointer.y - from.1, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE) };//at once, on the thread that owns the window
			}
		}
		WM_MOUSEMOVE | WM_LBUTTONUP => window_drag_end(h, true),//the button is up, whether the release came or a move without it
		WM_CAPTURECHANGED | WM_NCDESTROY => window_drag_end(h, false),//another window took the mouse, so the drag is over and there is nothing to release; or the window is going, and Microsoft asks that a subclass come off before it does
		_ => {}
	}
	unsafe { DefSubclassProc(h, message, w, l) }
}
#[cfg(target_os = "windows")]
fn window_drag_end(h: windows::Win32::Foundation::HWND, release: bool) {//the subclass off first, so the release's own WM_CAPTURECHANGED finds nothing of the drag's
	use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
	use windows::Win32::UI::Shell::RemoveWindowSubclass;
	window_drag_held(&WINDOW_DRAG).take();
	let _ = unsafe { RemoveWindowSubclass(h, Some(window_drag_proc), WINDOW_DRAG_SUBCLASS) };
	if release { let _ = unsafe { ReleaseCapture() }; }
}

#[cfg(target_os = "linux")]
#[command]
pub fn window_drag(window: WebviewWindow) -> Result<(), String> { window.start_dragging().map_err(|e| e.to_string()) }

/*
Fuji's own fullscreen, and what the windows in it share.

Fuji's fullscreen is tao's simple fullscreen, and the page reaches it only through the two commands below, in and out. On the Mac it hides the menu bar and the dock as well, and those belong to the application rather than to a window. tao saves what they were doing as a window goes in and puts that back as the same window comes out, read in tao 0.35.3, which is right for one window and wrong for two: the second window in saves them hidden, so a first that comes out before it brings the menu bar and dock back over a window still in fuji's fullscreen, and the second coming out then hides them again with no window in it, for the rest of the run. So Rust keeps the windows in fuji's fullscreen and answers for the application: while any window is still in it the menu bar and dock stay as they are, and as the last one comes out they go back to what they were doing before the first went in. A window that closes without coming out comes off the list as it goes, so the last one out is never missed. Windows and Linux have nothing to share, since there it is the platform's ordinary fullscreen, which keeps nothing for the application.
*/
#[cfg(target_os = "macos")]
static WINDOW_FULLSCREEN: Mutex<(Vec<String>, objc2_app_kit::NSApplicationPresentationOptions)> = Mutex::new((Vec::new(), objc2_app_kit::NSApplicationPresentationOptions::Default));//the label of every window in fuji's fullscreen, and what the menu bar and dock were doing before the first of them went in
#[cfg(target_os = "macos")]
fn window_fullscreen() -> std::sync::MutexGuard<'static, (Vec<String>, objc2_app_kit::NSApplicationPresentationOptions)> { WINDOW_FULLSCREEN.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) }//take the list even if a previous holder panicked
#[cfg(target_os = "macos")]
fn window_application() -> Option<objc2::rc::Retained<objc2_app_kit::NSApplication>> { objc2::MainThreadMarker::new().map(objc2_app_kit::NSApplication::sharedApplication) }//AppKit's application, which holds what the menu bar and dock are doing; none off the main thread, where every caller here already is

/// Put the window in fuji's own fullscreen, tao's simple one: the screen it is on, at once, with no Space and no animation, and on Windows and Linux the ordinary fullscreen
#[command]
pub fn window_fullscreen_enter(window: WebviewWindow) -> tauri::Result<()> {
	#[cfg(target_os = "macos")] {
		let mut fullscreen = window_fullscreen();
		if fullscreen.0.is_empty() { if let Some(app) = window_application() { fullscreen.1 = app.presentationOptions() } }//the first window in, so what the menu bar and dock are doing now is what the last one out gives back
		if !fullscreen.0.iter().any(|label| label == window.label()) { fullscreen.0.push(window.label().to_string()) }
	}
	window.set_simple_fullscreen(true)
}

/// Take the window out of fullscreen without changing whether it is showing, which tao's own way does everywhere but windows
#[command]
pub fn window_fullscreen_leave(window: WebviewWindow) -> tauri::Result<()> {
	#[cfg(target_os = "windows")] {
		let w = window.clone();
		return window.run_on_main_thread(move || window_fullscreen_leave_hidden(&w))//one task on the main thread, where tauri makes each change at once rather than queuing it, so nothing can come between the restore and the hide
	}
	#[cfg(target_os = "macos")]
	let during = window_application().map(|app| app.presentationOptions());//what the menu bar and dock are doing with this window in, which tao is about to undo
	#[cfg(not(target_os = "windows"))]
	window.set_simple_fullscreen(false)?;//at once, since a plain command runs on the main thread, so on the mac tao has put back what it saved before the line below looks
	#[cfg(target_os = "macos")]
	if let Some(during) = during { window_fullscreen_out(window.label(), during) }
	#[cfg(not(target_os = "windows"))]
	Ok(())
}

//this window has come out of fuji's fullscreen, or is closing: off the list, and the menu bar and dock left as they were while it was in, or, if it was the last, put back as they were before the first went in
#[cfg(target_os = "macos")]
fn window_fullscreen_out(label: &str, during: objc2_app_kit::NSApplicationPresentationOptions) {
	let mut fullscreen = window_fullscreen();
	let Some(at) = fullscreen.0.iter().position(|l| l == label) else { return };//never in it, or out already, which leaves tao's answer standing
	fullscreen.0.remove(at);
	let Some(app) = window_application() else { return };
	if fullscreen.0.is_empty() { app.setPresentationOptions(fullscreen.1) } else { app.setPresentationOptions(during) }
}

/// The window with this label is closing: off the list of windows in fuji's fullscreen, so one that goes without coming out never leaves the menu bar and dock hidden behind it
#[cfg(target_os = "macos")]
pub fn window_fullscreen_closed(label: &str) { if let Some(app) = window_application() { window_fullscreen_out(label, app.presentationOptions()) } }//nothing has changed them, so while another window is still in, they stay as they are
#[cfg(not(target_os = "macos"))]
pub fn window_fullscreen_closed(_label: &str) {}//elsewhere fullscreen keeps nothing for the application

//tao leaves fullscreen on windows by restoring the placement it saved on the way in, and that placement carries a show command, so a window hidden for a transition would flash up at its old frame; measured on the Windows box on 2026-09-28, 76 ms, pressing Esc on a fullscreen table. So a hidden window is cloaked, which the compositor draws as nothing, while tao restores it, and hidden again before the cloak comes off
#[cfg(target_os = "windows")]
fn window_fullscreen_leave_hidden(window: &WebviewWindow) {
	use windows::Win32::Foundation::HWND;
	use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_CLOAK};
	use windows::Win32::UI::WindowsAndMessaging::{IsWindowVisible, ShowWindow, SW_HIDE};

	let Ok(handle) = window.hwnd() else { return };
	let h = HWND(handle.0);
	if unsafe { IsWindowVisible(h) }.as_bool() { let _ = window.set_simple_fullscreen(false); return }//a showing window is meant to be seen leaving
	let cloak = |on: i32| unsafe { let _ = DwmSetWindowAttribute(h, DWMWA_CLOAK, &on as *const i32 as *const _, std::mem::size_of::<i32>() as u32); };//a BOOL, which is four bytes
	cloak(1);
	let _ = window.set_simple_fullscreen(false);
	let _ = unsafe { ShowWindow(h, SW_HIDE) };//windows' own hide, since tao's would do nothing: tao still counts the window hidden
	cloak(0);
}

/*
Where a window is, as the person looking at the screen would say.

Tauri's outer position and size are the platform's own window rectangle, and on the mac that is what anyone would mean: a frame there leaves the shadow out. On Windows 10 and 11 it does not. The rectangle includes the resize borders, the thick frame a window is grabbed by, which Windows 10 kept and made transparent: about 7 backing pixels on the left, the right and the bottom at 100 percent. So the numbers say a window is 7 backing pixels wider on each side and taller at the bottom than anything drawn on the screen, and a window placed flush with the edge of the work area by them stops 7 backing pixels short of it. The desktop window manager knows where the visible frame is, through DwmGetWindowAttribute with DWMWA_EXTENDED_FRAME_BOUNDS, and window_seen is the one place that answer belongs. Every rectangle and metric Windows hands back here is in backing pixels, the display resolution, because fuji declares itself DPI aware; window_frame and window_frame_set are where they become css pixels, dividing or multiplying by backing per css.

That answer is right only once a window has been shown. For one that never has, the manager returns the outer rectangle unchanged, and every window fuji places is one of those, because it places them hidden. So while a window is hidden window_unseen works the border out instead: the resize frame and its padding at the window's own DPI, less the one pixel of it that is drawn, on the left, the right and the bottom and never the top. The top border is there too, but painted as the upper band of the title bar rather than left transparent, which is why a maximized window, with its frame pushed off the screen, loses exactly that much of its title bar: 8 of 31 backing pixels at 100 percent. A maximized window hangs the whole frame off every side of the screen, top included, and a fullscreen one has no resize border at all.

All of that was measured on the Windows 10 box on 2026-09-28, at 1920 by 1200. The border came to 7, 8, 10 and 11 backing pixels at 100, 125, 150 and 175 percent, the formula and the manager agreeing at every one; maximized hung 8 off every side at 100 percent and 11 at 150, and fullscreen none. A sheet placed one backing pixel in from each corner of the work area showed exactly one pixel of desktop and then the one-pixel border at every scale, with the taskbar at the bottom and on the left, and read back the css frame it had asked for, fractions and all. A window that has been shown keeps the manager's right answer after it is hidden again, so only the never-shown case needs the arithmetic, but a hidden window cannot say which it is, and the two agree.

The page reads and places a window through window_frame and window_frame_set, in css pixels, and so never learns that a platform counts borders it does not draw.
*/

/// Where the window is and how big, in css pixels, as the user sees its frame
#[derive(Serialize, Deserialize)]
pub struct Frame { x: f64, y: f64, width: f64, height: f64 }

/// The window's visible frame, in css pixels
#[command]
pub fn window_frame(window: WebviewWindow) -> tauri::Result<Frame> {//tauri turns its own error into the rejection the page sees, so none of these need converting
	let backing_per_css = window.scale_factor()?;//what tauri calls the scale factor, on this window's screen
	let (at, size) = window_seen(&window)?;
	Ok(Frame { x: at.x as f64 / backing_per_css, y: at.y as f64 / backing_per_css, width: size.width as f64 / backing_per_css, height: size.height as f64 / backing_per_css })
}

/// Put the window's visible frame exactly here, in css pixels
#[command]
pub fn window_frame_set(window: WebviewWindow, frame: Frame) -> tauri::Result<()> {
	let backing_per_css = window.scale_factor()?;
	let (_, seen) = window_seen(&window)?;
	let inner = window.inner_size()?;
	let chrome = (seen.width.saturating_sub(inner.width), seen.height.saturating_sub(inner.height));//what the frame has around the content: a title bar and borders, or nothing on a window without decorations. set_size means the content, so this comes off the frame asked for
	let width  = ((frame.width  * backing_per_css).round() as u32).saturating_sub(chrome.0);
	let height = ((frame.height * backing_per_css).round() as u32).saturating_sub(chrome.1);
	window.set_size(PhysicalSize::new(width, height))?;//size first and then position, because a resize on the mac keeps the bottom left corner, which is where AppKit measures from, and so moves the top; tao queues both onto the main thread, in this order
	window_seen_move(&window, PhysicalPosition::new((frame.x * backing_per_css).round() as i32, (frame.y * backing_per_css).round() as i32))
}

//the visible frame, in backing pixels, which tauri calls physical: the outer rectangle less whatever the platform counts in it and does not draw
fn window_seen(window: &WebviewWindow) -> tauri::Result<(PhysicalPosition<i32>, PhysicalSize<u32>)> {
	let (at, size) = (window.outer_position()?, window.outer_size()?);
	let (left, top, right, bottom) = window_unseen(window);
	Ok((PhysicalPosition::new(at.x + left, at.y + top), PhysicalSize::new((size.width as i32 - left - right).max(0) as u32, (size.height as i32 - top - bottom).max(0) as u32)))
}

//what the outer rectangle holds beyond the visible frame, left, top, right and bottom: nothing, off windows
#[cfg(not(target_os = "windows"))]
fn window_unseen(_window: &WebviewWindow) -> (i32, i32, i32, i32) { (0, 0, 0, 0) }

//the invisible resize border, left, top, right and bottom, in backing pixels; the essay above has the rule and the measurements
#[cfg(target_os = "windows")]
fn window_unseen(window: &WebviewWindow) -> (i32, i32, i32, i32) {
	use windows::Win32::Foundation::{HWND, RECT};
	use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
	use windows::Win32::UI::HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi};
	use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongW, GetWindowRect, IsIconic, IsWindowVisible, IsZoomed, GWL_STYLE, SM_CXBORDER, SM_CXPADDEDBORDER, SM_CXSIZEFRAME, WS_THICKFRAME};

	let Ok(handle) = window.hwnd() else { return (0, 0, 0, 0) };
	let h = HWND(handle.0);//tauri's windows crate is an older version than fuji's, so the handle crosses as the pointer inside it
	unsafe {
		if IsWindowVisible(h).as_bool() && !IsIconic(h).as_bool() {//shown, so the manager knows, and knows for maximized and snapped windows as well as ordinary ones
			let (mut outer, mut seen) = (RECT::default(), RECT::default());
			if GetWindowRect(h, &mut outer).is_ok() && DwmGetWindowAttribute(h, DWMWA_EXTENDED_FRAME_BOUNDS, &mut seen as *mut RECT as *mut _, std::mem::size_of::<RECT>() as u32).is_ok() {
				return (seen.left - outer.left, seen.top - outer.top, outer.right - seen.right, outer.bottom - seen.bottom)
			}
		}
		if GetWindowLongW(h, GWL_STYLE) as u32 & WS_THICKFRAME.0 == 0 { return (0, 0, 0, 0) }//no resize border at all, which is fullscreen
		let dpi = GetDpiForWindow(h);//the window's own, 96 at 100 percent, which the metrics below are asked at: a system metric is otherwise the primary display's, and Windows rounds each metric at each scale on its own, so the border at 100 percent scaled up is wrong everywhere above it, 7 times 1.5 saying 10.5 where the border is 10
		let frame = GetSystemMetricsForDpi(SM_CXSIZEFRAME, dpi) + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
		if IsZoomed(h).as_bool() { return (frame, frame, frame, frame) }//maximized, with the whole frame off every side of the screen
		let unseen = frame - GetSystemMetricsForDpi(SM_CXBORDER, dpi);//less the one pixel of it that is drawn
		(unseen, 0, unseen, unseen)//none on top, where the same border is the title bar's upper band, drawn rather than transparent
	}
}

//move the window so its visible corner lands here. set_position places the outer corner, so this steps back by whatever lies between the outer corner and the visible one, which on windows is the invisible border's left and top. On windows tao asks for the move without waiting for it, and the page reveals the window straight after; on the Windows box the move always landed first, but within a millisecond of the reveal, which is a margin rather than a guarantee
fn window_seen_move(window: &WebviewWindow, to: PhysicalPosition<i32>) -> tauri::Result<()> {
	let outer = window.outer_position()?;
	let (seen, _) = window_seen(window)?;
	window.set_position(PhysicalPosition::new(to.x - (seen.x - outer.x), to.y - (seen.y - outer.y)))
}

/*
A native rectangle for a video, on Windows.

mpv draws video on Windows into a window it is handed, through its wid option: it makes a child of its own inside that window, fills it, and presents through Direct3D 11. The page cannot make a native window, so one command here makes a child of the Tauri window at a rectangle the page measured, in backing pixels of the client area, which is what a child's coordinates are, and answers its handle as the decimal string the option takes. It is a plain command rather than an async one so the child is made on the thread that owns the window and WebView2's own child, and it is raised above WebView2 among the siblings, since Windows puts a new child at the bottom of them, under the web view, seen in a dump of the window tree on the Windows box on 2026-10-13. Its class paints black under anything mpv has not drawn yet, and takes the mouse over it as its own, to hand up to the page.

**The mouse over the video reaches the page as an event, and mpv arranged half of it.** mpv's input-cursor-passthrough option, which hands pointer events through its window on a desktop, does nothing to a window with a parent, read in w32_common.c of 0.41.0; what mpv does instead when embedded is disable its own window, EnableWindow(w32->window, 0) in the same file, and a disabled child gets no mouse at all. Windows delivers the mouse over a disabled child to the nearest enabled ancestor, which is our window, as plain button and move messages; so the window procedure below sends each one up to the page as the event video-mouse, down, up or move with the point in the video's rectangle, and the page tells a click from a drag the way it does for a picture. The drag works because the button is really down when the page asks the window to follow the pointer. Nothing here finds or touches mpv's window: a hook on its thread and the layered styles were both tried on the Windows box on 2026-10-13 and both are unnecessary, since no mouse message ever reaches that thread, and the layered styles cannot be set on another thread's window anyway, SetWindowLongPtrW answering zero with no error. Answering the hit test transparent on this window was the third road tried that day, and it hands the hit to the web view's host window beside this one, whose default procedure drops it rather than passing it down to the engine's children. Only this module knows a video has a window of its own.

**A file dropped on the video reaches the page as an event too, through a drop target of this window's own.** A drop is OLE's: the program dragging asks the window under the pointer, and then each window above it, for the drop target registered on it, and shows the refusing cursor when none is found. The web view's are on its own windows, siblings of this one, which wry registered when the web view was made by walking the children the window had then; this window did not exist yet, so a drop over the video found nothing, seen on the Windows box on 2026-10-13. The older road, DragAcceptFiles and WM_DROPFILES, was tried on 2026-10-14 and the pointer still refused, so this window registers a drop target of its own, the small COM object below, the same way wry does for the web view's windows: it answers copy to a drag that carries files, and on the drop reads every path and sends them up as the event video-drop, in the shape the web view's own drop event has, and the shell hands it to the same function, so the panel never learns there was a second road. tao initialized OLE on the thread that owns the window, which registering requires, and the command that makes the window is plain so that it runs there.

On the Mac and on Linux the two commands make and remove a view of fuji's own, the two sections below.
*/
#[cfg(target_os = "windows")]
static VIDEO_WINDOWS: Mutex<Vec<(String, isize)>> = Mutex::new(Vec::new());//the video window of each window that has one, by label
#[cfg(target_os = "windows")]
static VIDEO_APP: OnceLock<AppHandle> = OnceLock::new();//the application, kept for the window procedure, which Windows calls with nothing of ours in hand
#[cfg(target_os = "windows")]
fn video_windows() -> std::sync::MutexGuard<'static, Vec<(String, isize)>> { VIDEO_WINDOWS.lock().unwrap_or_else(|p| p.into_inner()) }
#[cfg(target_os = "windows")]
fn video_forget(label: &str) -> Option<isize> {//out of the list, answering the handle for the caller to destroy or leave to its parent
	let mut list = video_windows();
	let at = list.iter().position(|(l, _)| l == label)?;
	Some(list.remove(at).1)
}
#[cfg(target_os = "windows")]
#[derive(Clone, Serialize)]
struct VideoMouse { kind: &'static str, x: i32, y: i32 }//one mouse message over the video, for the page: down, up or move, and where in the video's rectangle, in backing pixels
#[cfg(target_os = "windows")]
fn video_mouse(hwnd: windows::Win32::Foundation::HWND, kind: &'static str, x: i32, y: i32) {//to the page of the window this video window belongs to, as the event video-mouse
	use tauri::Emitter;
	let handle = hwnd.0 as isize;
	let Some(label) = video_windows().iter().find(|(_, h)| *h == handle).map(|(l, _)| l.clone()) else { return };
	if let Some(app) = VIDEO_APP.get() { let _ = app.emit_to(&label, "video-mouse", VideoMouse { kind, x, y }); }
}
#[cfg(target_os = "windows")]
#[derive(Clone, Serialize)]
struct VideoDrop { paths: Vec<String> }//the files dropped on the video, for the page, as the web view's own drop event lists them: every one, as Windows wrote the path, for the shell to forwardize at the boundary
#[cfg(target_os = "windows")]
#[windows::core::implement(windows::Win32::System::Ole::IDropTarget)]
struct VideoTarget { handle: isize, files: std::cell::Cell<bool> }//the drop target registered on one video window: its handle, to find the window's label, and whether the drag that entered carries files, answered once at the edge and repeated for every move
#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
impl windows::Win32::System::Ole::IDropTarget_Impl for VideoTarget_Impl {//OLE calls these on the window's thread as a drag crosses the window; the effect written back is the cursor the user sees
	fn DragEnter(&self, data: windows::core::Ref<windows::Win32::System::Com::IDataObject>, _keys: windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS, _at: &windows::Win32::Foundation::POINTL, effect: *mut windows::Win32::System::Ole::DROPEFFECT) -> windows::core::Result<()> {
		self.files.set(data.as_ref().is_some_and(|d| unsafe { d.QueryGetData(&video_files()) }.is_ok()));
		unsafe { *effect = if self.files.get() { windows::Win32::System::Ole::DROPEFFECT_COPY } else { windows::Win32::System::Ole::DROPEFFECT_NONE } };
		Ok(())
	}
	fn DragOver(&self, _keys: windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS, _at: &windows::Win32::Foundation::POINTL, effect: *mut windows::Win32::System::Ole::DROPEFFECT) -> windows::core::Result<()> {
		unsafe { *effect = if self.files.get() { windows::Win32::System::Ole::DROPEFFECT_COPY } else { windows::Win32::System::Ole::DROPEFFECT_NONE } };
		Ok(())
	}
	fn DragLeave(&self) -> windows::core::Result<()> { self.files.set(false); Ok(()) }
	fn Drop(&self, data: windows::core::Ref<windows::Win32::System::Com::IDataObject>, _keys: windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS, _at: &windows::Win32::Foundation::POINTL, effect: *mut windows::Win32::System::Ole::DROPEFFECT) -> windows::core::Result<()> {
		use tauri::Emitter;
		use windows::Win32::System::Ole::{ReleaseStgMedium, DROPEFFECT_COPY, DROPEFFECT_NONE};
		use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};
		self.files.set(false);
		unsafe { *effect = DROPEFFECT_NONE };
		let Some(data) = data.as_ref() else { return Ok(()) };
		let mut medium = unsafe { data.GetData(&video_files()) }?;//the list of files as the shell hands it, an HDROP in global memory
		let drop = HDROP(unsafe { medium.u.hGlobal }.0);
		let count = unsafe { DragQueryFileW(drop, u32::MAX, None) };//every index asks how many
		let paths = (0..count).map(|i| {
			let length = unsafe { DragQueryFileW(drop, i, None) } as usize;//without a buffer, the length in characters, not counting the null
			let mut text = vec![0u16; length + 1];
			let wrote = unsafe { DragQueryFileW(drop, i, Some(&mut text)) } as usize;
			String::from_utf16_lossy(&text[..wrote.min(length)])
		}).collect();
		unsafe { ReleaseStgMedium(&mut medium) };//the shell's memory for the list, given back
		unsafe { *effect = DROPEFFECT_COPY };
		let Some(label) = video_windows().iter().find(|(_, h)| *h == self.handle).map(|(l, _)| l.clone()) else { return Ok(()) };
		if let Some(app) = VIDEO_APP.get() { let _ = app.emit_to(&label, "video-drop", VideoDrop { paths }); }
		Ok(())
	}
}
#[cfg(target_os = "windows")]
fn video_files() -> windows::Win32::System::Com::FORMATETC {//the one format a drop is asked for: the shell's list of files, in global memory
	use windows::Win32::System::Com::{DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL};
	FORMATETC { cfFormat: windows::Win32::System::Ole::CF_HDROP.0, ptd: std::ptr::null_mut(), dwAspect: DVASPECT_CONTENT.0, lindex: -1, tymed: TYMED_HGLOBAL.0 as u32 }
}

/// A native child window of this window at this rectangle of its client area, in backing pixels, for a video to draw in, answering its handle as a decimal string for mpv's wid option; a second call replaces the first. Null off Windows, where a video is drawn another way
#[cfg(target_os = "windows")]
#[command]
pub fn window_video_make(window: WebviewWindow, x: i32, y: i32, width: i32, height: i32) -> Result<Option<String>, String> {
	use windows::core::{w, PCWSTR};
	use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
	use windows::Win32::Graphics::Gdi::{GetStockObject, BLACK_BRUSH, HBRUSH};
	use windows::Win32::System::LibraryLoader::GetModuleHandleW;
	use windows::Win32::System::Ole::{IDropTarget, RegisterDragDrop};
	use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, LoadCursorW, RegisterClassW, SetWindowPos, CS_DBLCLKS, HWND_TOP, IDC_ARROW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_NOPARENTNOTIFY, WS_VISIBLE};

	unsafe extern "system" fn video_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {//the class's window procedure: the left button and the move go up to the page as events, and Windows' own does the rest, the hit test included, so a hit here is this window's
		let kind = match msg { WM_LBUTTONDOWN => "down", WM_LBUTTONUP => "up", WM_MOUSEMOVE => "move", WM_LBUTTONDBLCLK => "double", _ => return unsafe { DefWindowProcW(hwnd, msg, w, l) } };//the second press of a double click comes as double in place of down, because the class asks for that, and its release as up like any other
		video_mouse(hwnd, kind, (l.0 & 0xffff) as u16 as i16 as i32, ((l.0 >> 16) & 0xffff) as u16 as i16 as i32);//the point in this window's own client area, in backing pixels, signed, since a drag can leave it
		LRESULT(0)
	}
	static CLASS: OnceLock<Result<(), String>> = OnceLock::new();//registered once per process, the first time a rectangle is asked for
	let class = w!("VideoSurface");
	let _ = VIDEO_APP.set(window.app_handle().clone());//for the window procedure, which has no other way to reach the page
	CLASS.get_or_init(|| unsafe {
		let instance = GetModuleHandleW(PCWSTR::null()).map_err(|e| e.to_string())?;
		let spec = WNDCLASSW { style: CS_DBLCLKS, lpfnWndProc: Some(video_proc), hInstance: HINSTANCE(instance.0), hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0), hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(), lpszClassName: class, ..Default::default() };//double clicks told from two clicks by Windows, black until mpv draws, and the arrow, since the mouse over the video is this window's to answer for
		if RegisterClassW(&spec) == 0 { return Err(windows::core::Error::from_thread().to_string()) }
		Ok(())
	}).clone()?;
	window_video_drop(window.clone())?;//the one before it, if any
	let parent = HWND(window.hwnd().map_err(|e| e.to_string())?.0);
	let child = unsafe { CreateWindowExW(WS_EX_NOPARENTNOTIFY, class, PCWSTR::null(), WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN, x, y, width, height, Some(parent), None, Some(window_module()), None) }.map_err(|e| e.to_string())?;//clipping siblings and children so nothing of fuji's paints over mpv's window inside it
	unsafe { SetWindowPos(child, Some(HWND_TOP), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) }.map_err(|e| e.to_string())?;//raised above the web view, where Windows did not put it
	let target: IDropTarget = VideoTarget { handle: child.0 as isize, files: std::cell::Cell::new(false) }.into();
	unsafe { RegisterDragDrop(child, &target) }.map_err(|e| e.to_string())?;//OLE keeps its own reference to the target until the window is revoked, so this one can go
	video_windows().push((window.label().to_string(), child.0 as isize));
	Ok(Some((child.0 as isize).to_string()))
}
#[cfg(target_os = "windows")]
fn window_module() -> windows::Win32::Foundation::HINSTANCE { unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(windows::core::PCWSTR::null()) }.map(|m| windows::Win32::Foundation::HINSTANCE(m.0)).unwrap_or_default() }//this executable, which the class was registered under

/// Remove the video window made for this window, if it has one; nothing, otherwise
#[cfg(target_os = "windows")]
#[command]
pub fn window_video_drop(window: WebviewWindow) -> Result<(), String> {
	use windows::Win32::Foundation::HWND;
	use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;
	let Some(handle) = video_forget(window.label()) else { return Ok(()) };
	let _ = unsafe { windows::Win32::System::Ole::RevokeDragDrop(HWND(handle as *mut _)) };//the drop target first, which frees it, and a window with none registered is nothing to report
	unsafe { DestroyWindow(HWND(handle as *mut _)) }.map_err(|e| e.to_string())
}

/// Move this window's video window to this rectangle of its client area, in backing pixels, and show it, or hide it where the rectangle has no area; mpv's own window inside follows its size, through the hook mpv puts on the window it is handed, read in w32_common.c of 0.41.0. Nothing, for a window with no video window
#[cfg(target_os = "windows")]
#[command]
pub fn window_video_place(window: WebviewWindow, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
	use windows::Win32::Foundation::HWND;
	use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, ShowWindow, HWND_TOP, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE};
	let Some(handle) = video_windows().iter().find(|(l, _)| l == window.label()).map(|(_, handle)| *handle) else { return Ok(()) };
	let child = HWND(handle as *mut _);
	if width <= 0 || height <= 0 { let _ = unsafe { ShowWindow(child, SW_HIDE) }; return Ok(()) }//its answer, whether the window was visible before, is nothing this needs
	unsafe { SetWindowPos(child, Some(HWND_TOP), x, y, width, height, SWP_NOACTIVATE | SWP_SHOWWINDOW) }.map_err(|e| e.to_string())//above the web view still, and shown
}

/// The window with this label is closing: forget its video window, which goes with its parent
#[cfg(target_os = "windows")]
pub fn window_video_closed(label: &str) { video_forget(label); }

/*
A native rectangle for a video, on the Mac.

mpv takes a window to draw in on X11, Windows and Android and nowhere else, so on the Mac the video is drawn by the library's render API into a surface of fuji's own: a view of ours above the WKWebView at the rectangle the page measured, whose layer is a CAOpenGLLayer, the arrangement mpv's own Mac window is built on, read in video/out/mac/gl_layer.swift of 0.41.0. Core Animation owns the surface and asks the layer to draw into it, on the main thread with the layer's GL context current; the draw hands media.rs the framebuffer and the size it finds bound, media.rs renders the player's current frame into them, and the superclass flushes the context, as its header asks. OpenGL is deprecated on the Mac and still there, and it is the one hardware backend the render API has.

**The view ignores the mouse, so the page sees every click and drag directly.** hitTest answers nil, and that is the whole of it: AppKit skips the view when it looks for what is under the pointer and finds the web view beneath, so the panel's own handlers on the square fire as they do over a picture, and a file dragged over the video lands on the web view's drop target as it does anywhere else on the page. Nothing here sends the page an event, which is the half of Windows' arrangement the Mac does not need.

**The GL context is made here, once, with the view, and lent to both of its users.** Core Animation draws with it current on the main thread, and media.rs makes and frees the render context with it current on the blocking pool, as the render API requires of every call; the lock CGL offers around the pool's turns is the protocol Apple documents for a context two threads share, and the layer hands its context and pixel format back to Core Animation retained, as copy methods must. A frame ready is said by mpv from a thread of its own, and the layer is displayed on the main thread, where a layer is drawn; the render never waits for the frame's time there, because media.rs starts the player with video-timing-offset at zero and says why.

The frame is the page's rectangle in backing pixels over the window's scale into points, laid out in the window's own coordinates from its content layout rectangle and converted into the superview's, so it lands right wherever the web view sits in its superview and whichever way that view is flipped; the mask keeps the top edge where it is when the window changes size, and window_video_place moves and resizes the view whenever the page asks, which is how the region follows the video's shape and the window's size. The pixel format asks for the 3.2 core profile and falls back to the legacy one, as mpv's own layer does.
*/
#[cfg(target_os = "macos")]
static VIDEO_VIEWS: Mutex<Vec<(String, isize, isize)>> = Mutex::new(Vec::new());//the video view of each window that has one, by label, and its layer, as addresses
#[cfg(target_os = "macos")]
static VIDEO_GL: Mutex<Vec<(isize, isize, isize)>> = Mutex::new(Vec::new());//each layer's pixel format and GL context, by the layer, held retained for as long as the view stands
#[cfg(target_os = "macos")]
static VIDEO_APP: OnceLock<AppHandle> = OnceLock::new();//the application, for the frame callback to reach the main thread
#[cfg(target_os = "macos")]
#[link(name = "OpenGL", kind = "framework")]
extern "C" {//CGL, the context layer under every OpenGL on the Mac, and the two GL reads the draw makes; declared by hand like the library's own functions, since fuji calls ten of them
	fn CGLChoosePixelFormat(attributes: *const u32, pix: *mut *mut c_void, count: *mut i32) -> i32;
	fn CGLRetainPixelFormat(pix: *mut c_void) -> *mut c_void;
	fn CGLReleasePixelFormat(pix: *mut c_void);
	fn CGLCreateContext(pix: *mut c_void, share: *mut c_void, ctx: *mut *mut c_void) -> i32;
	fn CGLRetainContext(ctx: *mut c_void) -> *mut c_void;
	fn CGLReleaseContext(ctx: *mut c_void);
	fn CGLSetCurrentContext(ctx: *mut c_void) -> i32;
	fn CGLLockContext(ctx: *mut c_void) -> i32;
	fn CGLUnlockContext(ctx: *mut c_void) -> i32;
	fn CGLErrorString(error: i32) -> *const std::ffi::c_char;
	fn glGetIntegerv(name: u32, out: *mut i32);
}
#[cfg(target_os = "macos")]
fn video_views() -> std::sync::MutexGuard<'static, Vec<(String, isize, isize)>> { VIDEO_VIEWS.lock().unwrap_or_else(|p| p.into_inner()) }
#[cfg(target_os = "macos")]
fn video_forget(label: &str) -> Option<(isize, isize)> {//out of the list, answering the view and its layer for the caller to remove or leave to the window
	let mut list = video_views();
	let at = list.iter().position(|(l, _, _)| l == label)?;
	let (_, view, layer) = list.remove(at);
	Some((view, layer))
}
#[cfg(target_os = "macos")]
fn video_gl_of(layer: isize) -> Option<(isize, isize)> { VIDEO_GL.lock().unwrap_or_else(|p| p.into_inner()).iter().find(|(l, _, _)| *l == layer).map(|(_, pix, gl)| (*pix, *gl)) }//a layer's pixel format and context
#[cfg(target_os = "macos")]
fn video_gl_forget(layer: isize) {//the view's own references to the layer's pixel format and context, given back; Core Animation holds its own until the layer goes, and media.rs its own until the render context does
	let mut list = VIDEO_GL.lock().unwrap_or_else(|p| p.into_inner());
	let Some(at) = list.iter().position(|(l, _, _)| *l == layer) else { return };
	let (_, pix, gl) = list.remove(at);
	unsafe { CGLReleaseContext(gl as *mut c_void); CGLReleasePixelFormat(pix as *mut c_void) }
}
#[cfg(target_os = "macos")]
fn cgl_error(error: i32) -> String { unsafe { std::ffi::CStr::from_ptr(CGLErrorString(error)) }.to_string_lossy().into_owned() }
#[cfg(target_os = "macos")]
fn video_pixel_format() -> Result<*mut c_void, String> {//a hardware pixel format for the layer: the 3.2 core profile, or the legacy one where that is refused, double buffered, offline renderers allowed and automatic graphics switching supported, the attributes mpv's own layer asks for by their values in CGLTypes.h
	let mut error = 0;
	for profile in [0x3200u32, 0x1000] {//kCGLOGLPVersion_3_2_Core, kCGLOGLPVersion_Legacy
		let attributes = [99, profile, 73, 5, 96, 101, 0];//kCGLPFAOpenGLProfile and its value, kCGLPFAAccelerated, kCGLPFADoubleBuffer, kCGLPFAAllowOfflineRenderers, kCGLPFASupportsAutomaticGraphicsSwitching, and the end
		let mut pix = std::ptr::null_mut();
		let mut count = 0;
		error = unsafe { CGLChoosePixelFormat(attributes.as_ptr(), &mut pix, &mut count) };
		if error == 0 && !pix.is_null() { return Ok(pix) }
	}
	Err(format!("CGLChoosePixelFormat: {}", cgl_error(error)))
}
#[cfg(target_os = "macos")]
#[allow(deprecated)]//OpenGL is deprecated on the Mac and still there, as the essay says, and objc2 marks the layer class with Apple's warning; here and on each function below that names it
fn video_classes() {//the two classes, registered once per process: a view that ignores the mouse, and a layer that draws through media.rs; made with the runtime's own builder rather than objc2's macro, the way dock.rs makes its delegate
	use objc2::runtime::ClassBuilder;
	use objc2::{sel, ClassType};
	use objc2_app_kit::NSView;
	use objc2_quartz_core::CAOpenGLLayer;
	static DONE: OnceLock<()> = OnceLock::new();
	DONE.get_or_init(|| {
		if let Some(mut view) = ClassBuilder::new(c"VideoSurface", NSView::class()) {
			unsafe { view.add_method(sel!(hitTest:), video_hit_test as extern "C" fn(_, _, _) -> _) }
			view.register();
		}
		if let Some(mut layer) = ClassBuilder::new(c"VideoLayer", CAOpenGLLayer::class()) {
			unsafe {
				layer.add_method(sel!(copyCGLPixelFormatForDisplayMask:), video_pixel_format_copy as extern "C" fn(_, _, _) -> _);
				layer.add_method(sel!(copyCGLContextForPixelFormat:), video_context_copy as extern "C" fn(_, _, _) -> _);
				layer.add_method(sel!(drawInCGLContext:pixelFormat:forLayerTime:displayTime:), video_draw as extern "C" fn(_, _, _, _, _, _));
			}
			layer.register();
		}
	});
}
#[cfg(target_os = "macos")]
extern "C" fn video_hit_test(_this: &objc2::runtime::AnyObject, _command: objc2::runtime::Sel, _point: objc2_foundation::NSPoint) -> *mut objc2_app_kit::NSView { std::ptr::null_mut() }//nothing here is under the pointer, so AppKit looks beneath and finds the web view
#[cfg(target_os = "macos")]
#[allow(deprecated)]
extern "C" fn video_pixel_format_copy(this: &objc2::runtime::AnyObject, _command: objc2::runtime::Sel, mask: u32) -> *mut c_void {//Core Animation asks once, and takes the answer retained
	use objc2::{msg_send, ClassType};
	match video_gl_of(this as *const _ as isize) { Some((pix, _)) => unsafe { CGLRetainPixelFormat(pix as *mut c_void) }, None => unsafe { msg_send![super(this, objc2_quartz_core::CAOpenGLLayer::class()), copyCGLPixelFormatForDisplayMask: mask] } }
}
#[cfg(target_os = "macos")]
#[allow(deprecated)]
extern "C" fn video_context_copy(this: &objc2::runtime::AnyObject, _command: objc2::runtime::Sel, pix: *mut c_void) -> *mut c_void {//and the context the same way, the one made with the view, so the render context media.rs makes against it draws here
	use objc2::{msg_send, ClassType};
	match video_gl_of(this as *const _ as isize) { Some((_, gl)) => unsafe { CGLRetainContext(gl as *mut c_void) }, None => unsafe { msg_send![super(this, objc2_quartz_core::CAOpenGLLayer::class()), copyCGLContextForPixelFormat: pix] } }
}
#[cfg(target_os = "macos")]
#[allow(deprecated)]
extern "C" fn video_draw(this: &objc2::runtime::AnyObject, _command: objc2::runtime::Sel, ctx: *mut c_void, pix: *mut c_void, time: f64, stamp: *const c_void) {//on the main thread, with the context current and a framebuffer bound: the player's current frame into it, and then the superclass's flush
	use objc2::{msg_send, ClassType};
	let mut framebuffer = 0;
	let mut viewport = [0; 4];
	unsafe { glGetIntegerv(0x8CA6, &mut framebuffer); glGetIntegerv(0x0BA2, viewport.as_mut_ptr()) }//GL_FRAMEBUFFER_BINDING, which Core Animation may leave at zero for the layer's own, and GL_VIEWPORT, which it sets to the surface in pixels
	crate::media::video_render(this as *const _ as isize, framebuffer, viewport[2], viewport[3]);
	unsafe { let _: () = msg_send![super(this, objc2_quartz_core::CAOpenGLLayer::class()), drawInCGLContext: ctx, pixelFormat: pix, forLayerTime: time, displayTime: stamp]; }
}

/// The layer and GL context of this window's video view, for media.rs to make a render context against; none where the window has no view
#[cfg(target_os = "macos")]
pub fn video_gl(label: &str) -> Option<(isize, isize)> {
	let layer = video_views().iter().find(|(l, _, _)| l == label).map(|(_, _, layer)| *layer)?;
	video_gl_of(layer).map(|(_, gl)| (layer, gl))
}
/// One turn with this GL context current on the calling thread, under CGL's lock, for a render API call off the main thread; the bounds are Linux's, where the turn is taken on another thread, so media.rs writes one call for both
#[cfg(target_os = "macos")]
pub fn video_gl_current<R: Send + 'static>(gl: isize, work: impl FnOnce() -> R + Send + 'static) -> R {
	let gl = gl as *mut c_void;
	unsafe { CGLLockContext(gl); CGLSetCurrentContext(gl) };
	let answer = work();
	unsafe { CGLSetCurrentContext(std::ptr::null_mut()); CGLUnlockContext(gl) };
	answer
}
/// A reference of the caller's own to this GL context, and the same reference given back, so a render context outlives the view it was made for if the window goes first
#[cfg(target_os = "macos")]
pub fn video_gl_retain(gl: isize) { unsafe { CGLRetainContext(gl as *mut c_void) }; }
#[cfg(target_os = "macos")]
pub fn video_gl_release(gl: isize) { unsafe { CGLReleaseContext(gl as *mut c_void) } }
/// A frame is ready for this layer: display it on the main thread, where a layer is drawn, and commit at once rather than at the end of the run loop's turn; nothing, for a layer that is gone
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn video_display(layer: isize) {
	let Some(app) = VIDEO_APP.get() else { return };
	let _ = app.run_on_main_thread(move || {
		if !video_views().iter().any(|(_, _, l)| *l == layer) { return }//removed between the callback and this turn, so the pointer is not to be followed
		let layer: &objc2_quartz_core::CAOpenGLLayer = unsafe { &*(layer as *const objc2_quartz_core::CAOpenGLLayer) };
		layer.display();
		objc2_quartz_core::CATransaction::flush();
	});
}

/// A view of fuji's own above the web view at this rectangle of the window, in backing pixels, with a GL layer for a video to draw in, answering the layer's address as text, which media.rs takes as the wid and resolves through the window's label; a second call replaces the first
#[cfg(target_os = "macos")]
#[allow(deprecated)]
#[command]
pub fn window_video_make(window: WebviewWindow, x: i32, y: i32, width: i32, height: i32) -> Result<Option<String>, String> {
	use objc2::rc::Retained;
	use objc2::runtime::AnyClass;
	use objc2::{msg_send, MainThreadMarker};
	use objc2_app_kit::{NSAutoresizingMaskOptions, NSView, NSWindowOrderingMode};
	use objc2_quartz_core::{CALayer, CAOpenGLLayer};
	if MainThreadMarker::new().is_none() { return Err("the video view is made on the main thread, and this was not it".into()) }
	let _ = VIDEO_APP.set(window.app_handle().clone());
	video_classes();
	window_video_drop(window.clone())?;//the one before it, if any
	let web = video_webview(&window)?;
	let webview: &NSView = unsafe { &*(web as *const NSView) };
	let Some(parent) = (unsafe { webview.superview() }) else { return Err("the web view has no superview".into()) };
	let Some(ns_window) = webview.window() else { return Err("the web view is in no window".into()) };
	let scale = ns_window.backingScaleFactor();
	let flipped = parent.isFlipped();
	let frame = video_frame(webview, &parent, scale, x, y, width, height);
	let Some(layer_class) = AnyClass::get(c"VideoLayer") else { return Err("the video layer class did not register".into()) };//both classes before anything is made, so a refusal leaves nothing behind
	let Some(view_class) = AnyClass::get(c"VideoSurface") else { return Err("the video view class did not register".into()) };

	let pix = video_pixel_format()?;
	let mut gl = std::ptr::null_mut();
	let made = unsafe { CGLCreateContext(pix, std::ptr::null_mut(), &mut gl) };
	if made != 0 || gl.is_null() { unsafe { CGLReleasePixelFormat(pix) }; return Err(format!("CGLCreateContext: {}", cgl_error(made))) }

	let layer: Retained<CAOpenGLLayer> = unsafe { msg_send![layer_class, new] };
	let layer_at = &*layer as *const CAOpenGLLayer as isize;
	VIDEO_GL.lock().unwrap_or_else(|p| p.into_inner()).push((layer_at, pix as isize, gl as isize));//before anything asks the layer for them
	layer.setContentsScale(scale);//the surface in backing pixels rather than points
	layer.setAsynchronous(false);//drawn when displayed, which video_display does for each frame, rather than polled at the display's rate
	layer.setNeedsDisplayOnBoundsChange(true);
	layer.setOpaque(true);

	let view: Retained<NSView> = unsafe { msg_send![view_class, new] };
	view.setFrame(frame);
	let base: &CALayer = &layer;
	view.setLayer(Some(base));//the layer first and then wantsLayer, which makes the view layer-hosting: AppKit draws nothing of its own into it
	view.setWantsLayer(true);
	view.setAutoresizingMask(if flipped { NSAutoresizingMaskOptions::ViewMaxYMargin } else { NSAutoresizingMaskOptions::ViewMinYMargin });//the margin on the far side from the top flexes, so the top edge holds
	parent.addSubview_positioned_relativeTo(&view, NSWindowOrderingMode::Above, Some(webview));
	video_views().push((window.label().to_string(), &*view as *const NSView as isize, layer_at));
	Ok(Some(layer_at.to_string()))
}

/// Move this window's video view to this rectangle of the window, in backing pixels, and show it, or hide it where the rectangle has no area; the layer draws again at its new size, since it asks to when its bounds change. Nothing, for a window with no video view
#[cfg(target_os = "macos")]
#[command]
pub fn window_video_place(window: WebviewWindow, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
	use objc2::MainThreadMarker;
	use objc2_app_kit::NSView;
	if MainThreadMarker::new().is_none() { return Err("the video view is placed on the main thread, and this was not it".into()) }
	let Some(view) = video_views().iter().find(|(l, _, _)| l == window.label()).map(|(_, view, _)| *view) else { return Ok(()) };
	let view: &NSView = unsafe { &*(view as *const NSView) };
	if width <= 0 || height <= 0 { view.setHidden(true); return Ok(()) }
	let web = video_webview(&window)?;
	let webview: &NSView = unsafe { &*(web as *const NSView) };
	let Some(parent) = (unsafe { view.superview() }) else { return Err("the video view is in no superview".into()) };
	let Some(ns_window) = view.window() else { return Err("the video view is in no window".into()) };
	let frame = video_frame(webview, &parent, ns_window.backingScaleFactor(), x, y, width, height);
	view.setFrame(frame);
	view.setHidden(false);
	let said = |r: objc2_foundation::NSRect| format!("{},{} {}x{}", r.origin.x, r.origin.y, r.size.width, r.size.height);//points, from the bottom left unless flipped
	crate::log::log(&format!("window: video view placed at {} in its superview, flipped {}; the web view's frame {} and bounds {}, flipped {}; the superview's bounds {}; the window's content layout {} and frame {}", said(frame), parent.isFlipped(), said(webview.frame()), said(webview.bounds()), webview.isFlipped(), said(parent.bounds()), said(ns_window.contentLayoutRect()), said(ns_window.frame())));//where the view landed against the web view and the window, which a page measuring only itself cannot see
	Ok(())
}
#[cfg(target_os = "macos")]
fn video_webview(window: &WebviewWindow) -> Result<usize, String> {//the address of the window's web view, asked on the main thread, where with_webview answers at once
	let found = std::sync::Arc::new(Mutex::new(0usize));
	let seen = found.clone();
	window.with_webview(move |webview| { *seen.lock().unwrap_or_else(|p| p.into_inner()) = webview.inner() as usize }).map_err(|e| e.to_string())?;
	let web = *found.lock().unwrap_or_else(|p| p.into_inner());
	if web == 0 { return Err("the window has no web view to put the video over".into()) }
	Ok(web)
}
#[cfg(target_os = "macos")]
fn video_frame(webview: &objc2_app_kit::NSView, parent: &objc2_app_kit::NSView, scale: f64, x: i32, y: i32, width: i32, height: i32) -> objc2_foundation::NSRect {//a rectangle of the page, in backing pixels from the page's top left, as a frame in the superview's points. The page's top left is the top left of the window's content layout rectangle, the part no title bar covers: where the web view starts below a title bar the two are the same, and where it runs up under one, WebKit insets the page by the bar, which showed as a title bar's height of white under the video in a window and none in fuji's fullscreen, on the Mac mini on 2026-10-16. A title bar drawn transparent over a full-size page, as a video window's is, insets nothing, so there the page's top left is the window's own and this lands the view a title bar too low, read in WebKit's PageClientImplMac.mm; such a window needs the content view's top left instead, which nothing here does yet
	use objc2_foundation::{NSPoint, NSRect, NSSize};
	let (x, y, width, height) = (x as f64 / scale, y as f64 / scale, width as f64 / scale, height as f64 / scale);//into points, which a frame is in
	let Some(window) = webview.window() else { return NSRect::new(NSPoint::new(x, y), NSSize::new(width, height)) };
	let layout = window.contentLayoutRect();//in the window's own coordinates, from its bottom left
	let in_window = NSRect::new(NSPoint::new(layout.origin.x + x, layout.origin.y + layout.size.height - y - height), NSSize::new(width, height));
	parent.convertRect_fromView(in_window, None)//into the superview's, whichever way it is flipped
}

/// Remove the video view made for this window, if it has one; nothing, otherwise
#[cfg(target_os = "macos")]
#[command]
pub fn window_video_drop(window: WebviewWindow) -> Result<(), String> {
	use objc2::MainThreadMarker;
	use objc2_app_kit::NSView;
	let Some((view, layer)) = video_forget(window.label()) else { return Ok(()) };
	if MainThreadMarker::new().is_none() { return Err("the video view is removed on the main thread, and this was not it".into()) }
	let view: &NSView = unsafe { &*(view as *const NSView) };
	view.removeFromSuperview();//the superview held the view and the view its layer, so both go here, and Core Animation gives back the pixel format and context it copied
	video_gl_forget(layer);
	Ok(())
}

/// The window with this label is closing: forget its video view, which goes with its window
#[cfg(target_os = "macos")]
pub fn window_video_closed(label: &str) { if let Some((_, layer)) = video_forget(label) { video_gl_forget(layer) } }

/*
A native rectangle for a video, on Linux.

Every Linux desktop fuji ships to runs Wayland, where mpv cannot be handed a window, so the video is drawn as it is on the Mac, by the render API into a surface of fuji's own: a GTK GL area over the web view at the rectangle the page measured, which serves a desktop still on X11 the same way. The window wry builds is a GTK window holding a vertical box with the WebKit view packed in it, and the first rectangle asked for lifts the web view out of that box into an overlay put back in its place, GTK's own arrangement for one widget drawn over another; the area is an overlay child placed by its margins from the top left, which holds its top edge as the Mac's view does. GTK makes the area's GL context as the area is realized, binds the area's framebuffer and sets the viewport before asking for a frame, and composites what was drawn into the window with everything else; the render signal hands media.rs the framebuffer and the size it finds bound, as the Mac's draw does, and clears the area to black when no player draws it, since GTK leaves a fresh buffer undefined.

**The area is passed through, so the page sees every click and drag directly.** GTK's overlay can mark a child pass-through, the Mac's hitTest nil in GTK's words: the pointer over the area goes to the web view beneath, so the panel's own handlers see the mouse, and a file dragged over the video lands on the web view's drop target as it does anywhere else on the page. Nothing here sends the page an event.

**The GL context is GTK's, and current on the main thread alone.** GTK makes it current before each render and clears it after, so a render API call from the blocking pool, making the render context or freeing it, runs on the main thread with the area made current for it while the pool waits, which is video_gl_current here. The area's unrealize, which GTK raises as the widget leaves the window, frees any render context on it first, with the context still there to be current, so a window closing mid-play never leaves a render context for the player's destroy to free without one; lib.rs forgets the view before it destroys the player for that reason. The one turn this cannot cover is a player destroyed on the pool at the moment its window closes, where the turn on the main thread may come after the area has gone and the free runs with no context current, which GL answers by doing nothing.

The rectangle arrives in backing pixels and GTK lays widgets out in css pixels, so the margins and the size are divided by the window's scale, which GTK keeps whole. The GL functions media.rs asks for are found through the loader of whichever GL the display has, EGL's on Wayland and GLX's on X11, each answering for the context current on the calling thread, the way mpv's own outputs on the two find them. None of this has run on a Linux desktop yet: the containers prove that it compiles and links, and the first smoke test is the Linux desktop's.
*/
#[cfg(target_os = "linux")]
static VIDEO_VIEWS: Mutex<Vec<(String, isize)>> = Mutex::new(Vec::new());//the video area of each window that has one, by label, as its address: the wid the page hands back and the key media.rs draws by
#[cfg(target_os = "linux")]
static VIDEO_APP: OnceLock<AppHandle> = OnceLock::new();//the application, for the frame callback and the pool to reach the main thread
#[cfg(target_os = "linux")]
static VIDEO_LOADER: OnceLock<usize> = OnceLock::new();//the GL loader, found with the display the first time a rectangle is asked for
#[cfg(target_os = "linux")]
thread_local! { static VIDEO_AREAS: std::cell::RefCell<Vec<(isize, gtk::GLArea)>> = const { std::cell::RefCell::new(Vec::new()) }; }//the areas themselves, by address, on the main thread, which alone may touch a widget: a reference each, held while the view stands
#[cfg(target_os = "linux")]
fn video_views() -> std::sync::MutexGuard<'static, Vec<(String, isize)>> { VIDEO_VIEWS.lock().unwrap_or_else(|p| p.into_inner()) }
#[cfg(target_os = "linux")]
fn video_forget(label: &str) -> Option<isize> {//out of the list, answering the area's address for the caller to remove or leave to its window
	let mut list = video_views();
	let at = list.iter().position(|(l, _)| l == label)?;
	Some(list.remove(at).1)
}
#[cfg(target_os = "linux")]
fn video_area(at: isize) -> Option<gtk::GLArea> { VIDEO_AREAS.with(|list| list.borrow().iter().find(|(a, _)| *a == at).map(|(_, area)| area.clone())) }//the area at this address, on the main thread, while the view stands
#[cfg(target_os = "linux")]
fn video_area_forget(at: isize) { VIDEO_AREAS.with(|list| { let mut list = list.borrow_mut(); if let Some(i) = list.iter().position(|(a, _)| *a == at) { list.remove(i); } }) }
#[cfg(target_os = "linux")]
fn video_gl_loader() -> usize {//eglGetProcAddress or glXGetProcAddressARB, by the display GTK is on, and zero where the library will not load
	use gtk::prelude::*;
	let wayland = gdk::Display::default().map(|display| display.type_().name().starts_with("GdkWayland")).unwrap_or(false);
	let (library, symbol) = if wayland { (c"libEGL.so.1", c"eglGetProcAddress") } else { (c"libGL.so.1", c"glXGetProcAddressARB") };
	let handle = unsafe { libc::dlopen(library.as_ptr(), libc::RTLD_LAZY | libc::RTLD_GLOBAL) };
	if handle.is_null() { return 0 }
	(unsafe { libc::dlsym(handle, symbol.as_ptr()) }) as usize
}
/// A GL function by name through the display's own loader, for media.rs; null before a view has been made, or where the loader has no such function
#[cfg(target_os = "linux")]
pub fn video_gl_address(name: *const std::ffi::c_char) -> *mut std::ffi::c_void {
	let Some(loader) = VIDEO_LOADER.get().copied().filter(|l| *l != 0) else { return std::ptr::null_mut() };
	let loader: unsafe extern "C" fn(*const std::ffi::c_char) -> *mut std::ffi::c_void = unsafe { std::mem::transmute(loader) };
	unsafe { loader(name) }
}
#[cfg(target_os = "linux")]
fn video_gl_integers(name: u32, out: &mut [i32]) {//glGetIntegerv, for the framebuffer and the viewport GTK bound, through the loader like every GL function here
	static GET: OnceLock<usize> = OnceLock::new();
	let get = *GET.get_or_init(|| video_gl_address(c"glGetIntegerv".as_ptr()) as usize);
	if get == 0 { return }
	let get: unsafe extern "C" fn(u32, *mut i32) = unsafe { std::mem::transmute(get) };
	unsafe { get(name, out.as_mut_ptr()) }
}
#[cfg(target_os = "linux")]
fn video_gl_clear() {//the area black, for a draw with no player: glClearColor and then glClear of the color buffer
	static CLEAR: OnceLock<(usize, usize)> = OnceLock::new();
	let (color, clear) = *CLEAR.get_or_init(|| (video_gl_address(c"glClearColor".as_ptr()) as usize, video_gl_address(c"glClear".as_ptr()) as usize));
	if color == 0 || clear == 0 { return }
	let color: unsafe extern "C" fn(f32, f32, f32, f32) = unsafe { std::mem::transmute(color) };
	let clear: unsafe extern "C" fn(u32) = unsafe { std::mem::transmute(clear) };
	unsafe { color(0.0, 0.0, 0.0, 1.0); clear(0x4000) }//GL_COLOR_BUFFER_BIT
}
#[cfg(target_os = "linux")]
fn video_rendered(area: &gtk::GLArea) {//the render signal, on the main thread with the context current and the area's framebuffer bound: the player's current frame into it, or black where no player draws this area
	use gtk::prelude::*;
	let mut framebuffer = [0];
	let mut viewport = [0; 4];
	video_gl_integers(0x8CA6, &mut framebuffer); video_gl_integers(0x0BA2, &mut viewport);//GL_FRAMEBUFFER_BINDING and GL_VIEWPORT, which GTK set to the area in pixels
	if !crate::media::video_render(area.as_ptr() as isize, framebuffer[0], viewport[2], viewport[3]) { video_gl_clear() }
}
#[cfg(target_os = "linux")]
fn video_gl_turn<R>(gl: isize, work: impl FnOnce() -> R) -> R {//on the main thread: the area's context current for the work, where the area still stands, and cleared after
	use gtk::prelude::*;
	let area = video_area(gl).filter(|area| area.context().is_some());
	if let Some(area) = &area { area.make_current() }
	let answer = work();
	if area.is_some() { gdk::GLContext::clear_current() }
	answer
}

/// The surface and GL context of this window's video view, for media.rs to make a render context against, both the area's address here; none where the window has no view
#[cfg(target_os = "linux")]
pub fn video_gl(label: &str) -> Option<(isize, isize)> { video_views().iter().find(|(l, _)| l == label).map(|(_, at)| (*at, *at)) }
/// One turn with this GL context current, for a render API call: on the main thread, which alone may make GTK's context current, with the caller waiting if it is another thread; the work runs without a context where the view has already gone, or where the main thread is no longer taking turns
#[cfg(target_os = "linux")]
pub fn video_gl_current<R: Send + 'static>(gl: isize, work: impl FnOnce() -> R + Send + 'static) -> R {
	if gtk::is_initialized_main_thread() { return video_gl_turn(gl, work) }
	let work: std::sync::Arc<Mutex<Option<Box<dyn FnOnce() -> R + Send>>>> = std::sync::Arc::new(Mutex::new(Some(Box::new(work))));//held here as well as handed over, so work the main thread never ran is run here instead
	let (send, receive) = std::sync::mpsc::channel();
	if let Some(app) = VIDEO_APP.get() {
		let handed = work.clone();
		let hopped = app.run_on_main_thread(move || { if let Some(work) = handed.lock().unwrap_or_else(|p| p.into_inner()).take() { let _ = send.send(video_gl_turn(gl, work)); } });
		if hopped.is_ok() { if let Ok(answer) = receive.recv() { return answer } }
	}
	let left = work.lock().unwrap_or_else(|p| p.into_inner()).take();
	left.expect("work the main thread did not run is still here")()
}
/// A reference of the caller's own to this GL context, and the same given back: nothing on Linux, where the area's unrealize frees whatever draws it before the context goes, so no reference past the view's own is needed
#[cfg(target_os = "linux")]
pub fn video_gl_retain(_gl: isize) {}
#[cfg(target_os = "linux")]
pub fn video_gl_release(_gl: isize) {}
/// A frame is ready for this area: queue a render on the main thread, where GTK draws, for its next frame; nothing, for an area that is gone
#[cfg(target_os = "linux")]
pub fn video_display(layer: isize) {
	let Some(app) = VIDEO_APP.get() else { return };
	let _ = app.run_on_main_thread(move || { use gtk::prelude::*; if let Some(area) = video_area(layer) { area.queue_render() } });
}

/// A GL area of fuji's own over the web view at this rectangle of the window, in backing pixels, for a video to draw in, answering the area's address as text, which media.rs takes as the wid and resolves through the window's label; a second call replaces the first
#[cfg(target_os = "linux")]
#[command]
pub fn window_video_make(window: WebviewWindow, x: i32, y: i32, width: i32, height: i32) -> Result<Option<String>, String> {
	use gtk::prelude::*;
	if !gtk::is_initialized_main_thread() { return Err("the video view is made on the main thread, and this was not it".into()) }
	let _ = VIDEO_APP.set(window.app_handle().clone());
	VIDEO_LOADER.get_or_init(video_gl_loader);
	window_video_drop(window.clone())?;//the one before it, if any
	let found = std::sync::Arc::new(Mutex::new(0usize));
	let seen = found.clone();
	window.with_webview(move |webview| { *seen.lock().unwrap_or_else(|p| p.into_inner()) = webview.inner().as_ptr() as usize }).map_err(|e| e.to_string())?;//at once, since this is the main thread
	let web = *found.lock().unwrap_or_else(|p| p.into_inner());
	if web == 0 { return Err("the window has no web view to put the video over".into()) }
	let web: gtk::Widget = unsafe { glib::translate::from_glib_none(web as *mut gtk::ffi::GtkWidget) };//a reference of our own to the widget at that address
	let scale = web.scale_factor().max(1);
	let (x, y, width, height) = (x / scale, y / scale, width / scale, height / scale);//into css pixels, which GTK lays out in
	let overlay = match web.parent() {
		Some(parent) if parent.is::<gtk::Overlay>() => parent.downcast::<gtk::Overlay>().map_err(|_| "the web view's overlay is not one".to_string())?,//the overlay the first rectangle made
		Some(parent) => {//wry's box: the web view comes out of it into an overlay that goes in its place
			let container = parent.downcast::<gtk::Container>().map_err(|_| "the web view's parent is no container".to_string())?;
			let overlay = gtk::Overlay::new();
			container.remove(&web);
			overlay.add(&web);
			match container.downcast_ref::<gtk::Box>() { Some(packed) => packed.pack_start(&overlay, true, true, 0), None => container.add(&overlay) }
			overlay.show();
			overlay
		}
		None => return Err("the web view is in no container".into()),
	};
	let area = gtk::GLArea::new();
	area.set_has_alpha(false);
	area.set_has_depth_buffer(false);
	area.set_has_stencil_buffer(false);
	area.set_auto_render(false);//drawn when a frame is queued, which video_display does for each, rather than on every redraw of the window
	area.set_halign(gtk::Align::Start);
	area.set_valign(gtk::Align::Start);
	area.set_margin_start(x);
	area.set_margin_top(y);
	area.set_size_request(width, height);
	let at = area.as_ptr() as isize;
	area.connect_render(|area, _| { video_rendered(area); glib::Propagation::Stop });
	area.connect_unrealize(move |area| { area.make_current(); crate::media::video_unrealize(at); gdk::GLContext::clear_current() });//with the context still there, before GTK takes it
	overlay.add_overlay(&area);
	overlay.set_overlay_pass_through(&area, true);
	area.show();
	area.realize();//now rather than at the next layout, so the GL context exists for the render context the player's start makes against it
	if let Some(error) = area.error() { overlay.remove(&area); return Err(format!("the GL area could not make a context: {error}")) }
	if area.context().is_none() { overlay.remove(&area); return Err("the GL area has no context".into()) }
	VIDEO_AREAS.with(|list| list.borrow_mut().push((at, area)));
	video_views().push((window.label().to_string(), at));
	Ok(Some(at.to_string()))
}

/// Move this window's video area to this rectangle of the window, in backing pixels, and show it, or hide it where the rectangle has no area; GTK lays the area out again by its margins and size, in css pixels, and renders it at its new size. Nothing, for a window with no video area
#[cfg(target_os = "linux")]
#[command]
pub fn window_video_place(window: WebviewWindow, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
	use gtk::prelude::*;
	if !gtk::is_initialized_main_thread() { return Err("the video view is placed on the main thread, and this was not it".into()) }
	let Some(at) = video_views().iter().find(|(l, _)| l == window.label()).map(|(_, at)| *at) else { return Ok(()) };
	let Some(area) = video_area(at) else { return Ok(()) };
	if width <= 0 || height <= 0 { area.hide(); return Ok(()) }//unmapped rather than unrealized, so its GL context, and the render context made against it, stand
	let scale = area.scale_factor().max(1);//into css pixels, as window_video_make divides them
	area.set_margin_start(x / scale);
	area.set_margin_top(y / scale);
	area.set_size_request(width / scale, height / scale);
	area.show();
	Ok(())
}

/// Remove the video area made for this window, if it has one; nothing, otherwise
#[cfg(target_os = "linux")]
#[command]
pub fn window_video_drop(window: WebviewWindow) -> Result<(), String> {
	let Some(at) = video_forget(window.label()) else { return Ok(()) };
	if !gtk::is_initialized_main_thread() { return Err("the video view is removed on the main thread, and this was not it".into()) }
	video_area_remove(at);
	Ok(())
}
#[cfg(target_os = "linux")]
fn video_area_remove(at: isize) {//out of its overlay, which unrealizes it and so frees any render context on it first; the reference here goes with it
	use gtk::prelude::*;
	let Some(area) = video_area(at) else { return };
	video_area_forget(at);
	if let Some(parent) = area.parent().and_then(|parent| parent.downcast::<gtk::Container>().ok()) { parent.remove(&area) }
}

/// The window with this label is closing: forget its video area and take it out now, while its context stands, so what drew it is freed before the window and the player go
#[cfg(target_os = "linux")]
pub fn window_video_closed(label: &str) {
	let Some(at) = video_forget(label) else { return };
	if gtk::is_initialized_main_thread() { video_area_remove(at) }
}
