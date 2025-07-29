//./src-tauri/src/lib.rs

mod io;//compile io.rs as a module named io

use tauri::{PhysicalSize, Window};//bring in what we need from tauri
use display_info::DisplayInfo;

/*
#[cfg(target_os = "macos")]//bring in this only on mac
use core_graphics::display::CGDisplay;
*/

//example command from scaffolding; Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
	format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn count_pixels_1(window: Window) -> Result<(u32, u32, f64), String> {
	let monitor = window
		.current_monitor()
		.map_err(|e| e.to_string())?
		.ok_or("No monitor detected".to_string())?;
	let PhysicalSize {width, height} = *monitor.size();
	let scale = monitor.scale_factor();
	Ok((width, height, scale))
}

#[tauri::command]
fn count_pixels_2() -> Result<(u32, u32, f32), String> {
	// fetch all displays
	let mut displays = DisplayInfo::all().map_err(|e| e.to_string())?;
	// pick the primary, or fallback to the first
	let disp = displays
		.iter()
		.find(|d| d.is_primary)
		.cloned()
		.unwrap_or_else(|| displays.remove(0));

	// disp.width/height = hardware pixels, disp.scale_factor = DPI scale
	Ok((disp.width, disp.height, disp.scale_factor))
}

/*
#[tauri::command]
fn count_pixels_3(window: Window) -> Result<(u64, u64, f64), String> {
	// first grab the logical & scale so we can return scale if you still need it
	let monitor = window
		.current_monitor()
		.map_err(|e| e.to_string())?
		.ok_or_else(|| "No monitor detected".to_string())?;
	let scale = monitor.scale_factor();

	// --- macOS native path: CoreGraphics ---
	#[cfg(target_os = "macos")] {
		let mode = CGDisplay::main()
			.display_mode()
			.ok_or_else(|| "Unable to get CGDisplayMode".to_string())?;
		let pixel_w = mode.pixel_width();
		let pixel_h = mode.pixel_height();
		return Ok((pixel_w, pixel_h, scale));
	}

	// --- Windows / Linux fallback ---
	#[cfg(not(target_os = "macos"))] {
		let mut displays = DisplayInfo::all().map_err(|e| e.to_string())?;
		let disp = displays
			.iter()
			.find(|d| d.is_primary)
			.cloned()
			.unwrap_or_else(|| displays.remove(0));
		return Ok((disp.width, disp.height, disp.scale_factor as f64));
	}
}
*/

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()//start building the Tauri application
		.plugin(tauri_plugin_opener::init())//add the “open external URL” plugin
		.plugin(tauri_plugin_fs::init())//add the filesystem plugin for scoped access
		.invoke_handler(//register all the commands JS can invoke…
			tauri::generate_handler![
				greet,          //functions from above
				count_pixels_1,
				count_pixels_2,
//				count_pixels_3,
				io::io_readdir, //functions we've written in io.rs
				io::io_stat,
				io::io_read,
				io::io_copy,
			]
		)                                                     
		.run(tauri::generate_context!())//launch the app with the generated config (tauri.conf.json)
		.expect("error while running tauri application");//panic if startup fails (e.g. bad config)
}
