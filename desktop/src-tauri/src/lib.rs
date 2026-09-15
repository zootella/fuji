/*
The boundary, and the table of contents. Everything the page is allowed to ask Rust to do is named once in this file, and anything not named here is unreachable from JavaScript no matter what the page tries to invoke. So this is both the map of fuji's Rust and the whole of its attack surface, and adding a line to the handler list below is the deliberate act of widening it.

The other half of that surface is the plugins. A plugin brings a family of commands written by somebody else, and registering one here does not decide how much of it the page may reach — capabilities/default.json does, naming individual permissions rather than a plugin's default set. Read the two files together; neither tells the whole story alone.

Neither plugin has a caller in the page yet, and both are here on purpose: reveal and the file dialogs are the next features, the grants beside them are already narrowed to what those features need, and taking them out to put them back is churn rather than safety.

Three registrations, one piece of setup, and a launch is all this does. Plugins, then the shared state that outlives any one command, then the commands themselves, then the menu, then start.

**No window is made here.** Every window fuji has comes from the event closure below, under one rule rather than a launch case and a running case: Ready makes a window if fuji has none. A double-click on the mac delivers Opened before Ready, so the picture already has its window and Ready finds one and does nothing; a launch with nothing to show reaches Ready empty-handed and gets its window there; on windows and linux Opened never fires at all, so Ready is always the one. Making the window in setup instead is what made a double-click open two, one of them blank.

The launch is split on purpose. Tauri's builder offers .run(), which starts the application and never returns; this file calls .build() and then .run(closure) instead, because the closure is handed every event the application loop produces, and one of them — Exit — is fuji's last chance to write anything to disk. desktop.rs carries the long version of why that event and no other.

For a reader new to Rust: `mod desktop;` compiles the sibling file desktop.rs as a module of this one, which is how a Rust program is assembled — there is no import path listing files, the module declarations are the listing. The chain of dots is a builder, each call returning the thing it was called on so the next can follow. `.expect(...)` says take the value or panic with this message, which is the right shape at startup because a configuration fuji cannot parse is a bug to fix rather than a condition to survive.
*/

mod associate;//each of these compiles the sibling .rs file of the same name
mod desktop;
mod disk;
mod log;
#[cfg(target_os = "macos")]//the whole module is macos-only: it calls tauri menu methods that do not exist on other targets, and a menu belongs along the top of the screen only here
mod menu;
mod open;
mod panel;
mod settings;
mod thumbnail;
mod window;

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
		.setup(|app| {//before any page exists, which is the whole reason this is here rather than in the page
			#[cfg(target_os = "macos")]
			menu::menu_set(app.handle())?;//the mac alone has a menu bar along the top of the screen; everywhere else this would put a menu inside the window, so menu.rs is gated here rather than in itself
			Ok(())
		})
		.build(tauri::generate_context!())//build rather than run, so the closure below gets the event loop
		.expect("error while building tauri application")//panic if startup fails (e.g. bad config)
		.run(|app, event| {//this closure sees every event the application loop produces, for the life of the process
			match event {
				tauri::RunEvent::Ready => window::window_first(app, open::open_argv()),//fuji's first window, unless a picture has already been given one; window.rs has why that is a whole rule rather than a launch case
				tauri::RunEvent::Exit => { desktop::desktop_exit_write(app); log::log_write() }//the one event every quit path reaches; desktop.rs says why
				tauri::RunEvent::ExitRequested { code, api, .. } => {//raised only when the last window is destroyed, and never by the mac quit menu or a logout, which reach Exit above instead
					desktop::desktop_exit_write(app); log::log_write();//write now, rather than at a quit that on the mac may be hours away or may never come before the machine is turned off
					if code.is_none() && window::window_stays_resident() { api.prevent_exit() }//a code means somebody asked to exit on purpose; without one this is the last window closing, and the mac alone keeps the process and its dock icon after that
				}
				#[cfg(target_os = "macos")]//the variant is gated to macos, ios and android in tauri itself, so an arm without this would not compile on windows
				tauri::RunEvent::Opened { urls } => window::window_open(app, open::open_urls(urls)),//the user opened pictures with fuji; at launch this is what fuji started for and arrives before the arm above, and afterwards it is a request for another window. Either way the pictures get a window of their own
				#[cfg(target_os = "macos")]
				tauri::RunEvent::Reopen { has_visible_windows, .. } => { if !has_visible_windows { window::window_open(app, vec![]) } }//the dock icon clicked with nothing behind it, which is how a mac user asks a resident application for a window back
				#[cfg(target_os = "macos")]//this variant exists on every desktop, unlike the two above; what is mac-only is fuji's menu module, which nothing off the mac compiles
				tauri::RunEvent::MenuEvent(event) => menu::menu_chosen(app, event),//fuji makes a window itself and hands the other two items to the page, which already knows how to do them
				_ => {}//RunEvent is non-exhaustive, and everything else is somebody else's business
			}
		});
}
