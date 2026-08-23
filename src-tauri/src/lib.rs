//./src-tauri/src/lib.rs

mod disk;//compile disk.rs as a module named disk
mod panel;

pub fn run() {
	tauri::Builder::default()//start building the Tauri application
		.plugin(tauri_plugin_opener::init())//reveal a file in finder or explorer; capabilities grant only reveal, not url opening
		.plugin(tauri_plugin_dialog::init())//the familiar os open and save dialog boxes; capabilities grant only those two, not message boxes
		.invoke_handler(//register all the commands JS can invoke…
			tauri::generate_handler![
				disk::disk_readdir, //functions we've written in disk.rs
				disk::disk_stat,
				disk::disk_read,
				disk::disk_copy,
				panel::panel_resolution,//and in panel.rs
			]
		)                                                     
		.run(tauri::generate_context!())//launch the app with the generated config (tauri.conf.json)
		.expect("error while running tauri application");//panic if startup fails (e.g. bad config)
}
