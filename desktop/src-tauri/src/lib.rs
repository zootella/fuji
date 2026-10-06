/*
The boundary. Everything the page may ask Rust to do is named once in this file, in the handler list below, and nothing else is reachable from JavaScript. So that list is the map of fuji's Rust and the whole of its attack surface, and adding a line to it is the deliberate act of widening that surface.

**The page is the application, and Rust is a library beneath it.** The page decides everything: what a setting is, which files to show and in what order, which types to offer Windows under which keys, when to ask and how often. Rust grows only for what the page cannot do well — speed, reaching the operating system, or holding a security wall — and what it adds is a general command any desktop application could use as it stands: read a file, write a registry value, make a thumbnail of a path. The test for a new one is to describe it without naming a fuji feature. "Write this string value under the current user" passes; "register fuji's image types with Windows" fails, and was split into registry.rs and associate.js. A command that has to know why it is called, or that spells a fuji name in its code, is application logic that has leaked down; even the product's two names come through package_info, brandName as name from tauri.conf.json and brandStem as crate_name from Cargo.toml.

**A command takes one thing.** A list stays in the page, which calls once per item and owns the order, how many are in flight, and when to stop; so thumbnail_render takes one path and registry_set writes one value. A crossing costs little, and a loop down here is a decision taken away from the page.

**A command that waits runs its body on the blocking pool.** Tauri runs a plain #[command] on the thread that runs the window, one at a time in the order they arrive. So a command that works on the window stays plain — window.rs — because on that thread tauri makes a change at once, where from anywhere else it only queues the change and the command answers before the window has moved. A command that waits, on the disk, the processor or another program — disk.rs, thumbnail.rs, launch.rs, and panel.rs for the xrandr it runs on linux — is an async fn that hands its body to run_blocking below, so a slow read or a decode never holds up a window event or another command's reply, and a panic in it comes back to the page as an error. The rest return at once and are plain. The essay above disk_readdir in disk.rs says why the bodies go to tokio's blocking pool rather than tauri's workers, and what that costs.

**Rust trusts the page, and guards only what the page cannot.** A check here that repeats a decision the page made is a second copy of it, and second copies go stale. The trust rests on three walls around the page: every path it acts on came from the user or was built by fuji itself, never from outside content; untrusted text reaches it only through Vue's escaping interpolation, so it never becomes script; and the Content-Security-Policy in tauri.conf.json keeps foreign script out even if one of those cracks. The walls Rust does hold are the ones that must stand before the page could look: thumbnail.rs refuses a header claiming more memory than the machine has, before any decoder allocates. disk.rs names the next one, for when deleting arrives.

The plugins are the other half of the surface. Registering one here does not decide how much of it the page can reach; capabilities/default.json does, naming individual permissions, so read the two files together. Each grant is narrowed to the feature that needs it: the dialog plugin to the open and save boxes, which File, Open uses, and the opener to revealing a file, which nothing calls yet, and to one url pattern, Windows Settings' Default apps, which the file types in fuji's settings open for the user. A link to anywhere else is refused.

**No window is made here.** Every window comes from the event closure below, under one rule: Ready makes a window if there is none. A double-click on the Mac delivers Opened before Ready, so the picture already has its window; on Windows and Linux Opened never fires, so Ready always makes it. Making a window in setup instead is what once opened two for one double-click, one of them blank.
*/

mod desktop;//compile desktop.rs as a module named desktop: text the page hands down to be written on the way out
mod disk;//and disk.rs: file commands, thin wrappers over std::fs
mod fit;//and fit.rs: the fits, the arithmetic fit.js has too, which a native thumbnail's size is chosen by
mod launch;//and launch.rs: launch services, the mac's record of which application opens which kind of file
#[cfg(target_os = "macos")]//the dock menu is a macos idea and the module is all AppKit
mod dock;//and dock.rs: the dock icon's own menu, and its one New Window item
mod log;//and log.rs: the log's text, held from both sides and written on the way out
#[cfg(target_os = "macos")]//the whole module is macos-only: it calls tauri menu methods that do not exist on other targets, and a menu belongs along the top of the screen only here
mod menu;//and menu.rs: the menu bar
mod open;//and open.rs: the files the operating system handed over, held for the window made to show them
mod panel;//and panel.rs: how many pixels the main display really has
mod paths;//and paths.rs: where this copy of the program is
mod registry;//and registry.rs: the windows registry, read and written for the page
mod thumbnail;//and thumbnail.rs: the operating system's thumbnailer, one path at a time
mod touch;//and touch.rs: trackpad scrolls dropped before the page sees them, for the windows that asked
mod window;//and window.rs: making windows, placing them, and how long the process outlives them

//the body of a command that waits, run on tokio's blocking pool and answered as a value however it ends: what the body returned, or an error if it panicked. The essay above disk_readdir in disk.rs says why every waiting command goes through here, and what it costs
pub(crate) async fn run_blocking<T: Send + 'static>(body: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
	tauri::async_runtime::spawn_blocking(body).await.map_err(|e| format!("the command panicked: {e}"))?
}

pub fn run() {
	log::log_panics();//before anything can panic, so every panic after this has its place in the log; a line held for the log survives a panic run_blocking catches, and is lost with the rest of the held text in one that ends the process
	window::window_launch();//first of all, so this copy's launch moment is when it started; window.rs tells a flurry of copies from a deliberate second launch by it
	tauri::Builder::default()//start building the Tauri application
		.plugin(tauri_plugin_opener::init())//reveal a file in finder or explorer, and open windows' default apps settings; capabilities grant those two and no other url
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
				launch::launch_opens,//and in launch.rs
				launch::launch_set,
				panel::panel_resolution,//and in panel.rs
				thumbnail::thumbnail_render,//and in thumbnail.rs
				open::open_files,//and in open.rs
				paths::paths_executable,//and in paths.rs
				registry::registry_get,//and in registry.rs
				registry::registry_set,
				registry::registry_delete,
				registry::registry_notify,
				registry::registry_opens,
				touch::touch_block,//and in touch.rs
				window::window_frame,//and in window.rs
				window::window_frame_set,
				window::window_fullscreen_leave,
			]
		)
		.setup(|_app| {//before any page exists, which is the whole reason this is here rather than in the page; the underscore is for windows and linux, where both lines that read it are compiled away
			#[cfg(target_os = "macos")]
			menu::menu_set(_app.handle())?;//the mac alone has a menu bar along the top of the screen; everywhere else this would put a menu inside the window, so menu.rs is gated here rather than in itself
			#[cfg(target_os = "macos")]
			dock::dock_install(_app.handle());//and the other menu, the one on the dock icon
			touch::touch_start();//watch scroll wheel events for the pages that will ask to be spared a trackpad's; every platform calls it and only the mac installs anything
			Ok(())
		})
		.build(tauri::generate_context!())//build and then run with a closure, rather than a bare run that never returns, so the closure below sees every event the loop produces; Exit among them is the last chance to write to disk, and desktop.rs has why
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
