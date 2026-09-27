use std::sync::atomic::{AtomicUsize, Ordering};
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::{open, settings};

/*
Making fuji's windows, and deciding how long fuji outlives them. This is here rather than in tauri.conf.json because every one of these questions needs something read, measured, or counted first, and because a window is no longer a thing there is exactly one of.

**How many, and who asks for one.** A window is made for the pictures it will show, and `window_build` is the only way one comes into being. Every launch on every platform makes one through `window_first`, when the application loop is ready, for whatever the command line handed over — unless a double-click on the mac has already made one, which `window_first` explains. On macOS a window is made again whenever the running application is told to open something, and again when the user clicks a dock icon with no windows behind it. Windows and Linux never make a second one, because their shells start another process instead — so the same function serves a platform with many windows and a platform with exactly one, and neither needs to know which it is.

**Why the platforms differ at all, since a process per window everywhere would be simpler.** Because the macOS Dock will not have it. Measured on the Mac mini, macOS 15.7.4, 2026-09-13: two running copies of one bundle appear to LaunchServices as two foreground applications — separate serial numbers under a single bundle identifier — and the Dock draws a tile per foreground application rather than per bundle. So several fujis would be a row of identical icons with nothing able to merge them. Windows and Linux hide the same architecture — the Windows taskbar combines buttons for one executable, and GNOME and KDE group by `WM_CLASS` — so macOS is the only shell of the three that needs code.

**What it is called.** Each window gets a label of its own, counted from one, and `capabilities/default.json` grants permissions to the pattern rather than to a name. A window whose label the capability does not match is built and then silently has no permissions at all, which looks like a page that loads and cannot do anything, so the two have to be kept in step.

**How big.** The size the user last left, out of the settings file, which settings.rs reads before any page exists — see that module for why the file is read twice and by whom. A first launch has nothing recorded and gets a fraction of the desktop instead. Every window asks again rather than copying its siblings, so a window opened after another was resized comes back at the newer size.

**Where.** The builder is given a size and no position, and fuji remembers no position — several windows returning to one remembered rectangle would land on top of each other. What happens next is not the same on the two platforms, and the difference is worth knowing because the obvious assumption is wrong.

On Windows the window manager places the window, cascading it down a staircase of its own. **On macOS nothing places it, and tao centers it** — `if attrs.position.is_none() { ns_window.center() }`, read out of tao 0.35.3. Two windows of one size on one screen therefore center to the identical rectangle and stack perfectly. Nothing was ever going to intervene: a plain NSWindow sits exactly where its frame says, and cascading on macOS is an AppKit convenience that document-based applications opt into through NSWindowController and Tauri does not use. Observed on the Mac mini 2026-09-14, which is what put the cascade below into this file.

**And then where it actually went, which is the part that needs correcting.** Windows cascades new windows down a fixed staircase and does not check that what it is placing fits the work area. Measured on 2026-09-13: three instances 1062 pixels tall on a work area 1160 deep, cascaded to 52, 104 and 138, the last two hanging 6 and 40 pixels under the taskbar. Building the window at its true size does not help — that was tried first, on the theory that the operating system would place a window properly if only it were told the truth, and it made no difference.

So the window is checked after it is built and moved if it is out of bounds. The one thing that makes this invisible is that the window is created hidden: the page calls show() later, once it has something to draw, so all of this happens before anyone is looking. Nothing flashes and nothing jumps.

**So fuji staggers its own windows before anything else**, stepping a new one down and to the right of any sibling already standing where it landed, until it stands alone. That is `window_cascade`, and it needs no platform test of its own: it asks whether another window of this process is at this spot, and on Windows and Linux there is never another window of this process, so it does nothing there and the manager's own cascade stands. A window the user has dragged elsewhere frees the spot it left, so the next one opens centered rather than stepping around a ghost.

**A window that is still out of bounds goes somewhere random inside the work area**, rolled on both axes even if only one of them was out. Both axes, because keeping a good one sounds tidier and piles windows up instead: Windows cascades in both directions at once, so every window corrected for hanging off the bottom would keep the same cascaded column and come to rest in a line — trading a stack for a row. The roll is the last resort now rather than the only tool, since a uniform roll was all that was available back when every window was its own process and no copy of fuji could know where its siblings were.

**A window with no room to roll is put at the top left of the work area and allowed to overhang.** It is never resized. A window that could have fitted and did not is a defect the user will rightly blame on fuji; a window larger than the screen it is on has to hang off something, and the user can see why. Of the two edges it could hang off, the far ones are the right choice, because the near ones carry the title bar the window is dragged by.

All four edges are treated the same way by the same code. The bottom is the one that comes up, because fuji's windows are often tall for a standing figure and the Windows taskbar is usually along the bottom — but the taskbar can be moved to any edge, and macOS keeps a menu bar at the top and usually a Dock somewhere, so there is nothing to gain by special casing one of them. `work_area` already answers all of it: Windows resolves it through the same call that knows where the taskbar is, and macOS through the frame that already excludes the menu bar and the Dock.

**How long fuji outlives its last window** is the one place fuji deliberately behaves differently on each platform, and `window_stays_resident` below is the whole of it. On Windows and Linux, closing the window closes fuji, which is what those desktops mean by closing a window. On macOS an application is a place the user is in rather than a window they have open: the dock icon stays, with its dot, which is how a Mac user reopens it, learns they can keep it there, and decides what to quit when the machine is busy. Going with that grain costs almost nothing here, because a fuji with no windows has destroyed its webviews and is the Rust host alone — and the Rust is dumb, so it is doing nothing.
*/

//how big to open when the settings file has nothing to say, which is a first launch: a fraction of the usable desktop, in css pixels
const STARTING_WIDTH_FRACTION: f64 = 0.6;
const STARTING_HEIGHT_FRACTION: f64 = 0.8;
fn window_starting_size(app: &AppHandle) -> (f64, f64) {
	let Ok(Some(monitor)) = app.primary_monitor() else { return (800.0, 600.0) };//no monitor to measure, so the size tauri.conf.json carried before it stopped declaring a window at all
	let area = monitor.work_area();
	let scale = monitor.scale_factor();//work_area is in tauri physical pixels and the builder wants css ones
	(
		(area.size.width  as f64 / scale * STARTING_WIDTH_FRACTION ).round(),
		(area.size.height as f64 / scale * STARTING_HEIGHT_FRACTION).round(),
	)
}

static WINDOW_COUNT: AtomicUsize = AtomicUsize::new(0);//how many windows this process has ever made, which is where the next label comes from
fn window_label() -> String { format!("fuji-{}", WINDOW_COUNT.fetch_add(1, Ordering::Relaxed) + 1) }//counted rather than reused, so a label never names two windows even after one closes; capabilities/default.json grants to fuji-*

/// Make a window for these pictures, at the size the settings file remembers, stepped clear of any sibling it lands on and moved back inside the work area if it ends up outside it
pub fn window_build(app: &AppHandle, paths: Vec<String>) -> tauri::Result<()> {
	let label = window_label();
	open::open_hold(app, &label, paths);//before the window exists, so its page finds them the moment it mounts and asks
	let (width, height) = settings::settings_window_size(app).unwrap_or_else(|| window_starting_size(app));
	let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::default())
		.title("Fuji")
		.inner_size(width, height)//css pixels, which is what the builder calls logical and what the settings file holds
		.visible(false)//the page shows it once it has something to draw, which is also what gives the check below somewhere to happen unseen
		.fullscreen(false)
		.build()?;

	window_cascade(&window);//macos centers every window it is given no position for, so a second one lands exactly on the first
	window_settle(&window);//best effort: a window that cannot be measured is left where it is rather than moved somewhere worse
	Ok(())
}

/// Make fuji's first window, unless something has already made one, which is how a launch produces exactly one window however it was started
pub fn window_first(app: &AppHandle, paths: Vec<String>) {
	if !app.webview_windows().is_empty() { return }//a picture opened by double-click has already been given a window: measured on the Mac mini 2026-09-14, macOS delivers that event 39 milliseconds before setup() even runs and 52 before the one that calls this, so by now it is done. Only the second number is load-bearing; the first is worth knowing because a double-click builds fuji's first window before fuji has finished setting itself up, which is the opposite of what anyone adding to setup() would assume. Making a window in setup instead of here is what used to open two, one of them blank
	window_open(app, paths)//nothing has, so this is an ordinary launch: empty from the dock, or on the pictures the command line carried, which is how windows and linux hand over a double-click
}

/// Make a window and report trouble to the log rather than to a caller who has nowhere to put it; the run event closure is that caller
pub fn window_open(app: &AppHandle, paths: Vec<String>) {
	if let Err(e) = window_build(app, paths) { crate::log::log(&format!("window: could not make a window, {e}")) }//a window that never appears is worth a line, and there is nobody above to tell
}

/// Does fuji keep running once its last window has closed; the essay above is why the platforms answer differently
#[cfg(all(target_os = "macos", not(debug_assertions)))]
pub fn window_stays_resident() -> bool { true }
#[cfg(not(all(target_os = "macos", not(debug_assertions))))]
pub fn window_stays_resident() -> bool { false }//and a debug build answers no on the mac as well, the way associate.rs excuses itself from the registry for its own reasons. pnpm local runs the binary out of target/debug rather than a bundle, so there is no dock tile a user could click to ask for a window back, and closing the window is how a development run is meant to end

//how far to step a window that landed on one of its siblings, in css pixels: about a title bar, which is the step macos itself uses where it cascades at all, and enough to see and to grab
const CASCADE_STEP: f64 = 28.0;
const CASCADE_LIMIT: usize = 16;//stop stepping after this many, because a window marching further than sixteen title bars is worse off than one left where it landed; window_settle then rolls it somewhere inside the work area

//step this window clear of any sibling standing where it landed; nothing at all on a platform where this process has one window
fn window_cascade(window: &WebviewWindow) {
	let others: Vec<PhysicalPosition<i32>> = window.app_handle().webview_windows().iter()
		.filter(|(label, _)| *label != window.label())//this window is already in the list, having just been built
		.filter_map(|(_, sibling)| sibling.outer_position().ok())
		.collect();
	if others.is_empty() { return }//asked first so this really is nothing at all on windows and linux, where a process never has a second window to stand on

	let (Ok(mut at), Ok(scale)) = (window.outer_position(), window.scale_factor()) else { return };
	let step = (CASCADE_STEP * scale).round() as i32;//positions are reported in tauri's physical pixels and the step is written in css ones

	for _ in 0..CASCADE_LIMIT {
		let taken = others.iter().any(|o| (o.x - at.x).abs() < step && (o.y - at.y).abs() < step);//near enough to look stacked rather than exactly equal, so a window nudged a pixel does not hide behind this test
		if !taken { break }
		at.x += step; at.y += step;
	}
	let _ = window.set_position(at);
}

//move the window inside the work area if it has ended up outside, and leave it exactly alone if it has not
fn window_settle(window: &WebviewWindow) {
	let Ok(Some(monitor)) = window.current_monitor() else { return };//the monitor it actually landed on, which on more than one screen is not necessarily the primary one
	let work = monitor.work_area();//the screen minus whatever the operating system keeps: the taskbar on any edge, the menu bar, the dock
	let Ok((at, size)) = window_seen(window) else { return };//the frame as the user sees it, because that is what overhangs rather than the web view inside it or a border nobody can see

	if inside(at.x, size.width, work.position.x, work.size.width)
	&& inside(at.y, size.height, work.position.y, work.size.height) { return }//wholly within the work area, so whatever put it there stands untouched — the manager's own cascade on windows, tao's centering and our step away from a sibling on the mac

	let x = roll(size.width, work.position.x, work.size.width);
	let y = roll(size.height, work.position.y, work.size.height);
	let _ = window_seen_move(window, PhysicalPosition::new(x, y));
}

/*
Where a window is, as the person looking at the screen would say.

Tauri's outer position and size are the platform's own window rectangle, and on the mac that is what anyone would mean: a frame there leaves the shadow out. On Windows 10 and 11 it does not. The rectangle includes the resize borders, which since Windows 10 are invisible, about 7 pixels on the left, the right and the bottom at 100 percent. So the numbers say a window is 7 pixels wider on each side and taller at the bottom than anything drawn on the screen, and a window placed flush with the bottom of the work area by them stops 7 pixels short of it. The desktop window manager knows where the visible frame is, through DwmGetWindowAttribute with DWMWA_EXTENDED_FRAME_BOUNDS, and window_seen is the one place that answer belongs.

It does not have it yet: it passes the outer rectangle through on every platform, until the Windows body is written and measured on that machine. The measuring matters, because fuji places its windows while they are still hidden, and that attribute is known to answer badly for a window that has never been shown. Everything else here is already written against window_seen, so the correction is one function body and nothing that calls it changes.

The page asks through window_frame and window_frame_set, in css pixels, and so never learns that a platform counts borders it does not draw.
*/

/// Where the window is and how big, in css pixels, as the user sees its frame
#[derive(Serialize, Deserialize)]
pub struct Frame { x: f64, y: f64, width: f64, height: f64 }

/// The window's visible frame, in css pixels
#[command]
pub fn window_frame(window: WebviewWindow) -> tauri::Result<Frame> {//tauri turns its own error into the rejection the page sees, so none of these need converting
	let scale = window.scale_factor()?;//tauri's physical pixels per css pixel, on this window's screen
	let (at, size) = window_seen(&window)?;
	Ok(Frame { x: at.x as f64 / scale, y: at.y as f64 / scale, width: size.width as f64 / scale, height: size.height as f64 / scale })
}

/// Put the window's visible frame exactly here, in css pixels
#[command]
pub fn window_frame_set(window: WebviewWindow, frame: Frame) -> tauri::Result<()> {
	let scale = window.scale_factor()?;
	let (_, seen) = window_seen(&window)?;
	let inner = window.inner_size()?;
	let chrome = (seen.width.saturating_sub(inner.width), seen.height.saturating_sub(inner.height));//what the frame has around the content: a title bar and borders, or nothing on a window without decorations. set_size means the content, so this comes off the frame asked for
	let width  = ((frame.width  * scale).round() as u32).saturating_sub(chrome.0);
	let height = ((frame.height * scale).round() as u32).saturating_sub(chrome.1);
	window.set_size(PhysicalSize::new(width, height))?;//size first and then position, because a resize on the mac keeps the bottom left corner, which is where AppKit measures from, and so moves the top; tao queues both onto the main thread, in this order
	window_seen_move(&window, PhysicalPosition::new((frame.x * scale).round() as i32, (frame.y * scale).round() as i32))
}

//the visible frame, in tauri's physical pixels: the outer rectangle on every platform for now, and the place the windows correction goes
fn window_seen(window: &WebviewWindow) -> tauri::Result<(PhysicalPosition<i32>, PhysicalSize<u32>)> {
	Ok((window.outer_position()?, window.outer_size()?))
}

//move the window so its visible corner lands here. set_position places the outer corner, so this steps back by whatever lies between the outer corner and the visible one, which is nothing until window_seen learns otherwise
fn window_seen_move(window: &WebviewWindow, to: PhysicalPosition<i32>) -> tauri::Result<()> {
	let outer = window.outer_position()?;
	let (seen, _) = window_seen(window)?;
	window.set_position(PhysicalPosition::new(to.x - (seen.x - outer.x), to.y - (seen.y - outer.y)))
}

//is this edge and the far one within the work area
fn inside(at: i32, size: u32, work_at: i32, work_size: u32) -> bool {
	at >= work_at && at + size as i32 <= work_at + work_size as i32
}

//one axis: somewhere for this edge inside the work area, rolled uniformly over whatever room is left
fn roll(size: u32, work_at: i32, work_size: u32) -> i32 {
	let slack = work_size as i32 - size as i32;//how much room the window leaves along this axis
	if slack <= 0 { return work_at }//it cannot fit, so put the near edge on the work area and let the far one overhang, because the near edge carries the titlebar
	work_at + fastrand::i32(0..=slack)//room to choose, so choose uniformly
}
