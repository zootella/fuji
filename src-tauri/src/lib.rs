//./src-tauri/src/lib.rs

mod io;//compile io.rs as a module named io

use tauri::{PhysicalSize, Window};//bring in what we need from tauri

//example command from scaffolding; Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
	format!("Hello, {}! You've been greeted from Rust!", name)
}

//new custom command: get the physical device pixel resolution of the display
#[tauri::command]
fn count_pixels(window: Window) -> Result<(u32, u32, f64), String> {
	let monitor = window
		.current_monitor()
		.map_err(|e| e.to_string())?
		.ok_or("No monitor detected".to_string())?;
	let PhysicalSize {width, height} = *monitor.size();
	let scale = monitor.scale_factor();
	Ok((width, height, scale))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()//start building the Tauri application
		.plugin(tauri_plugin_opener::init())//add the “open external URL” plugin
		.plugin(tauri_plugin_fs::init())//add the filesystem plugin for scoped access
		.invoke_handler(//register all the commands JS can invoke…
			tauri::generate_handler![
				greet,          //functions from above
				count_pixels,
				io::io_readdir, //functions we've written in io.rs
				io::io_stat,
				io::io_read,
				io::io_copy,
			]
		)                                                     
		.run(tauri::generate_context!())//launch the app with the generated config (tauri.conf.json)
		.expect("error while running tauri application");//panic if startup fails (e.g. bad config)
}
