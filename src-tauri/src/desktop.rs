//./src-tauri/src/desktop.rs

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{command, AppHandle, Manager, State};

use crate::disk;

/*
Where fuji meets the actions the user takes on the desktop itself rather than inside the window. Today that is one action, the way out: hand it a path and the text that ought to be at that path, and it writes them when the application exits. It never looks inside the text, and a second caller with a different path needs nothing added here, even though the settings file is why it exists.

It takes text rather than bytes because fuji speaks UTF-8 everywhere and never offers a choice of encoding. Rust's String is UTF-8 by construction, so what arrives here is already the bytes JavaScript would have encoded, guaranteed by the type rather than by agreement — and it crosses as a string the size of the file, where a byte array would cross as one JSON number per byte.

The reason it lives down here is that the page cannot do it. RunEvent::Exit is the one event every quit path reaches: closing the last window raises RunEvent::ExitRequested first, but that never fires for the macOS Quit menu item, the Dock's Quit, or a logout, which all send terminate: straight to the application; tao turns that into applicationWillTerminate:, then LoopDestroyed, which arrives here as Exit. The webview is in fact still alive at that moment — what is gone is the opportunity, because Exit arrives on the main thread inside an operating system callback. Synchronous work like a file write is fine, but a round trip to JavaScript needs the main thread's run loop to turn, and it will not turn again before the process is gone.
*/

#[derive(Default)]
pub struct ExitFiles(pub Mutex<HashMap<String, String>>);//the text to write when the application exits, by path

/// Update the text to write to a path when the application exits, or hand down nothing to forget one
#[command]
pub fn desktop_exit_hold(files: State<'_, ExitFiles>, path: String, text: String) {
	let mut files = files.0.lock().unwrap();
	if text.is_empty() { files.remove(&path); } else { files.insert(path, text); }//an update to nothing is how a caller says the path is already current and needs no write
}

/// Write everything held; called from RunEvent::Exit
pub fn desktop_exit_write(app: &AppHandle) {
	let state = app.state::<ExitFiles>();
	let mut files = state.0.lock().unwrap();
	for (path, text) in files.drain() {
		if let Err(e) = disk::disk_write(path, text.into_bytes()) {//a rename, not a conversion: a String is already the utf-8 bytes disk_write wants
			eprintln!("fuji could not write a file on the way out: {e}");//nothing above can be told now, and the process is leaving
		}
	}
}
