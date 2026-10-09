use tauri::command;
use crate::run_blocking;

//other programs: this one hands them a file or an address, the way a double-click would, and lets them go

/// Open this file, folder or address with the program the system has for it, the way a double-click in the file manager would, and let that program go: an https address in the browser, an ms-settings address in Windows Settings, a folder in Finder or Explorer. Answered once the system has taken it, with no wait on the program, and on the blocking pool because taking it can mean starting a browser
#[command]
pub async fn process_open(target: String) -> Result<(), String> {
	run_blocking(move || open::that_detached(&target).map_err(|e| format!("could not open {target}: {e}"))).await
}
