use tauri::{command, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use crate::run_blocking;

//the operating system's own dialog boxes, which only rust can put up. The dialog plugin does the work through its Rust half, which lib.rs registers for this alone; none of the plugin is granted to the page, so these commands are the only way it reaches a dialog

/// Put the system's Open box up over the window asking, for one file of any kind, and answer the path chosen, or blank when the user cancels. On the blocking pool, because the answer waits on the user
#[command]
pub async fn dialog_open(window: WebviewWindow) -> Result<String, String> {
	run_blocking(move || {
		let Some(chosen) = window.dialog().file().set_parent(&window).blocking_pick_file() else { return Ok(String::new()) };//cancelled, which is an ordinary answer
		let path = chosen.into_path().map_err(|e| e.to_string())?;
		path.into_os_string().into_string().map_err(|path| format!("the path chosen is not text: {path:?}"))//a path that is not utf-8 cannot cross to the page as a string
	}).await
}
