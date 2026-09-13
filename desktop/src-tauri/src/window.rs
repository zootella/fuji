use tauri::{App, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::settings;

/*
Making fuji's window, which happens here rather than in tauri.conf.json because two of the three decisions need something read or measured first.

**How big.** The size the user last left, out of the settings file, which settings.rs reads before the page exists — see that module for why the file is read twice and by whom. A first launch has nothing recorded and gets a fraction of the desktop instead.

**Where.** Nowhere: the builder is given a size and no position, and the window manager places it. That is deliberate and instances.md argues it — fuji can be running several times over, so a remembered position would stack every copy in the same rectangle, and the window manager is the only thing that knows where the other windows are. Fuji cannot know, because the instances share nothing by design.

**And then where it actually went, which is the part that needs correcting.** Windows cascades new windows down a fixed staircase and does not check that what it is placing fits the work area. Measured on 2026-09-13: three instances 1062 pixels tall on a work area 1160 deep, cascaded to 52, 104 and 138, the last two hanging 6 and 40 pixels under the taskbar. Building the window at its true size does not help — that was tried first, on the theory that the operating system would place a window properly if only it were told the truth, and it made no difference.

So the window is checked after it is built and moved if it is out of bounds. The one thing that makes this invisible is that the window is created hidden: the page calls show() later, once it has something to draw, so all of this happens before anyone is looking. Nothing flashes and nothing jumps.

**A window that is moved goes somewhere random inside the work area**, rolled on both axes even if only one of them was out. Random rather than a corner, or a clamp to the nearest edge, because the instances cannot coordinate: no copy of fuji can know where its siblings are without asking the operating system about them, which is exactly the awareness this design exists to avoid. A uniform roll separates windows statistically instead, which is the only kind of separation available to processes that do not speak.

Both axes, because keeping a good one sounds tidier and piles windows up instead. The operating system cascades in both directions at once, so every window corrected for hanging off the bottom would keep the same cascaded column and come to rest in a line — trading a stack for a row.

**A window with no room to roll is put at the top left of the work area and allowed to overhang.** It is never resized. A window that could have fitted and did not is a defect the user will rightly blame on fuji; a window larger than the screen it is on has to hang off something, and the user can see why. Of the two edges it could hang off, the far ones are the right choice, because the near ones carry the title bar the window is dragged by.

All four edges are treated the same way by the same code. The bottom is the one that comes up, because fuji's windows are often tall for a standing figure and the Windows taskbar is usually along the bottom — but the taskbar can be moved to any edge, and macOS keeps a menu bar at the top and usually a Dock somewhere, so there is nothing to gain by special casing one of them. `work_area` already answers all of it: Windows resolves it through the same call that knows where the taskbar is, and macOS through the frame that already excludes the menu bar and the Dock.
*/

//how big to open when the settings file has nothing to say, which is a first launch: a fraction of the usable desktop, in css pixels
const STARTING_WIDTH_FRACTION: f64 = 0.6;
const STARTING_HEIGHT_FRACTION: f64 = 0.8;
fn window_starting_size(app: &tauri::AppHandle) -> (f64, f64) {
	let Ok(Some(monitor)) = app.primary_monitor() else { return (800.0, 600.0) };//no monitor to measure, so the size tauri.conf.json carried before it stopped declaring a window at all
	let area = monitor.work_area();
	let scale = monitor.scale_factor();//work_area is in tauri physical pixels and the builder wants css ones
	(
		(area.size.width  as f64 / scale * STARTING_WIDTH_FRACTION ).round(),
		(area.size.height as f64 / scale * STARTING_HEIGHT_FRACTION).round(),
	)
}

/// Build fuji's window, at the size the settings file remembers and wherever the window manager puts it, moved back into view if that turns out to be off the edge of it
pub fn window_build(app: &App) -> tauri::Result<()> {
	let (width, height) = settings::settings_window_size(app.handle()).unwrap_or_else(|| window_starting_size(app.handle()));
	let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())//the label capabilities/default.json names; a different one here would silently have no permissions
		.title("Fuji")
		.inner_size(width, height)//css pixels, which is what the builder calls logical and what the settings file holds
		.visible(false)//the page shows it once it has something to draw, which is also what gives the check below somewhere to happen unseen
		.fullscreen(false)
		.build()?;

	window_settle(&window);//best effort: a window that cannot be measured is left where it is rather than moved somewhere worse
	Ok(())
}

//move the window inside the work area if the window manager has put it outside, and leave it exactly alone if it has not
fn window_settle(window: &WebviewWindow) {
	let Ok(Some(monitor)) = window.current_monitor() else { return };//the monitor it actually landed on, which on more than one screen is not necessarily the primary one
	let work = monitor.work_area();//the screen minus whatever the operating system keeps: the taskbar on any edge, the menu bar, the dock
	let (Ok(at), Ok(size)) = (window.outer_position(), window.outer_size()) else { return };//outer at both ends, because the frame is what overhangs rather than the web view inside it

	if inside(at.x, size.width, work.position.x, work.size.width)
	&& inside(at.y, size.height, work.position.y, work.size.height) { return }//wholly within the work area, so the window manager's own placement stands untouched, cascade and all

	//and otherwise both axes are rolled, even the one that was fine. Keeping a good axis sounds tidier and piles windows up instead: the operating system cascades both together, so every window corrected for hanging off the bottom would inherit the same column and stand in a line
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
	work_at + fastrand::i32(0..=slack)//room to choose, so choose uniformly: the only separation available to processes that cannot ask each other where they are
}
