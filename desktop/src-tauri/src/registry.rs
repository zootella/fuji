//the registry commands: five lines over copper, which reads and writes the windows registry for the page

use tauri::command;
use copper::registry::Opener;

/// Read a string value under the current user; answers blank when the key or the value is not there
#[command]
pub fn registry_get(key: String, name: String) -> Result<String, String> {
	copper::registry::get(&key, &name)
}

/// Write a string value under the current user, creating the key if it is missing, and only if it would change; answers whether it changed
#[command]
pub fn registry_set(key: String, name: String, value: String) -> Result<bool, String> {
	copper::registry::set(&key, &name, &value)
}

/// Delete a value under the current user, a blank name being the key's default value; answers whether there was one to delete
#[command]
pub fn registry_delete(key: String, name: String) -> Result<bool, String> {
	copper::registry::delete(&key, &name)
}

/// Tell the shell that file associations changed
#[command]
pub fn registry_notify() -> Result<(), String> {
	copper::registry::notify()
}

/// Which program Windows would open a file type with, the type written with its dot, like .png
#[command]
pub fn registry_opens(extension: String) -> Result<Opener, String> {
	copper::registry::opens(&extension)
}
