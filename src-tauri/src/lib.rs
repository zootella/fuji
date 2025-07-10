//./src-tauri/src/lib.rs

mod io;//compile io.rs as a module named io

//example command from scaffolding; Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
	format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()//start building the Tauri application
		.plugin(tauri_plugin_opener::init())//add the “open external URL” plugin
		.plugin(tauri_plugin_fs::init())//add the filesystem plugin for scoped access
		.invoke_handler(//register all the commands JS can invoke…
			tauri::generate_handler![
				greet,          //example greeting function above
				io::fs_readdir, //functions we've written in io.rs
				io::fs_stat,
				io::fs_read,
				io::fs_copy,
			]
		)                                                     
		.run(tauri::generate_context!())//launch the app with the generated config (tauri.conf.json)
		.expect("error while running tauri application");//panic if startup fails (e.g. bad config)
}
