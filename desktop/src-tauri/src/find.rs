use tauri::command;
use crate::run_blocking;
use steel::find::Found;

/*
Finding things in the folder tree: commands that read many folders in one call. The trip across the bridge is about half a millisecond, which is nothing against one folder and everything against a stretch of a disk with few pictures, thousands of folders read to find the next one that matters; the page paid that trip per folder once, and a call that reads the stretch is sevenfold faster on the same ground, measured on the Mac mini on 2026-10-07. One command today, and the module is where the next way of locating things on the disk goes. Each reads a folder through glance in disk.rs, so the rule for what a folder counts, hidden names and symlinks left out and extensions compared lowercased, is stated once, there, beside the listing it has to agree with. Nothing here knows what a picture, a bucket or a page is: the order is the one find prints, and what is looked for is the list of extensions handed over.
*/

/// The next folders in sorted preorder after or before a folder that hold a file with one of these extensions: up to want of them, reading at most examine folders, with where the walk stopped so a caller can continue
#[command]
pub async fn find_folders(from: String, forward: bool, extensions: Vec<String>, want: u32, examine: u32) -> Result<Found, String> {
	run_blocking(move || Ok(steel::find::find(&from, forward, extensions.into_iter().collect(), want, examine))).await
}

