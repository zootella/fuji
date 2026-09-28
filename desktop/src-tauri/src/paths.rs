use tauri::command;

/*
Where the program is: the one fact about this copy that only Rust can learn, since the page has no way to ask where its own executable sits. The page asks through paths_executable and decides everything that follows from the answer; associate.js is the caller today, and it compares the folder with the one the installer recorded to know it is running from an installed copy rather than from target/.

Fuji is always installed, so this is the whole of it. A program meant to run from a stick would also need the folder a user thinks of as the program's, which on macOS is the one holding the .app rather than the one holding the executable, as the anchor for paths in its settings; fuji has neither the stick nor settings that name paths relative to itself.
*/

/// The path of the running program file; the page asks once, since it never changes while the program runs
#[command]
pub fn paths_executable() -> Result<String, String> {
	std::env::current_exe().map(|path| path.to_string_lossy().into_owned()).map_err(|e| format!("paths: could not find the program's own path, {e}"))//a path that is not utf-8 comes through with replacement characters, and then matches nothing, which is the right answer for it
}
