//the inventory's commands: start one over a folder, read its counts while it runs, and stop it; the loop that walks, reads and writes is copper's, in its inventory.rs

use std::collections::HashMap;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{AppHandle, Manager, State, WebviewWindow};
use copper::inventory::{Inventory, Options, Progress};

/*
An inventory walks a folder's whole tree in one call, the exception to a command taking one thing, for the reason find.rs makes its own: many files take less time to read and hash than a trip across the bridge, so asking per file would spend the run on the asking. The page gives the folder and the file to write, reads the counts with inventory_progress each animation frame while its panel shows, and stops it with inventory_stop. Rust holds an inventory only while it runs, and the window that started it stops it by closing, so none runs on with no page that knows of it.

*/

#[derive(Default)]
pub struct Inventories(Mutex<HashMap<String, (String, Arc<Inventory>)>>);//the inventories under way, by the path of the file each writes: the label of the window that started it, and its counts
fn inventories_lock(inventories: &Inventories) -> MutexGuard<'_, HashMap<String, (String, Arc<Inventory>)>> { inventories.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) }//a map after a panic under its lock is still a map, as media.rs reasons

/// Take an inventory of the tree under folder into the file at output, written over if it is there, a line for every name, as options say; answers the counts once it ends, done or stopped
#[tauri::command]
pub async fn inventory_take(window: WebviewWindow, inventories: State<'_, Inventories>, folder: String, output: String, options: Options) -> Result<Progress, String> {
	let inventory = Arc::new(Inventory::default());
	{
		let mut map = inventories_lock(&inventories);
		if map.contains_key(&output) { return Err(format!("inventory: one is already writing {output}")) }
		map.insert(output.clone(), (window.label().to_string(), inventory.clone()));
	}
	let key = output.clone();
	let body = inventory.clone();
	let result = crate::run_blocking(move || copper::inventory::run(&folder, &output, &options, &body)).await;
	inventories_lock(&inventories).remove(&key);
	result.map(|()| inventory.progress())
}

/// How the inventory writing output is going; nothing once it has ended
#[tauri::command]
pub fn inventory_progress(inventories: State<'_, Inventories>, output: String) -> Option<Progress> {//plain rather than async, so it answers at once on the window's thread, never waiting on the inventory's
	inventories_lock(&inventories).get(&output).map(|(_, inventory)| inventory.progress())
}

/// Ask the inventory writing output to stop, which it does at its next read
#[tauri::command]
pub fn inventory_stop(inventories: State<'_, Inventories>, output: String) {
	if let Some((_, inventory)) = inventories_lock(&inventories).get(&output) { inventory.stop.store(true, Relaxed) }
}

/// The window with this label is closing: stop every inventory it started
pub fn inventory_window_closed(app: &AppHandle, label: &str) {
	let Some(inventories) = app.try_state::<Inventories>() else { return };
	for (owner, inventory) in inventories_lock(&inventories).values() { if owner == label { inventory.stop.store(true, Relaxed) } }
}

