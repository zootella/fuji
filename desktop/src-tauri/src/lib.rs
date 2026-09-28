/*
The boundary. Everything the page may ask Rust to do is named once in this file, in the handler list below, and nothing else is reachable from JavaScript. So that list is the map of fuji's Rust and the whole of its attack surface, and adding a line to it is the deliberate act of widening that surface.

**The page is the application, and Rust is a library beneath it.** The page decides everything: what a setting is, which files to show and in what order, which types to offer Windows under which keys, when to ask and how often. Rust grows only for what the page cannot do well — speed, reaching the operating system, or holding a security wall — and what it adds is a general command any desktop application could use as it stands: read a file, write a registry value, make a thumbnail of a path. The test for a new one is to describe it without naming a fuji feature. "Write this string value under the current user" passes; "register fuji's image types with Windows" fails, and was split into registry.rs and associate.js. A command that has to know why it is called, or that spells a fuji name in its code, is application logic that has leaked down; even the product name comes from tauri.conf.json, through package_info.

**A command takes one thing.** A list stays in the page, which calls once per item and owns the order, how many are in flight, and when to stop; so thumbnail_probe takes one path and registry_set writes one value. A crossing costs little, and a loop down here is a decision taken away from the page.

**Rust trusts the page, and guards only what the page cannot.** A check here that repeats a decision the page made is a second copy of it, and second copies go stale. The trust rests on three walls around the page: every path it acts on came from the user or was built by fuji itself, never from outside content; untrusted text reaches it only through Vue's escaping interpolation, so it never becomes script; and the Content-Security-Policy in tauri.conf.json keeps foreign script out even if one of those cracks. The walls Rust does hold are the ones that must stand before the page could look: thumbnail.rs refuses bytes that are not what they claim, and headers claiming more memory than the machine has, before any decoder runs. disk.rs names the next one, for when deleting arrives.

The plugins are the other half of the surface. Registering one here does not decide how much of it the page can reach; capabilities/default.json does, naming individual permissions, so read the two files together. Neither plugin has a caller yet, on purpose: reveal and the file dialogs are the next features, and their grants are already narrowed to what those features need.

**No window is made here.** Every window comes from the event closure below, under one rule: Ready makes a window if there is none. A double-click on the Mac delivers Opened before Ready, so the picture already has its window; on Windows and Linux Opened never fires, so Ready always makes it. Making a window in setup instead is what once opened two for one double-click, one of them blank.
*/

mod desktop;//compile desktop.rs as a module named desktop: text the page hands down to be written on the way out
mod disk;//and disk.rs: file commands, thin wrappers over std::fs
#[cfg(target_os = "macos")]//the dock menu is a macos idea and the module is all AppKit
mod dock;//and dock.rs: the dock icon's own menu, and its one New Window item
mod log;//and log.rs: the log's text, held from both sides and written on the way out
#[cfg(target_os = "macos")]//the whole module is macos-only: it calls tauri menu methods that do not exist on other targets, and a menu belongs along the top of the screen only here
mod menu;//and menu.rs: the menu bar
mod open;//and open.rs: the files the operating system handed over, held for the window made to show them
mod panel;//and panel.rs: how many pixels the main display really has
mod paths;//and paths.rs: where this copy of the program is
mod registry;//and registry.rs: the windows registry, read and written for the page
mod thumbnail;//and thumbnail.rs: the operating system's thumbnailer, behind a probe and a render
mod touch;//and touch.rs: trackpad scrolls dropped before the page sees them, for the windows that asked
mod window;//and window.rs: making windows, placing them, and how long the process outlives them

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
				paths::paths_executable,//and in paths.rs
				registry::registry_get,//and in registry.rs
				registry::registry_set,
				registry::registry_notify,
				touch::touch_block,//and in touch.rs
				window::window_frame,//and in window.rs
				window::window_frame_set,
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
