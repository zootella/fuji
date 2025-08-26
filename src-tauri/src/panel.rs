//./src-tauri/src/panel.rs

//we're down here in Rust to count how many pixels are on the hardware display panel, not CSS pixels, not OS backing nor scaled pixels, real ones

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Arrow {//general {x, y} pair useful for a positions or a dimensions
	pub x: u32,
	pub y: u32,
}

#[tauri::command]
pub fn panel_resolution() -> Arrow {
	platform::panel_resolution()//returns a {x, y} pixel count, or {0, 0} for any error or inability to find the answer
}

#[cfg(target_os = "windows")]
mod platform {
	use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
	use super::Arrow;
	use std::panic;

	pub fn panel_resolution() -> Arrow {
		panic::catch_unwind(|| unsafe {
			let width = GetSystemMetrics(SM_CXSCREEN);//don't cast to unsigned here
			let height = GetSystemMetrics(SM_CYSCREEN);
			if width <= 0 || height <= 0 {//to detect -1 as error here
				return Arrow { x: 0, y: 0 };
			}
			Arrow { x: width as u32, y: height as u32 }//and then cast here
		})
		.unwrap_or(Arrow { x: 0, y: 0 })
	}
}

#[cfg(target_os = "macos")]
mod platform {
	use core_graphics::display::{
		CGMainDisplayID,
		CGDisplayCopyAllDisplayModes,
		CGDisplayMode,
	};
	use foreign_types_shared::ForeignType;
	use super::Arrow;
	use std::ffi::c_void;
	use std::panic;
	
	extern "C" {
		fn CFArrayGetCount(array: *const c_void) -> isize;
		fn CFArrayGetValueAtIndex(array: *const c_void, index: isize) -> *const c_void;
		fn CFRelease(cf: *const c_void);
	}
	
	pub fn panel_resolution() -> Arrow {
		unsafe {
			let id = CGMainDisplayID();
			let modes = CGDisplayCopyAllDisplayModes(id, std::ptr::null());//list all available display modes
			if modes.is_null() {
				return Arrow { x: 0, y: 0 };
			}
			let count = CFArrayGetCount(modes as *const c_void);//array length
			let mut winning_height = 0;//unfortunately, the api doesn't expose a property that says "this resolution is hardware"
			let mut winner = Arrow { x: 0, y: 0 };//so we return the first resolution we found with the tallest height
			
			for i in 0..count {
				let mode_ref = CFArrayGetValueAtIndex(modes as *const c_void, i);
				let mode = CGDisplayMode::from_ptr(mode_ref as *mut _);
				let h = mode.pixel_height() as u32;//get the framebuffer height of this mode
				let w = mode.pixel_width() as u32;//get the framebuffer width of this mode
				if h > winning_height {
					winning_height = h;
					winner = Arrow { x: w, y: h };
				}
			}
			winner
		}
	}
}

#[cfg(target_os = "linux")]
mod platform {
	use std::process::Command;
	use super::Arrow;
	
	pub fn panel_resolution() -> Arrow {
		let output = match Command::new("xrandr").arg("--query").output() {
			Ok(o) => o,
			Err(_) => return Arrow { x: 0, y: 0 },
		};
		let stdout = String::from_utf8_lossy(&output.stdout);
		for line in stdout.lines() {
			if line.contains(" connected primary ") {
				if let Some(res_part) = line.split_whitespace().find(|w| w.contains('+')) {
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
mod platform {
	use super::Arrow;
	pub fn panel_resolution() -> Arrow {
		Arrow { x: 0, y: 0 }
	}
}
