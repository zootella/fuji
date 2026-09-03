//./src-tauri/src/lib.rs

mod desktop;
mod disk;//compile disk.rs as a module named disk
mod panel;

pub fn run() {
	tauri::Builder::default()//start building the Tauri application
		.plugin(tauri_plugin_opener::init())//reveal a file in finder or explorer; capabilities grant only reveal, not url opening
		.plugin(tauri_plugin_dialog::init())//the familiar os open and save dialog boxes; capabilities grant only those two, not message boxes
		.manage(desktop::ExitFiles::default())//text the page has handed down to be written on the way out
		.invoke_handler(//register all the commands JS can invoke…
			tauri::generate_handler![
				disk::disk_readdir, //functions we've written in disk.rs
				disk::disk_stat,
				disk::disk_read,
				disk::disk_write,
				disk::disk_copy,
				desktop::desktop_exit_hold,//and in desktop.rs
				panel::panel_resolution,//and in panel.rs
			]
		)                                                     
		.build(tauri::generate_context!())//build the app with the generated config (tauri.conf.json) rather than running it right away, so we get the event loop below
		.expect("error while building tauri application")//panic if startup fails (e.g. bad config)
		.run(|app, event| {
			if let tauri::RunEvent::Exit = event { desktop::desktop_exit_write(app) }//the one event every way of quitting reaches; desktop.rs says why the others don't
		});
}
