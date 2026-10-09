use tauri::{command, WebviewWindow};
use crate::run_blocking;

//the operating system's own open box, which only rust can put up. On the Mac it is AppKit's NSOpenPanel, called directly, because the Mac's box takes a file or a folder alike and the dialog plugin asks for one or the other; off the Mac it is the plugin's Rust half, since the open boxes of Windows and GTK choose files or folders and never both in one. None of the plugin is granted to the page, so these are the only way it reaches a dialog

/// Put the system's Open box up over the window asking, for one file, or on the Mac one file or folder, and answer the path chosen, or blank when the user cancels. On the blocking pool, because the answer waits on the user
#[command]
pub async fn dialog_open(window: WebviewWindow) -> Result<String, String> {
	run_blocking(move || dialog_open_over(&window)).await
}

#[cfg(target_os = "macos")]
fn dialog_open_over(window: &WebviewWindow) -> Result<String, String> {//the panel as a sheet on this window, put up on the main thread and waited for here
	let (sender, receiver) = std::sync::mpsc::channel();
	let asking = window.clone();
	window.run_on_main_thread(move || {
		let Ok(address) = asking.ns_window() else { return };//the NSWindow tauri built, read here on the main thread, where windows close, so it cannot close before the sheet is on it; a window already gone drops the answer
		let parent = unsafe { &*(address as *const objc2_app_kit::NSWindow) };
		mac::dialog_panel(Some(parent), move |path| { let _ = sender.send(path); })
	}).map_err(|e| e.to_string())?;
	receiver.recv().map_err(|_| "the open box never answered".to_string())//a sender dropped unanswered, which only a panel that never went up leaves
}

#[cfg(not(target_os = "macos"))]
fn dialog_open_over(window: &WebviewWindow) -> Result<String, String> {//the dialog plugin's box for a file, which blocks this pool thread while the plugin runs it on the main thread
	use tauri_plugin_dialog::DialogExt;
	let Some(chosen) = window.dialog().file().set_parent(window).blocking_pick_file() else { return Ok(String::new()) };//cancelled, which is an ordinary answer
	let path = chosen.into_path().map_err(|e| e.to_string())?;
	path.into_os_string().into_string().map_err(|path| format!("the path chosen is not text: {path:?}"))//a path that is not utf-8 cannot cross to the page as a string
}

/// Put the Open box up on its own, with no window behind it, for a file or a folder; answer gets the path chosen, or blank, on the main thread. Call from the main thread, which menu.rs does when there is no window to ask from
#[cfg(target_os = "macos")]
pub fn dialog_open_alone(answer: impl FnOnce(String) + 'static) {
	mac::dialog_panel(None, answer)
}

#[cfg(target_os = "macos")]
mod mac {
	use std::cell::Cell;
	use block2::RcBlock;
	use objc2::MainThreadMarker;
	use objc2_app_kit::{NSModalResponse, NSModalResponseOK, NSOpenPanel, NSWindow};

	pub fn dialog_panel(parent: Option<&NSWindow>, answer: impl FnOnce(String) + 'static) {//one file or folder, as a sheet on the parent or as a panel of its own, answered once when the user closes it
		let Some(marker) = MainThreadMarker::new() else { return };//both callers run on the main thread; were one not to, the dropped answer reaches the page as an error rather than as a cancel
		let panel = NSOpenPanel::openPanel(marker);
		panel.setCanChooseFiles(true);
		panel.setCanChooseDirectories(true);
		panel.setAllowsMultipleSelection(false);
		let answer = Cell::new(Some(answer));//appkit takes a block it may call any number of times, and this one answers the first time only
		let shown = panel.clone();//the block reads the choice off the panel, and holds it until appkit lets the block go
		let done = RcBlock::new(move |response: NSModalResponse| {
			let mut path = String::new();//blank for a cancel, and for a panel that could not be shown
			if response == NSModalResponseOK { path = shown.URL().and_then(|url| url.path()).map(|p| p.to_string()).unwrap_or_default() }
			if let Some(answer) = answer.take() { answer(path) }
		});
		match parent {
			Some(window) => panel.beginSheetModalForWindow_completionHandler(window, &done),//slides down from the window's title bar, and holds that window alone
			None => panel.beginWithCompletionHandler(&done),//a window of its own, which is how a Mac application with no document open asks
		}
	}
}
