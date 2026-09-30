use std::fs::{File, OpenOptions, TryLockError};
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::open;

/*
Making fuji's windows, saying where one is, and deciding how long fuji outlives them. This is here rather than in tauri.conf.json because a window is no longer a thing there is exactly one of: each is made when it is asked for, under a label counted for it.

**How many, and who asks for one.** A window is made for the pictures it will show, and `window_build` is the only way one comes into being. Every launch on every platform makes one through `window_first`, when the application loop is ready, for whatever the command line handed over — unless a double-click on the mac has already made one, which `window_first` explains. On macOS a window is made again whenever the running application is told to open something, and again when the user clicks a dock icon with no windows behind it. Windows and Linux never make a second one, because their shells start another process instead — so the same function serves a platform with many windows and a platform with exactly one, and neither needs to know which it is.

**Why the platforms differ at all, since a process per window everywhere would be simpler.** Because the macOS Dock will not have it. Measured on the Mac mini, macOS 15.7.4, 2026-09-13: two running copies of one bundle appear to LaunchServices as two foreground applications — separate serial numbers under a single bundle identifier — and the Dock draws a tile per foreground application rather than per bundle. So several fujis would be a row of identical icons with nothing able to merge them. Windows and Linux hide the same architecture — the Windows taskbar combines buttons for one executable, and GNOME and KDE group by `WM_CLASS` — so macOS is the only shell of the three that needs code.

**What it is called.** Each window gets a label of its own, counted from one, and `capabilities/default.json` grants permissions to the pattern rather than to a name. A window whose label the capability does not match is built and then silently has no permissions at all, which looks like a page that loads and cannot do anything, so the two have to be kept in step.

**Where, and how big, is the page's to decide.** Every window is built hidden at whatever size Tauri defaults to, and the shell places it before the reveal: fitted around the picture for a preview, or as the contact sheet, at the size the user last gave one or a preset portion of the work area, somewhere at random inside it, by the rule library.js keeps. Rust reads none of it: the page has the settings file before it places anything. The window is hidden while this happens, so nothing flashes and nothing jumps.

Fuji places its windows itself because neither platform does it well. Windows cascades new ones down a fixed staircase without checking that they fit — measured on 2026-09-13, three windows 1062 tall on a work area 1160 deep, the last two hanging 6 and 40 under the taskbar. A work area 1160 deep on that 1200-row screen is what 100 percent leaves, so css and backing pixels were one and the numbers need no unit. macOS places nothing at all, and tao centers every window it is given no position for, so two windows of one size stack exactly on top of each other.

**How long fuji outlives its last window** is the one place fuji deliberately behaves differently on each platform, and `window_stays_resident` below is the whole of it. On Windows and Linux, closing the window closes fuji, which is what those desktops mean by closing a window. On macOS an application is a place the user is in rather than a window they have open: the dock icon stays, with its dot, which is how a Mac user reopens it, learns they can keep it there, and decides what to quit when the machine is busy. Going with that grain costs almost nothing here, because a fuji with no windows has destroyed its webviews and is the Rust host alone — and the Rust is dumb, so it is doing nothing.
*/

/*
Copies of fuji that start together.

Windows and Linux start a copy of fuji for each picture opened, and Explorer starts one for every picture a user selects and opens with Enter, all within a fraction of a second. On Windows those copies race: WebView2 lets a second process into the data folder they share only once the first has its browser running, and refuses one that asks sooner. Measured on the Windows box on 2026-09-28, four copies started in the same millisecond made one window between them, as two copies of Microsoft's own sample app do. And the refusal was silent. Tauri answers a request for a window before trying it, then drops the half-made window without a word fuji can hear, so the copies that lost sat idle forever, with no window to close and nothing to end them.

Three rules answer it. A copy's first window takes a turn, holding a lock on a file until its page begins to load, when the web engine is up and the next copy can join it. A flurry opens one window: the copy that takes the turn stamps the file with its launch moment, as the file's modified time, which other copies can read through the lock, and a copy launched within WINDOW_FLURRY of the stamp leaves before making anything. A user who opens a folder's worth of pictures has one to look at, and the rest a flip or a contact sheet away; a copy launched later is a deliberate second launch, and waits its turn for a window of its own. And a copy whose first page never begins to load exits after WINDOW_ARRIVAL_WAIT, so whatever goes wrong, nothing is left running without a window. The one failure that last rule cannot see is a page that begins to load and then never reveals its window, since arriving is all it checks; none has been seen.

The mac should open a selection as one window already, since the shell expects Finder to hand it over in one event, but that is still to be confirmed there. Its later windows never take a turn, because they join a web engine already running in the same process.
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
fn window_label() -> String { format!("window-{}", WINDOW_COUNT.fetch_add(1, Ordering::Relaxed) + 1) }//counted rather than reused, so a label never names two windows even after one closes; capabilities/default.json grants to window-*

/// Make a hidden window for these pictures, for its page to place and reveal
pub fn window_build(app: &AppHandle, paths: Vec<String>) -> tauri::Result<()> {
	let label = window_label();
	open::open_hold(app, &label, paths);//before the window exists, so its page finds them the moment it mounts and asks
	let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::default())
		.title(&app.package_info().name)//the product name from tauri.conf.json, until the page titles the window for what it shows
		.visible(false)//the page places it and then shows it, once it has something to draw
		.fullscreen(false)
		.on_page_load(|_, _| window_arrived())//the one sign a window really came, since build answers before anything is tried
		.build()?;
	window_browser_keys_off(&window);
	Ok(())
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
fn window_launched() -> SystemTime { *WINDOW_LAUNCHED.get_or_init(SystemTime::now) }

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
	if let Err(e) = window_build(app, paths) { crate::log::log(&format!("window: could not make a window, {e}")) }//a window that never appears is worth a line, and there is nobody above to tell
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
	use windows_core::Interface;//for cast, from the version webview2-com is built on rather than fuji's own, which Cargo.toml explains

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

/// Take the window out of fullscreen without changing whether it is showing, which tao's own way does everywhere but windows
#[command]
pub fn window_fullscreen_leave(window: WebviewWindow) -> tauri::Result<()> {
	#[cfg(target_os = "windows")] {
		let w = window.clone();
		return window.run_on_main_thread(move || window_fullscreen_leave_hidden(&w))//one task on the main thread, where tauri makes each change at once rather than queuing it, so nothing can come between the restore and the hide
	}
	#[cfg(not(target_os = "windows"))]
	window.set_simple_fullscreen(false)
}

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
