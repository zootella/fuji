use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{command, AppHandle, Manager, State, WebviewWindow};
#[cfg(target_os = "macos")]
use tauri::Url;//the mac is the only platform with an event carrying these; the others get a command line

/*
A file the operating system handed fuji, because the user double-clicked a picture rather than opening fuji and going looking. This module sets those paths aside for the window being made to show them, and hands them over when that window's page asks for them.

The platforms deliver the same intention by completely different means. macOS keeps one copy of an application running and sends it an Apple event, which Tauri raises as RunEvent::Opened — at launch, and again every time the user opens another picture while fuji is already up. Windows has no event at all: Explorer runs the executable again with the path as an argument, a new process every time, so the whole mechanism is the command line. Linux does what Windows does.

Neither can be handed straight to the page, because both arrive before the webview exists. The window is built first and its page is built after.

**So one rule covers every case: a picture is delivered to the window made for it.** The paths wait here under that window's label until its page mounts and asks. That is the same sentence for the first window on any platform, for a second window macOS opens while fuji is already running, and for the whole separate process Explorer starts — which is why there is no "is fuji launching or already up" question anywhere in this file, and no event telling a page that something new has arrived. window.rs is where a window and its pictures are put together; the two readers below each turn one platform's delivery into a list of paths and decide nothing.

The list is taken rather than read, so nothing is shown twice: a reload during development would otherwise reopen whatever that window was made for.

This module examines nothing, except the one mechanical filter Windows makes necessary. It does not look at the extension, does not ask whether fuji can show the file, and does not choose a view. The page owns all of that, which is the rule in CLAUDE.md about the two layers.
*/

//keyed by window label; the Mutex is required because tauri may hand this to commands on different threads
#[derive(Default)]
pub struct OpenFiles(pub Mutex<HashMap<String, Vec<String>>>);

/// The pictures the window asking was made to show, taken away as it answers
#[command]
pub fn open_files(window: WebviewWindow, files: State<'_, OpenFiles>) -> Vec<String> {//tauri fills in the window that invoked this, which is how a page's own label reaches here without the page ever having to know it has one
	let mut files = files.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());//take the map even if a previous holder panicked
	files.remove(window.label()).unwrap_or_default()
}

/// Set pictures aside for a window about to be built, for its page to collect when it mounts
pub fn open_hold(app: &AppHandle, label: &str, paths: Vec<String>) {
	if paths.is_empty() { return }//a window opened with nothing to show, which is an ordinary launch and the dock asking for a window back
	//an entry leaves this map when its page asks for it, so a window whose page never asks — one that fails to build, or is closed before it mounts — leaves a few strings here until the process ends. Bounded by how many windows a person opens, and not worth code to sweep up
	let state = app.state::<OpenFiles>();
	let mut files = state.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	files.insert(label.to_string(), paths);
}

/// The paths on the command line, which is how windows and linux hand over a double-clicked file
pub fn open_argv() -> Vec<String> {
	std::env::args_os()
		.skip(1)//the first is fuji itself
		.filter_map(|argument| argument.into_string().ok())//an argument that is not utf-8 cannot be a path fuji carries, since everything above here is a rust String and then a javascript string
		.filter(|argument| std::path::Path::new(argument).is_file())//the only examination this module does, and it is mechanical rather than a judgement: a command line also carries switches and a dev build's own arguments, and a path that names a file is what the shell sends and they are not
		.collect()
}

/// The urls macOS sends when the user opens pictures with fuji, as paths; called from RunEvent::Opened
#[cfg(target_os = "macos")]
pub fn open_urls(urls: Vec<Url>) -> Vec<String> {
	urls.iter()
		.filter_map(|url| url.to_file_path().ok())//they are file:// urls, and a url that is not a file is not something this application was ever registered for
		.filter_map(|path| path.to_str().map(|s| s.to_string()))
		.collect()
}
