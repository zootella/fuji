/*
How many panel pixels are really on the glass. Not css pixels, not backing pixels, not whatever the operating system scaled things to — the count the panel was manufactured with. That is what the Mac answers. Windows answers its display resolution, and Linux the current mode xrandr reports, which is the same kind of answer: backing, and the panel only while the display runs at its native resolution. A user who lowered the resolution instead has the monitor or the graphics card stretching backing onto the panel, and fuji neither detects that nor adjusts for it.

The page cannot answer this. screen.width is css pixels, and devicePixelRatio is backing per css, so multiplying them gives backing pixels rather than panel ones. On a MacBook Air running a scaled resolution those numbers describe a convenient fiction: 1710 x 1112 css against a 2560 x 1664 panel, at a ratio of panel to css that matches neither. Fuji needs the real count so that "100%" can mean one image pixel sitting on one panel pixel, which is a promise only this file can keep. library.js collects all three numbers together in measureScreen.

Every operating system answers differently and none of them answers directly, so this is one command with four bodies and the detail sits beside the code it belongs to.

What none of the four does is notice which display fuji is on. They all ask about the main one, so a window dragged to a second monitor of a different resolution still gets told about the first. The way in, when that matters, is Tauri's own window.current_monitor(), which names the display a window occupies; each body below would then take that display rather than assuming the main one. The display_info crate is the other road, trading this file's unsafe blocks for a dependency that enumerates monitors and marks the primary itself.
*/

use serde::{Serialize, Deserialize};
use crate::run_blocking;

//serde turns this into json crossing to javascript; Copy makes it a value that is duplicated rather than moved, which is what you want from a pair of numbers
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Arrow {//general {x, y} pair useful for a positions or a dimensions
	pub x: u32,
	pub y: u32,
}

//the one command javascript calls; the body it reaches was chosen when this was compiled
#[tauri::command]//its body on the blocking pool because linux answers by running xrandr and waiting for it; the mac asks CoreGraphics and windows asks user32, and neither minds which thread asks
pub async fn panel_resolution() -> Arrow {
	run_blocking(|| Ok(platform::panel_resolution())).await.unwrap_or(Arrow { x: 0, y: 0 })//a {x, y} pixel count, or {0, 0} for any error or inability to find the answer, a panic in a body below included
}

#[cfg(target_os = "windows")]//conditional compilation: the bodies that do not match are absent from the binary, not skipped at runtime
mod platform {
	use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};//user32's stable way to ask the desktop's shape
	use super::Arrow;//super is the module above, so this is the Arrow declared at the top of this file

	pub fn panel_resolution() -> Arrow {
		unsafe {//unsafe promises the compiler we checked what it cannot
			let width = GetSystemMetrics(SM_CXSCREEN);//the primary display's resolution, in backing pixels because the process is dpi aware, and the panel's own only while the display runs at its native resolution
			let height = GetSystemMetrics(SM_CYSCREEN);
			if width <= 0 || height <= 0 {//a signed int, so compare before casting: a negative cast to u32 becomes an enormous number
				return Arrow { x: 0, y: 0 };
			}
			Arrow { x: width as u32, y: height as u32 }//every "as" is a conversion someone chose, and a block's last expression is its value
		}
	}
}

/*
Quartz has no call that says "this is the panel's resolution." What it has is the list of every mode the display can be set to, and the tallest of those is the panel's own, because a display cannot offer more pixels than it has. So: ask for the whole list, keep the biggest. It is a heuristic rather than a fact — a display offering a mode taller than its own panel would defeat it — and it has been checked on one machine, a MacBook Air, where it returns the true 2560 x 1664.

Two simpler calls were tried first and both answer a different question. CGDisplayPixelsHigh is one line and looks exactly right, but it reports the current display mode rather than the panel, so a scaled retina display answers with the size of the desktop it is pretending to be — 1112 css pixels where the panel has 1664. CGDisplay::main().display_mode() fails from the other direction: it gives the backing pixels of whichever mode is set, and macOS will happily render a scaled mode larger than the panel and downsample.

CoreFoundation's memory rules decide the shape of the loop below. A function with Create or Copy in its name hands you something you own and must release; a function with Get in its name lends you something you must not. Rust makes the second half easy to get wrong, because wrapping a borrowed pointer in a type that knows how to release it is exactly what you would do with an owned one — and this file did that for a year, releasing each mode once more than it had been retained, which corrupts memory rather than panicking or failing in any way the code could see. The rule to carry away: a pointer from a Get function must never end up somewhere with a destructor.
*/
#[cfg(target_os = "macos")]
mod platform {
	use core_graphics::display::{
		CGMainDisplayID,
		CGDisplayCopyAllDisplayModes,
		CGDisplayMode,
	};
	use foreign_types_shared::ForeignType;
	use super::Arrow;
	use std::ffi::c_void;//an address with no type attached, which is how C describes a pointer to anything
	use std::mem::ManuallyDrop;
	
	extern "C" {//declared by hand because the crate wrapping Quartz does not wrap CFArray; the linker finds the real ones
		fn CFArrayGetCount(array: *const c_void) -> isize;
		fn CFArrayGetValueAtIndex(array: *const c_void, index: isize) -> *const c_void;
		fn CFRelease(cf: *const c_void);
	}
	
	pub fn panel_resolution() -> Arrow {
		unsafe {
			let id = CGMainDisplayID();//the display with the menu bar on it, not necessarily the one fuji is showing on
			let modes = CGDisplayCopyAllDisplayModes(id, std::ptr::null());//Copy in the name, so this array is ours to release below. The null options are load-bearing: pass kCGDisplayShowDuplicateLowResolutionModes instead and the list gains the scaled modes' backing stores, so the tallest below becomes 3420 by 2224 on a machine whose panel is 2560 by 1664 — the backing store, which is the one answer this function exists not to give. measured on a MacBook Air
			if modes.is_null() {
				return Arrow { x: 0, y: 0 };
			}
			
			let count = CFArrayGetCount(modes as *const c_void);//array length
			let mut winning_height = 0;//let is immutable by default, so mut is what makes these two assignable in the loop
			let mut winner = Arrow { x: 0, y: 0 };
			
			for i in 0..count {//an exclusive range, so 0 up to but not including count
				let mode_ref = CFArrayGetValueAtIndex(modes as *const c_void, i);//Get in the name, so this one is only lent to us
				if mode_ref.is_null() {
					continue;
				}
				let mode = ManuallyDrop::new(CGDisplayMode::from_ptr(mode_ref as *mut _));//from_ptr would release this mode; ManuallyDrop cancels that
				let height = mode.pixel_height() as u32;//backing pixels, not css: a mode that looks like 1920 x 1080 is backed by 3840 x 2160
				let width = mode.pixel_width() as u32;
				if height > winning_height {
					winning_height = height;
					winner = Arrow { x: width, y: height };
				}
			}
			
			CFRelease(modes as *const c_void);//necessary to release the array
			winner
		}
	}
}

#[cfg(target_os = "linux")]
mod platform {
	use std::process::Command;//shell out to xrandr rather than bind the RandR C api, which every build machine would then need
	use super::Arrow;
	
	pub fn panel_resolution() -> Arrow {
		//reads output meant for a person, and speaks only to X11: a wayland session has no xrandr to answer
		let output = match Command::new("xrandr").arg("--query").output() {//Err(_) says we want to know there was an error, not which
			Ok(o) => o,
			Err(_) => return Arrow { x: 0, y: 0 },
		};
		let stdout = String::from_utf8_lossy(&output.stdout);//lossy: replace anything that is not valid utf-8 rather than failing over a stray byte
		for line in stdout.lines() {
			//each "if let Some" is one step that found its piece or gives up, so a surprise falls out the bottom as no answer rather than a wrong one
			if line.contains(" connected primary ") {
				if let Some(res_part) = line.split_whitespace().find(|w| w.contains('+')) {//the geometry field, written like "2560x1440+0+0"
					if let Some((resolution, _)) = res_part.split_once('+') {
						if let Some((width_str, height_str)) = resolution.split_once('x') {
							if let (Ok(width), Ok(height)) = (width_str.parse::<u32>(), height_str.parse::<u32>()) {
								return Arrow { x: width, y: height };
							}
						}
					}
				}
			}
		}
		Arrow { x: 0, y: 0 }
	}
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod platform {//everything else, so the file still compiles somewhere new; zero means no answer
	use super::Arrow;
	pub fn panel_resolution() -> Arrow {
		Arrow { x: 0, y: 0 }
	}
}
