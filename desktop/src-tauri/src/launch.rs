//the launch commands: which application opens a kind of file on the mac, and making fuji the one that does, each a line over copper on the blocking pool

use tauri::command;
use crate::run_blocking;
use copper::launch::Opener;

/// Which application the Mac would open a file type with, the type written with its dot, like .png
#[command]
pub async fn launch_opens(extension: String) -> Result<Opener, String> {
	run_blocking(move || copper::launch::opens(&extension)).await
}

/// Make the running application the one that opens a file type, and every type sharing its kind
#[command]
pub async fn launch_set(extension: String) -> Result<(), String> {
	run_blocking(move || copper::launch::set(&extension)).await
}
