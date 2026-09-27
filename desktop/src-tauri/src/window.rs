use std::sync::atomic::{AtomicUsize, Ordering};
use serde::Deserialize;
use tauri::{command, AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::open;

/*
Making fuji's windows, saying where one is, and deciding how long fuji outlives them. This is here rather than in tauri.conf.json because a window is no longer a thing there is exactly one of: each is made when it is asked for, under a label counted for it.

**How many, and who asks for one.** A window is made for the pictures it will show, and `window_build` is the only way one comes into being. Every launch on every platform makes one through `window_first`, when the application loop is ready, for whatever the command line handed over — unless a double-click on the mac has already made one, which `window_first` explains. On macOS a window is made again whenever the running application is told to open something, and again when the user clicks a dock icon with no windows behind it. Windows and Linux never make a second one, because their shells start another process instead — so the same function serves a platform with many windows and a platform with exactly one, and neither needs to know which it is.

**Why the platforms differ at all, since a process per window everywhere would be simpler.** Because the macOS Dock will not have it. Measured on the Mac mini, macOS 15.7.4, 2026-09-13: two running copies of one bundle appear to LaunchServices as two foreground applications — separate serial numbers under a single bundle identifier — and the Dock draws a tile per foreground application rather than per bundle. So several fujis would be a row of identical icons with nothing able to merge them. Windows and Linux hide the same architecture — the Windows taskbar combines buttons for one executable, and GNOME and KDE group by `WM_CLASS` — so macOS is the only shell of the three that needs code.

**What it is called.** Each window gets a label of its own, counted from one, and `capabilities/default.json` grants permissions to the pattern rather than to a name. A window whose label the capability does not match is built and then silently has no permissions at all, which looks like a page that loads and cannot do anything, so the two have to be kept in step.

**Where, and how big, is the page's to decide.** Every window is built hidden at whatever size Tauri defaults to, and the shell places it before the reveal: fitted around the picture for a preview, or at the ordinary preset size somewhere in the work area, which library.js keeps. Nothing about a window is remembered between launches. The window is hidden while this happens, so nothing flashes and nothing jumps.

Fuji places its windows itself because neither platform does it well. Windows cascades new ones down a fixed staircase without checking that they fit — measured on 2026-09-13, three windows 1062 pixels tall on a work area 1160 deep, the last two hanging 6 and 40 pixels under the taskbar. macOS places nothing at all, and tao centers every window it is given no position for, so two windows of one size stack exactly on top of each other.

**How long fuji outlives its last window** is the one place fuji deliberately behaves differently on each platform, and `window_stays_resident` below is the whole of it. On Windows and Linux, closing the window closes fuji, which is what those desktops mean by closing a window. On macOS an application is a place the user is in rather than a window they have open: the dock icon stays, with its dot, which is how a Mac user reopens it, learns they can keep it there, and decides what to quit when the machine is busy. Going with that grain costs almost nothing here, because a fuji with no windows has destroyed its webviews and is the Rust host alone — and the Rust is dumb, so it is doing nothing.
*/

static WINDOW_COUNT: AtomicUsize = AtomicUsize::new(0);//how many windows this process has ever made, which is where the next label comes from
fn window_label() -> String { format!("fuji-{}", WINDOW_COUNT.fetch_add(1, Ordering::Relaxed) + 1) }//counted rather than reused, so a label never names two windows even after one closes; capabilities/default.json grants to fuji-*

/// Make a hidden window for these pictures, for its page to place and reveal
pub fn window_build(app: &AppHandle, paths: Vec<String>) -> tauri::Result<()> {
	let label = window_label();
	open::open_hold(app, &label, paths);//before the window exists, so its page finds them the moment it mounts and asks
	WebviewWindowBuilder::new(app, &label, WebviewUrl::default())
		.title("Fuji")
		.visible(false)//the page places it and then shows it, once it has something to draw
		.fullscreen(false)
		.build()?;
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

/*
Where a window is, as the person looking at the screen would say.

Tauri's outer position and size are the platform's own window rectangle, and on the mac that is what anyone would mean: a frame there leaves the shadow out. On Windows 10 and 11 it does not. The rectangle includes the resize borders, which since Windows 10 are invisible, about 7 pixels on the left, the right and the bottom at 100 percent. So the numbers say a window is 7 pixels wider on each side and taller at the bottom than anything drawn on the screen, and a window placed flush with the edge of the work area by them stops 7 pixels short of it. The desktop window manager knows where the visible frame is, through DwmGetWindowAttribute with DWMWA_EXTENDED_FRAME_BOUNDS, and window_seen is the one place that answer belongs.

It does not have it yet: it passes the outer rectangle through on every platform, until the Windows body is written and measured on that machine. The measuring matters, because fuji places its windows while they are still hidden, and that attribute is known to answer badly for a window that has never been shown. Everything else here is already written against window_seen, so the correction is one function body and nothing that calls it changes.

The page places a window through window_frame_set, in css pixels, and so never learns that a platform counts borders it does not draw.
*/

/// Where the window goes and how big, in css pixels, as the user sees its frame
#[derive(Deserialize)]
pub struct Frame { x: f64, y: f64, width: f64, height: f64 }

/// Put the window's visible frame exactly here, in css pixels
#[command]
pub fn window_frame_set(window: WebviewWindow, frame: Frame) -> tauri::Result<()> {//tauri turns its own error into the rejection the page sees, so none of these need converting
	let scale = window.scale_factor()?;//tauri's physical pixels per css pixel, on this window's screen
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
