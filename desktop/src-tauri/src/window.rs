use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::{open, settings};

/*
Making fuji's windows, and deciding how long fuji outlives them. This is here rather than in tauri.conf.json because every one of these questions needs something read, measured, or counted first, and because a window is no longer a thing there is exactly one of.

**How many, and who asks for one.** A window is made for the pictures it will show, and `window_build` is the only way one comes into being. `lib.rs` calls it once during setup for whatever the command line handed over, which is every launch on every platform. On macOS it calls it again whenever the running application is told to open something, and again when the user clicks a dock icon with no windows behind it. Windows and Linux never make a second one, because their shells start another process instead — so the same function serves a platform with many windows and a platform with exactly one, and neither needs to know which it is. instances.md argues all of that.

**What it is called.** Each window gets a label of its own, counted from one, and `capabilities/default.json` grants permissions to the pattern rather than to a name. A window whose label the capability does not match is built and then silently has no permissions at all, which looks like a page that loads and cannot do anything, so the two have to be kept in step.

**How big.** The size the user last left, out of the settings file, which settings.rs reads before any page exists — see that module for why the file is read twice and by whom. A first launch has nothing recorded and gets a fraction of the desktop instead. Every window asks again rather than copying its siblings, so a window opened after another was resized comes back at the newer size.

**Where.** The builder is given a size and no position, and fuji remembers no position — several windows returning to one remembered rectangle would land on top of each other. What happens next is not the same on the two platforms, and the difference is worth knowing because the obvious assumption is wrong.

On Windows the window manager places the window, cascading it down a staircase of its own. **On macOS nothing places it, and tao centres it** — `if attrs.position.is_none() { ns_window.center() }`, read out of tao 0.35.3. Two windows of one size on one screen therefore centre to the identical rectangle and stack perfectly. Nothing was ever going to intervene: a plain NSWindow sits exactly where its frame says, and cascading on macOS is an AppKit convenience that document-based applications opt into through NSWindowController and Tauri does not use. Observed on the Mac mini 2026-09-14, which is what put the cascade below into this file.

**And then where it actually went, which is the part that needs correcting.** Windows cascades new windows down a fixed staircase and does not check that what it is placing fits the work area. Measured on 2026-09-13: three instances 1062 pixels tall on a work area 1160 deep, cascaded to 52, 104 and 138, the last two hanging 6 and 40 pixels under the taskbar. Building the window at its true size does not help — that was tried first, on the theory that the operating system would place a window properly if only it were told the truth, and it made no difference.

So the window is checked after it is built and moved if it is out of bounds. The one thing that makes this invisible is that the window is created hidden: the page calls show() later, once it has something to draw, so all of this happens before anyone is looking. Nothing flashes and nothing jumps.

**So fuji staggers its own windows before anything else**, stepping a new one down and to the right of any sibling already standing where it landed, until it stands alone. That is `window_cascade`, and it needs no platform test of its own: it asks whether another window of this process is at this spot, and on Windows and Linux there is never another window of this process, so it does nothing there and the manager's own cascade stands. A window the user has dragged elsewhere frees the spot it left, so the next one opens centred rather than stepping around a ghost.

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

	window_cascade(&window);//macos centres every window it is given no position for, so a second one lands exactly on the first
	window_settle(&window);//best effort: a window that cannot be measured is left where it is rather than moved somewhere worse
	Ok(())
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
	let (Ok(at), Ok(size)) = (window.outer_position(), window.outer_size()) else { return };//outer at both ends, because the frame is what overhangs rather than the web view inside it

	if inside(at.x, size.width, work.position.x, work.size.width)
	&& inside(at.y, size.height, work.position.y, work.size.height) { return }//wholly within the work area, so whatever put it there stands untouched — the manager's own cascade on windows, tao's centring and our step away from a sibling on the mac

	let x = roll(size.width, work.position.x, work.size.width);
	let y = roll(size.height, work.position.y, work.size.height);
	let _ = window.set_position(PhysicalPosition::new(x, y));
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
