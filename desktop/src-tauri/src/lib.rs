/*
The boundary, and the table of contents. Everything the page is allowed to ask Rust to do is named once in this file, and anything not named here is unreachable from JavaScript no matter what the page tries to invoke. So this is both the map of fuji's Rust and the whole of its attack surface, and adding a line to the handler list below is the deliberate act of widening it.

The other half of that surface is the plugins. A plugin brings a family of commands written by somebody else, and registering one here does not decide how much of it the page may reach — capabilities/default.json does, naming individual permissions rather than a plugin's default set. Read the two files together; neither tells the whole story alone.

Neither plugin has a caller in the page yet, and both are here on purpose: reveal and the file dialogs are the next features, the grants beside them are already narrowed to what those features need, and taking them out to put them back is churn rather than safety.

Three registrations, one piece of setup, and a launch is all this does. Plugins, then the shared state that outlives any one command, then the commands themselves, then the one thing that has to happen before the page exists, then start.

The launch is split on purpose. Tauri's builder offers .run(), which starts the application and never returns; this file calls .build() and then .run(closure) instead, because the closure is handed every event the application loop produces, and one of them — Exit — is fuji's last chance to write anything to disk. desktop.rs carries the long version of why that event and no other.

For a reader new to Rust: `mod desktop;` compiles the sibling file desktop.rs as a module of this one, which is how a Rust program is assembled — there is no import path listing files, the module declarations are the listing. The chain of dots is a builder, each call returning the thing it was called on so the next can follow. `.expect(...)` says take the value or panic with this message, which is the right shape at startup because a configuration fuji cannot parse is a bug to fix rather than a condition to survive.
*/

mod associate;//each of these compiles the sibling .rs file of the same name
mod desktop;
mod disk;
mod log;
mod open;
mod panel;
mod thumbnail;

pub fn run() {
	tauri::Builder::default()//start building the Tauri application
		.plugin(tauri_plugin_opener::init())//reveal a file in finder or explorer; capabilities grant only reveal, not url opening
		.plugin(tauri_plugin_dialog::init())//the familiar os open and save dialog boxes; capabilities grant only those two, not message boxes
		.manage(desktop::ExitFiles::default())//shared state any command can reach: text handed down to be written on the way out
		.manage(open::OpenFiles::default())//and the paths the operating system handed fuji, waiting for the page to be built and ask for them
		.invoke_handler(//the complete list of what javascript may invoke; a name absent here cannot be called at all
			tauri::generate_handler![
				disk::disk_readdir, //functions we've written in disk.rs
				disk::disk_stat,
				disk::disk_read,
				disk::disk_write,
				disk::disk_copy,
				desktop::desktop_exit_hold,//and in desktop.rs
				log::log_start,//and in log.rs
				log::log_append,
				panel::panel_resolution,//and in panel.rs
				thumbnail::thumbnail_render,//and in thumbnail.rs
				thumbnail::thumbnail_probe,
				open::open_files,//and in open.rs
				associate::associate_register,//and in associate.rs
			]
		)
		.setup(|app| { open::open_argv(app.handle()); Ok(()) })//before the window is built, because on windows and linux a double-clicked file arrives as an argument to this process and there is no later event to catch it
		.build(tauri::generate_context!())//build rather than run, so the closure below gets the event loop
		.expect("error while building tauri application")//panic if startup fails (e.g. bad config)
		.run(|app, event| {//this closure sees every event the application loop produces, for the life of the process
			match event {
				tauri::RunEvent::Exit => { desktop::desktop_exit_write(app); log::log_write() }//the one event every quit path reaches; desktop.rs says why
				#[cfg(target_os = "macos")]//the variant is gated to macos, ios and android in tauri itself, so an arm without this would not compile on windows
				tauri::RunEvent::Opened { urls } => open::open_urls(app, urls),//the user double-clicked a picture, at launch or while fuji was already running; open.rs says why it is held rather than delivered
				_ => {}//RunEvent is non-exhaustive, and everything else is somebody else's business
			}
		});
}
