//finding the next folders that hold a file of a kind: the folder tree in find's sorted order, walked with a stack from a folder in either direction, many folders per call; brass's find.rs holds the command over it, and says why one call reads many

use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::disk::glance;

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
		self.examined = self.examined.saturating_add(1);
		match glance(path, &self.wanted) {
			Ok((files, mut kids)) => { kids.sort(); (files, kids) }
			Err(_) => { self.refused = self.refused.saturating_add(1); (0, Vec::new()) }
		}
	}
}

fn say(path: &Path) -> String { path.to_string_lossy().replace('\\', "/") }//forward slashes, as every path the page holds has; join puts the platform's separator on windows

pub fn find(from: &str, forward: bool, wanted: HashSet<String>, want: u32, examine: u32) -> Found {//the command's whole body, a plain function so a test can run it on a tree it made
	let mut reader = Reader { wanted, examined: 0, refused: 0 };
	let from = PathBuf::from(from);

	//the ancestors from the root down to the folder's parent, each at the position of the child that leads toward the folder: past it going forward, at it going backward
	let mut stack: Vec<Level> = Vec::new();
	let mut ancestors: Vec<&Path> = from.ancestors().skip(1).collect(); ancestors.reverse();//the root first
	for (depth, ancestor) in ancestors.iter().enumerate() {
		let child = ancestors.get(depth.saturating_add(1)).copied().unwrap_or(from.as_path());//the one below, which leads toward the folder
		let name = child.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
		let (files, kids) = reader.read(ancestor);
		let at = kids.partition_point(|kid| kid.as_str() < name.as_str());//the first subfolder not before the name, which is the name itself when it is listed
		let index = if forward && kids.get(at) == Some(&name) { at.saturating_add(1) } else { at };
		stack.push(Level { path: ancestor.to_path_buf(), files, kids, index });
	}
	if forward { let (files, kids) = reader.read(&from); stack.push(Level { path: from.clone(), files, kids, index: 0 }) }//the folder itself, to enter its subfolders first; backward the folder is behind the walk already

	let mut folders: Vec<Folder> = Vec::new();
	loop {
		let Some(top) = stack.last_mut() else { return Found { folders, stopped: String::new(), done: true, examined: reader.examined, refused: reader.refused } };//the end of the volume, or its top
		if forward {
			if top.index >= top.kids.len() { stack.pop(); continue }//this folder's subfolders are done; climb
			let path = top.path.join(top.kids.get(top.index).cloned().unwrap_or_default()); top.index = top.index.saturating_add(1);//the index is within kids by the check above, and the empty name a miss would join is never reached
			let (files, kids) = reader.read(&path);
			if files > 0 { folders.push(Folder { path: say(&path), files }) }
			if folders.len() as u32 >= want || reader.examined >= examine { return Found { folders, stopped: say(&path), done: false, examined: reader.examined, refused: reader.refused } }//stopped at a folder just read and not entered, which continuing after it enters first
			stack.push(Level { path, files, kids, index: 0 });
		} else if top.index > 0 {
			top.index = top.index.saturating_sub(1);
			let path = top.path.join(top.kids.get(top.index).cloned().unwrap_or_default());
			let (files, kids) = reader.read(&path);
			let last = kids.len();
			stack.push(Level { path, files, kids, index: last });//enter at its last subfolder, since backward the subfolders come before the folder
		} else {
			let Some(level) = stack.pop() else { continue };//finished: its subfolders are behind, and now the folder itself, whose own files come before them; the stack is never empty here, and the loop's first line answers if it were
			if level.files > 0 { folders.push(Folder { path: say(&level.path), files: level.files }) }
			if folders.len() as u32 >= want || reader.examined >= examine { return Found { folders, stopped: say(&level.path), done: stack.is_empty(), examined: reader.examined, refused: reader.refused } }//stopped at a folder finished, which continuing before it leaves behind whole
		}
	}
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::arithmetic_side_effects, clippy::panic)]//a test may index, unwrap and add: a mistake in one fails the test, which is what a test is for
mod tests {
	use super::*;
	use crate::testing::Tree;

	fn tree(name: &str) -> Tree {//five folders in find's order, a, a/aa, b, b/bb, c, three of them holding pictures: a one, b/bb one, c two; b holds only text and a/aa nothing
		let t = Tree::new(name);
		for f in ["a/x.jpg", "b/y.txt", "b/bb/z.jpg", "c/w.jpg", "c/v.jpg", "c/u.txt"] { t.file(f) }
		t.folder("a/aa");
		t
	}
	fn wanted() -> HashSet<String> { [".jpg".to_string()].into_iter().collect() }
	fn found(t: &Tree, f: &Found) -> Vec<(String, u32)> { f.folders.iter().map(|folder| (t.relative(Path::new(&folder.path)), folder.files)).collect() }
	fn row(path: &str, files: u32) -> (String, u32) { (path.to_string(), files) }

	#[test]
	fn forward_from_a_folder_finds_the_folders_after_it_in_sorted_preorder() {//want stops the walk at the last folder of the tree, since forward from the tree's root the walk would go on into the rest of the temporary folder and the volume after it
		let t = tree("find-forward");
		let f = find(&t.text(""), true, wanted(), 3, 50);
		assert_eq!(found(&t, &f), vec![row("a", 1), row("b/bb", 1), row("c", 2)]);//a before its subfolder aa, which holds nothing; b holds no picture and is not listed, its subfolder is; c last with two
		assert_eq!((t.relative(Path::new(&f.stopped)), f.done), ("c".to_string(), false));//stopped at the folder just read and not entered, so continuing after it enters c first
		assert!(f.examined >= 6 && f.refused == 0, "the ancestors above the tree are read too, to find the start's place: {}", f.examined);
		let more = find(&t.text("a"), true, wanted(), 2, 50);//continuing from a: a's own subfolder first, then on
		assert_eq!(found(&t, &more), vec![row("b/bb", 1), row("c", 2)]);
	}

	#[test]
	fn backward_from_a_folder_finds_the_folders_before_it_with_a_parent_before_its_subfolders() {
		let t = tree("find-backward");
		let f = find(&t.text("c"), false, wanted(), 2, 50);
		assert_eq!(found(&t, &f), vec![row("b/bb", 1), row("a", 1)]);//the deepest last descendant of the earlier sibling first, b/bb, then b itself with nothing, then a/aa with nothing, then a
		assert_eq!((t.relative(Path::new(&f.stopped)), f.done), ("a".to_string(), false));//stopped at a folder finished, whose subfolders are behind it
		let one = find(&t.text("b/bb"), false, wanted(), 1, 50);
		assert_eq!(found(&t, &one), vec![row("a", 1)]);//before b/bb comes its parent b, with nothing, then a's subtree from its end
	}

	#[test]
	fn the_ceiling_ends_a_call_with_where_it_stopped() {
		let t = tree("find-ceiling");
		let f = find(&t.text("a"), true, wanted(), 10, 1);//one folder read is the ceiling, and the ancestors already cost more than that
		assert!(f.folders.len() <= 1 && !f.done && !f.stopped.is_empty() && f.examined >= 1, "{:?} {} {}", f.folders.len(), f.stopped, f.examined);
	}

	#[test]
	fn a_folder_that_cannot_be_read_is_passed_through_as_empty_and_counted() {
		let t = tree("find-missing");
		let f = find(&t.text("nowhere/at/all"), true, wanted(), 10, 50);//a start that is not there: its ancestors are read as far as they exist, the missing ones refused, and the walk goes on from where they would have been
		assert!(f.refused >= 1 && f.examined >= f.refused, "{} {}", f.refused, f.examined);
	}
}
