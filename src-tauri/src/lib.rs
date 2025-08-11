//./src-tauri/src/lib.rs

mod io;//compile io.rs as a module named io

pub fn run() {
	tauri::Builder::default()//start building the Tauri application
		.plugin(tauri_plugin_opener::init())//add the “open external URL” plugin
		.plugin(tauri_plugin_fs::init())//add the filesystem plugin for scoped access
		.invoke_handler(//register all the commands JS can invoke…
			tauri::generate_handler![
				greet,          //functions from below
				hard_vertical,
				io::io_readdir, //functions we've written in io.rs
				io::io_stat,
				io::io_read,
				io::io_copy,
			]
		)                                                     
		.run(tauri::generate_context!())//launch the app with the generated config (tauri.conf.json)
		.expect("error while running tauri application");//panic if startup fails (e.g. bad config)
}

//example command from scaffolding; Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
	format!("Hello, {}! You've been greeted from Rust!", name)
}

//we're down here in Rust to count how many rows of pixels are on the hardware display panel, not CSS pixels, not OS scaled pixels, real ones
#[tauri::command]
fn hard_vertical() -> u32 {
	platform::hard_vertical()//returns a pixel count, or 0 for any error or inability to find the number
}
#[cfg(target_os = "windows")]
mod platform {
	use windows::Win32::Graphics::Gdi::{GetSystemMetrics, SM_CYSCREEN};
	use std::panic;
	pub fn hard_vertical() -> u32 {
		panic::catch_unwind(|| unsafe {
			GetSystemMetrics(SM_CYSCREEN) as u32
		})
		.unwrap_or(0)
	}
}
#[cfg(target_os = "macos")]
pub mod platform {
	use core_graphics::display::{
		CGMainDisplayID,
		CGDisplayCopyAllDisplayModes,
		CGDisplayMode,
	};
	use foreign_types_shared::ForeignType;
	use std::ffi::c_void;
	extern "C" {
		fn CFArrayGetCount(array: *const c_void) -> isize;
		fn CFArrayGetValueAtIndex(array: *const c_void, index: isize) -> *const c_void;
	}
	pub fn hard_vertical() -> u32 {
		unsafe {
			let id = CGMainDisplayID();
			let modes = CGDisplayCopyAllDisplayModes(id, std::ptr::null());//list all available display modes
			if modes.is_null() {
				return 0;
			}
			let count = CFArrayGetCount(modes as *const c_void);//array length
			let mut max_height = 0;//winning highest pixel count; unfortunately a is hardware property is not exposed to us
			for i in 0..count {
				let mode_ref = CFArrayGetValueAtIndex(modes as *const c_void, i);
				let mode = CGDisplayMode::from_ptr(mode_ref as *mut _);
				let h = mode.pixel_height() as u32;//get the framebuffer height of this mode
				if h > max_height {
					max_height = h;
				}
			}
			max_height
		}
	}
}
#[cfg(target_os = "linux")]
mod platform {
	use std::process::Command;
	pub fn hard_vertical() -> u32 {
		let output = match Command::new("xrandr").arg("--query").output() {
			Ok(o) => o,
			Err(_) => return 0,
		};
		let stdout = String::from_utf8_lossy(&output.stdout);
		for line in stdout.lines() {
			if line.contains(" connected primary ") {
				if let Some(res_part) = line.split_whitespace().find(|w| w.contains('+')) {
					if let Some((resolution, _)) = res_part.split_once('+') {
						if let Some((_, height_str)) = resolution.split_once('x') {
							if let Ok(height) = height_str.parse::<u32>() {
								return height;
							}
						}
					}
				}
			}
		}
		0
	}
}
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod platform {
	pub fn hard_vertical() -> u32 {
		0
	}
}
