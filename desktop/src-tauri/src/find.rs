use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tauri::command;
use crate::run_blocking;
use crate::disk::glance;

/*
Finding things in the folder tree: commands that read many folders in one call. The trip across the bridge is about half a millisecond, which is nothing against one folder and everything against a stretch of a disk with few pictures, thousands of folders read to find the next one that matters; the page paid that trip per folder once, and a call that reads the stretch is sevenfold faster on the same ground, measured on the Mac mini on 2026-10-07. One command today, and the module is where the next way of locating things on the disk goes. Each reads a folder through glance in disk.rs, so the rule for what a folder counts, hidden names and symlinks left out and extensions compared lowercased, is stated once, there, beside the listing it has to agree with. Nothing here knows what a picture, a bucket or a page is: the order is the one find prints, and what is looked for is the list of extensions handed over.
*/

#[derive(Serialize)]
pub struct Folder {
	pub path:  String,//a folder found, with forward slashes as every path the page holds has
	pub files: u32,//how many of its visible regular files have one of the extensions asked about, at least 1
}

#[derive(Serialize)]
pub struct Found {
	pub folders:  Vec<Folder>,//the folders found, in the order of the walk
	pub stopped:  String,//the folder the walk stopped at, to continue from in the same direction; blank when done
	pub done:     bool,//the walk reached the end of the volume going forward, or its top going backward
	pub examined: u32,//how many folders it read on the way, the ancestors it had to read to find its place included
	pub refused:  u32,//how many of those could not be read and were passed through as empty
}

/*
find_folders: the folder tree in sorted preorder, walked with a stack, from a folder in either direction.

The order is the one find prints: a folder, then each of its visible subfolders by name, each walked the same way. Forward from a folder means everything after it in that order, starting with its own first subfolder; backward means everything before it, starting with the deepest last descendant of its earlier sibling, or its parent when it has none, since a parent comes before all of its subfolders. The walk finds its place by reading the folder's ancestors from the root down, which costs as many reads as the tree is deep, and from there every folder is read once, which is what the stack is for: a textbook iterator over a tree rather than one that asks a parent again at every step, which the page once did and which cost a folder of nine thousand subfolders nine thousand readings of itself. A folder the walk cannot read counts as empty and is passed through, and the names are compared rather than looked up, so a folder the rule would not list, a dot-folder opened on purpose, still has well defined neighbors.

Two stop rules, one each way, and they differ for a reason. Forward, the walk stops at a folder it has just read, before descending into it, and the page continues after that folder, which begins with the folder's own first subfolder, so nothing is skipped. Backward, the walk stops only at a folder it has finished, its subfolders behind it, because the folder before a folder is the last of that folder's own descendants: stopping at one merely read would skip its whole subtree when the page continued before it. So the backward ceiling is checked when a folder is finished rather than when it is read, and the overshoot is at most the depth of the tree.

The ceiling is what keeps one call honest: a call reads at most that many folders and then answers with where it stopped, so a walk through a quiet region is a series of calls the page can count and a user can watch, rather than one call holding a thread for as long as the region is long, and on a slow disk a call ends in a known time. The page chooses the ceiling, and pager.js says why it picked its number.
*/
struct Level { path: PathBuf, files: u32, kids: Vec<String>, index: usize }//one folder on the stack: its own matching files, its subfolders sorted by name, and which of them the walk is at: going forward the next to enter, going backward one past the next to enter

struct Reader { wanted: HashSet<String>, examined: u32, refused: u32 }//the glance that counts itself, sorts the subfolders, and treats a folder it cannot read as empty
impl Reader {
	fn read(&mut self, path: &Path) -> (u32, Vec<String>) {
		self.examined += 1;
		match glance(path, &self.wanted) {
			Ok((files, mut kids)) => { kids.sort(); (files, kids) }
			Err(_) => { self.refused += 1; (0, Vec::new()) }
		}
	}
}

fn say(path: &Path) -> String { path.to_string_lossy().replace('\\', "/") }//forward slashes, as every path the page holds has; join puts the platform's separator on windows

/// The next folders in sorted preorder after or before a folder that hold a file with one of these extensions: up to want of them, reading at most examine folders, with where the walk stopped so a caller can continue
#[command]
pub async fn find_folders(from: String, forward: bool, extensions: Vec<String>, want: u32, examine: u32) -> Result<Found, String> {
	run_blocking(move || Ok(find(&from, forward, extensions.into_iter().collect(), want, examine))).await
}

fn find(from: &str, forward: bool, wanted: HashSet<String>, want: u32, examine: u32) -> Found {//the command's whole body, a plain function so a test can run it on a tree it made
	let mut reader = Reader { wanted, examined: 0, refused: 0 };
	let from = PathBuf::from(from);

	//the ancestors from the root down to the folder's parent, each at the position of the child that leads toward the folder: past it going forward, at it going backward
	let mut stack: Vec<Level> = Vec::new();
	let mut ancestors: Vec<&Path> = from.ancestors().skip(1).collect(); ancestors.reverse();//the root first
	for (depth, ancestor) in ancestors.iter().enumerate() {
		let child = ancestors.get(depth + 1).copied().unwrap_or(from.as_path());//the one below, which leads toward the folder
		let name = child.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
		let (files, kids) = reader.read(ancestor);
		let at = kids.partition_point(|kid| kid.as_str() < name.as_str());//the first subfolder not before the name, which is the name itself when it is listed
		let index = if forward && at < kids.len() && kids[at] == name { at + 1 } else { at };
		stack.push(Level { path: ancestor.to_path_buf(), files, kids, index });
	}
	if forward { let (files, kids) = reader.read(&from); stack.push(Level { path: from.clone(), files, kids, index: 0 }) }//the folder itself, to enter its subfolders first; backward the folder is behind the walk already

	let mut folders: Vec<Folder> = Vec::new();
	loop {
		let Some(top) = stack.last_mut() else { return Found { folders, stopped: String::new(), done: true, examined: reader.examined, refused: reader.refused } };//the end of the volume, or its top
		if forward {
			if top.index >= top.kids.len() { stack.pop(); continue }//this folder's subfolders are done; climb
			let path = top.path.join(&top.kids[top.index]); top.index += 1;
			let (files, kids) = reader.read(&path);
			if files > 0 { folders.push(Folder { path: say(&path), files }) }
			if folders.len() as u32 >= want || reader.examined >= examine { return Found { folders, stopped: say(&path), done: false, examined: reader.examined, refused: reader.refused } }//stopped at a folder just read and not entered, which continuing after it enters first
			stack.push(Level { path, files, kids, index: 0 });
		} else if top.index > 0 {
			top.index -= 1;
			let path = top.path.join(&top.kids[top.index]);
			let (files, kids) = reader.read(&path);
			let last = kids.len();
			stack.push(Level { path, files, kids, index: last });//enter at its last subfolder, since backward the subfolders come before the folder
		} else {
			let level = stack.pop().unwrap();//finished: its subfolders are behind, and now the folder itself, whose own files come before them
			if level.files > 0 { folders.push(Folder { path: say(&level.path), files: level.files }) }
			if folders.len() as u32 >= want || reader.examined >= examine { return Found { folders, stopped: say(&level.path), done: stack.is_empty(), examined: reader.examined, refused: reader.refused } }//stopped at a folder finished, which continuing before it leaves behind whole
		}
	}
}
