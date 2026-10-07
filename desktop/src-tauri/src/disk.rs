use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tauri::command;
use crate::run_blocking;

/*
The design contract of this module: these commands hand the interface the full, standard power a desktop application has over the disk — the same power a native Mac or Windows app wields through its file APIs. They follow POSIX semantics faithfully, sharp edges included: disk_copy overwrites an existing destination, just like cp and std::fs::copy do, and disk_write truncates one. Code that calls these commands must be careful, exactly as native application code must.

The commands take any path and hold no guard, on purpose: the page alone knows what a path means and whether writing it is right, and a guard here would be a second copy of that knowledge. So the safety of the whole application rests on walls outside this file, and lib.rs has the long version of what they are.

Two of these commands now change the disk rather than read it. disk_copy writes a file the user asked for; disk_write is how settings and the performance log reach the disk at all, and both of its callers hand it a path they built themselves rather than one that came from the page. That is the current line, and it is thinner than it was when this contract was first written.

Commands here are named for the POSIX call they stand on — disk_readdir, disk_stat, disk_read — so the next ones write themselves: disk_rename over fs::rename, disk_unlink over fs::remove_file, disk_mkdir over fs::create_dir. Each is a line of std::fs and a map_err, which is why none of them is sitting here waiting. Two are exceptions to the naming, and they share one reading of a folder, glance below: disk_peek, a readdir that answers a count and the subfolders' names rather than a record per entry, for a caller that passes through folders it will never show; and disk_walk, which walks the folder tree in sorted preorder from a folder, forward or backward, and answers the next folders that hold a file with one of the extensions asked about. The walk is here because a page that asked for each folder across the bridge paid half a millisecond a folder on the trip alone, and a stretch of a disk with few pictures is thousands of folders; one call walks them all at the cost of the directory reads. It is a general path walker, not a feature: the order is the sorted preorder any find or ls -R gives, hidden names and symlinks left out, and what it looks for is the list of extensions it is handed.

When the delete family arrives, hold a guard here as well: a Rust-side registry of allowed roots, recording folders the user has actually dragged in or chosen, with commands refusing paths outside them. Read and copy trust their caller; unlink should trust less.
*/

#[derive(Serialize)]
pub struct DirEntry {
	pub name:       String,//base name of the entry, without the parent path
	pub is_file:    bool,//true if this entry is a regular file
	pub is_dir:     bool,//true if this entry is a directory
	pub is_symlink: bool,//true if this entry is a symbolic link
	pub size:       u64,//size in bytes; typically 0 for directories and symlinks
	pub mtime:      u128,//last modification time, in milliseconds since the unix epoch; 0 when the filesystem has no answer. From the metadata already read for the kind and the size, so it costs the listing nothing more
}

#[derive(Serialize)]
pub struct Peek {
	pub files:   u32,//how many visible regular files have one of the extensions asked about
	pub folders: Vec<String>,//the names of the visible subfolders, symlinks left out, in whatever order the disk handed them over
}

#[derive(Serialize)]
pub struct Found {
	pub path:  String,//a folder the walk found, with forward slashes as every path the page holds has
	pub files: u32,//how many of its visible regular files have one of the extensions asked about, at least 1
}

#[derive(Serialize)]
pub struct Walk {
	pub found:    Vec<Found>,//the folders found, in walk order
	pub stopped:  String,//the folder the walk stopped at, to continue from in the same direction; blank when done
	pub done:     bool,//the walk reached the end of the volume going forward, or its top going backward
	pub examined: u32,//how many folders it read on the way, the ancestors it had to read to find its place included
	pub refused:  u32,//how many of those could not be read and were passed through as empty
}

#[derive(Serialize)]
pub struct FileStat {
	pub is_file:    bool,//true if this path is a regular file
	pub is_dir:     bool,//true if this path is a directory
	pub is_symlink: bool,//true if this path is a symbolic link
	pub size:       u64,//size in bytes
	pub atime:      u128,//last access time, in milliseconds since the unix epoch; 0 when the filesystem has no answer
	pub mtime:      u128,//last modification time, in milliseconds since the unix epoch; 0 when the filesystem has no answer
	pub birthtime:  u128,//creation time, in milliseconds since the unix epoch; 0 when the filesystem has no answer, which is common on linux. Named for the birth time rather than ctime, which in posix is when the metadata last changed, a different time rust does not offer portably
}

/*
Every command here is an async fn whose body goes to run_blocking in lib.rs, and so is every other command that waits — thumbnail.rs, launch.rs and panel.rs. The body is ordinary blocking code, unchanged; the wrapper decides which threads it runs on.

Until October 2026 these were plain functions marked #[command(async)]. Tauri wraps such a body in an async task and spawns it on tokio's worker pool, one thread per core, but the body never yields: std::fs, ImageIO, WIC and Launch Services each hold their thread until they answer. So every call held a worker for its whole length, the time it spent only waiting on the disk included. That was two concerns. The first was the count: eight workers on the Mac mini and four on a Raspberry Pi, against governors in the page that let up to twelve calls through, so on a small machine the core count rather than the page decided how many ran, and a few reads stuck on a dead share would have held every worker and stalled every async command, ones that had nothing to do with that disk among them. The second was a panic. Tauri replies to the page after the body returns, and a panic unwinds past the reply, so the page's promise never settles — read in tauri 2.11.5's ipc/mod.rs and tauri-macros' wrapper.rs, not seen happening. Under a governor that call holds its place for good, and four of them freeze a line. Two Mac bodies caught their own panics; nothing else did.

run_blocking hands the body to spawn_blocking, which runs it on tokio's blocking pool instead: a thread for each body that waits, up to 512, each retired after ten idle seconds. Awaiting it reports a panic as an error, and run_blocking turns that into the same kind of string every command already answers with. So how many calls run together is the governors' decision alone; a stuck call costs one thread that nothing else is waiting for; and every panic in every command comes back to the page as an error, from one place, which is why the hand-written catch_unwinds are gone.

What it does not do, and what it costs. It is not faster on the machines fuji runs on today: on the Mac mini the governors never reached the core count. It cancels nothing: a read stuck on a dead share still sits in the kernel, now on a thread of its own, and five hundred and twelve of those would fill the blocking pool and bring the old problem back at a larger number — containment rather than a cure, and the deadline security.md proposes is still the answer to a call that never ends. Threads now come and go, so a burst after an idle spell pays to start some, and on Windows the thumbnail initializes and uninitializes COM around every render rather than once per thread, since a thread may not be there for the next one. A panic arrives coarse, as the fact that a command panicked and its message, without where. And each waiting command reads a level deeper, its body inside a closure that owns everything it uses, so a body can never borrow from its call; every argument here is already an owned String, so nothing had to change for that.

Truly asynchronous file reads were the other road, and they are not there to take. tokio::fs runs these same blocking calls on this same blocking pool. The kernels' genuinely asynchronous interfaces, io_uring on linux and overlapped I/O on windows, would be a different program, and ImageIO, WIC and Launch Services have no asynchronous form at all.
*/

/// POSIX-like `readdir`, shallow only
#[command]//every command here waits on the disk, so each is async and runs its body through run_blocking rather than on the thread that runs the window, where a slow folder or a large file would hold up every window event and every other reply until it finished
pub async fn disk_readdir(path: String) -> Result<Vec<DirEntry>, String> {
	run_blocking(move || {
		let mut results = Vec::new();
		for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
			let entry = match entry { Ok(entry) => entry, Err(_) => continue };//skip an entry rather than fail the whole folder over it
			let meta  = match entry.metadata() { Ok(meta) => meta, Err(_) => continue };//same for one we can't stat, like a locked file. The entry's own metadata, which describes a symlink rather than following it, and costs no call at all on windows, where the enumeration already carried the size and times, and one fstatat on the folder's handle elsewhere; symlink_metadata on the path would open every file on windows
			let ft    = meta.file_type();
			results.push(DirEntry {
				name:       entry.file_name().to_string_lossy().into_owned(),
				is_file:    ft.is_file(),
				is_dir:     ft.is_dir(),
				is_symlink: ft.is_symlink(),
				size:       meta.len(),
				mtime:      millis(meta.modified()),
			});
		}
		Ok(results)
	}).await
}

/// A glance at a folder: how many of its files have one of these extensions, and the names of its subfolders, from the directory read alone
//readdir without the stat per entry a listing costs and without a record per entry across the bridge: a cache folder of fifty thousand files answers in one call with a number and a few names, where disk_readdir stats every file and sends fifty thousand records, which the walk in walk.js measured at seconds for one folder it would never show. glance below says what visible means and how an extension is compared
#[command]
pub async fn disk_peek(path: String, extensions: Vec<String>) -> Result<Peek, String> {
	run_blocking(move || {
		let wanted: HashSet<String> = extensions.into_iter().collect();
		let (files, folders) = glance(Path::new(&path), &wanted).map_err(|e| e.to_string())?;
		Ok(Peek { files, folders })
	}).await
}

//one reading of a folder, for the glance and the walk: how many visible regular files have one of the wanted extensions, and the names of the visible subfolders. Visible means what ls shows without -a, names not starting with a dot; a symlink is neither a file nor a folder here, so a walk never follows one into a loop; and an extension is compared lowercased with its dot, like .jpg. library.js applies the same rule to a full listing in _listImages, and the two must agree, since the walk counts a folder's images here and cuts its buckets from there
fn glance(path: &Path, wanted: &HashSet<String>) -> std::io::Result<(u32, Vec<String>)> {
	let mut files = 0u32;
	let mut folders = Vec::new();
	for entry in fs::read_dir(path)? {
		let entry = match entry { Ok(entry) => entry, Err(_) => continue };//skip an entry rather than fail the whole folder over it, as the listing does
		let name = entry.file_name().to_string_lossy().into_owned();
		if name.starts_with('.') { continue }
		let ft = match entry.file_type() { Ok(ft) => ft, Err(_) => continue };//free on both platforms, since the kind comes with the entry itself; only a filesystem that withholds it costs a stat here
		if ft.is_dir() { folders.push(name) }
		else if ft.is_file() {
			let extension = Path::new(&name).extension().map(|e| format!(".{}", e.to_string_lossy().to_lowercase())).unwrap_or_default();//blank for a name with no dot after its first character, which matches nothing
			if wanted.contains(&extension) { files += 1 }
		}
	}
	Ok((files, folders))
}

/*
disk_walk: the folder tree in sorted preorder, walked with a stack, from a folder in either direction.

The order is the one find prints: a folder, then each of its visible subfolders by name, each walked the same way. Forward from a folder means everything after it in that order, starting with its own first subfolder; backward means everything before it, starting with the deepest last descendant of its earlier sibling, or its parent when it has none, since a parent comes before all of its subfolders. The walk finds its place by reading the folder's ancestors from the root down, which costs as many reads as the tree is deep, and from there every folder is read once, which is what the stack is for: a textbook iterator over a tree rather than one that asks a parent again at every step, which the page once did and which cost a folder of nine thousand subfolders nine thousand readings of itself. A folder the walk cannot read counts as empty and is passed through, and the names are compared rather than looked up, so a folder the rule would not list, a dot-folder opened on purpose, still has well defined neighbors.

Two stop rules, one each way, and they differ for a reason. Forward, the walk stops at a folder it has just read, before descending into it, and the page continues after that folder, which begins with the folder's own first subfolder, so nothing is skipped. Backward, the walk stops only at a folder it has finished, its subfolders behind it, because the folder before a folder is the last of that folder's own descendants: stopping at one merely read would skip its whole subtree when the page continued before it. So the backward ceiling is checked when a folder is finished rather than when it is read, and the overshoot is at most the depth of the tree.

The ceiling is what keeps one call honest: a call reads at most that many folders and then answers with where it stopped, so a walk through a quiet region is a series of calls the page can count and a user can watch, rather than one call holding a thread for as long as the region is long, and on a slow disk a call ends in a known time. The page chooses the ceiling, and walk.js says why it picked its number.
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
pub async fn disk_walk(from: String, forward: bool, extensions: Vec<String>, want: u32, examine: u32) -> Result<Walk, String> {
	run_blocking(move || Ok(walk(&from, forward, extensions.into_iter().collect(), want, examine))).await
}

fn walk(from: &str, forward: bool, wanted: HashSet<String>, want: u32, examine: u32) -> Walk {//the command's whole body, a plain function so a test can run it on a tree it made
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

	let mut found: Vec<Found> = Vec::new();
	loop {
		let Some(top) = stack.last_mut() else { return Walk { found, stopped: String::new(), done: true, examined: reader.examined, refused: reader.refused } };//the end of the volume, or its top
		if forward {
			if top.index >= top.kids.len() { stack.pop(); continue }//this folder's subfolders are done; climb
			let path = top.path.join(&top.kids[top.index]); top.index += 1;
			let (files, kids) = reader.read(&path);
			if files > 0 { found.push(Found { path: say(&path), files }) }
			if found.len() as u32 >= want || reader.examined >= examine { return Walk { found, stopped: say(&path), done: false, examined: reader.examined, refused: reader.refused } }//stopped at a folder just read and not entered, which continuing after it enters first
			stack.push(Level { path, files, kids, index: 0 });
		} else if top.index > 0 {
			top.index -= 1;
			let path = top.path.join(&top.kids[top.index]);
			let (files, kids) = reader.read(&path);
			let last = kids.len();
			stack.push(Level { path, files, kids, index: last });//enter at its last subfolder, since backward the subfolders come before the folder
		} else {
			let level = stack.pop().unwrap();//finished: its subfolders are behind, and now the folder itself, whose own files come before them
			if level.files > 0 { found.push(Found { path: say(&level.path), files: level.files }) }
			if found.len() as u32 >= want || reader.examined >= examine { return Walk { found, stopped: say(&level.path), done: stack.is_empty(), examined: reader.examined, refused: reader.refused } }//stopped at a folder finished, which continuing before it leaves behind whole
		}
	}
}

/// POSIX-like `stat(2)` metadata
//one caller in the page, associate.js, which asks only whether a path is there; a listing carries a folder's sizes and modified times without one of these per file
#[command]
pub async fn disk_stat(path: String) -> Result<FileStat, String> {
	run_blocking(move || {
		let meta  = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
		let ft    = meta.file_type();
		Ok(FileStat {
			is_file:    ft.is_file(),
			is_dir:     ft.is_dir(),
			is_symlink: ft.is_symlink(),
			size:       meta.len(),
			atime:      millis(meta.accessed()),
			mtime:      millis(meta.modified()),
			birthtime:  millis(meta.created()),
		})
	}).await
}
fn millis(time: std::io::Result<std::time::SystemTime>) -> u128 {//milliseconds since the unix epoch, or 0 when the filesystem cannot say
	time.ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis()).unwrap_or(0)
}

/// POSIX-like `open` + `read` + `close`
#[command]
pub async fn disk_read(path: String) -> Result<tauri::ipc::Response, String> {
	run_blocking(move || std::fs::read(&path).map(tauri::ipc::Response::new).map_err(|e| e.to_string())).await//Response carries the bytes raw; the essay below has the cost
}
/*
Returning Response rather than Vec<u8> is the difference between a copy and a translation. A Vec<u8> is serialized as a JSON array — one decimal number per byte, written on the Rust side and parsed on the JS side — so a 2.5 MB photograph crosses as roughly two and a half million numbers. Fuji measured that at about 150ms per megabyte on an M2, linear in file size, and it was landing on the main thread in the middle of flips. Response hands the same bytes over as an ArrayBuffer instead. The JS side already wrapped the result in `new Uint8Array(...)`, which accepts either, so nothing above had to change.

Note that this still reads the whole file into memory, and fuji holds it more than once: Rust's buffer, the transfer, and the JS heap.
plugin-fs does streaming by:
- on the Rust side, reading parts of the file in 64 KB chunks
- on the JS side, presenting that using the Web Streams API
so this will be fine for images, but for big files, you'll have to use plugin-fs or implement a fancier read function here of our own!
*/

/// "cp" (shallow, files only)
//also without a caller yet, and correct to be ready: copying is what a backup feature is made of, and the essay below is the research behind doing it well
#[command]
pub async fn disk_copy(source: String, destination: String) -> Result<(), String> {
	run_blocking(move || fs::copy(&source, &destination).map(|_| ()).map_err(|e| e.to_string())).await
}
/*
Bytes in a Tauri application live in three places: the kernel's page cache, the Rust process, and the webview's JS heap. Reading a file drags them through all three — disk, page cache, Rust buffer, the transfer, JS heap — and every one of those is a copy. That is the cost disk_read above pays, and the Response change was about removing the worst step from it rather than the steps themselves.

disk_copy never pays it at all. std::fs::copy reaches CopyFileEx on Windows and fclonefileat or fcopyfile on macOS, and both do the whole thing inside the kernel: no user-mode buffer is allocated in the Tauri process, and no byte of the file enters the Rust or JS heaps. It is as fast as the hardware allows for a file of any size.

What it cannot do is report. It is fire and forget — nothing above can learn that it is halfway done, or pause it, or cancel it. Tauri's plugin-fs answers that with streaming file handles, reading and writing in 64 KiB chunks that become a ReadableStream on the JS side, which buys progress and cancellation at the price of hauling every chunk up through all three layers.

There is a third shape, if fuji ever needs a progress bar on a large copy: keep the copy in the kernel and lift only the counter. CopyFileExW takes a progress callback on Windows, copyfile takes a status callback on macOS, and everywhere else a 64 KiB loop checking an AtomicBool would do. That is roughly two screenfuls of Rust, and it would let the page watch a copy it never touches.
*/

/// POSIX `open` with `O_TRUNC|O_CREAT` + `write` + `close`
#[command]
pub async fn disk_write(path: String, data: Vec<u8>) -> Result<(), String> {
	run_blocking(move || fs::write(&path, data).map_err(|e| e.to_string())).await
}
/*
disk_write truncates and writes in place, on purpose, and is not an atomic save. A process that ends mid-write, a quit landing while a write runs on the blocking pool, can leave the file torn; the usual cure is to write beside the file and rename over it. That is a different operation rather than a safer version of this one — the result is a new file, without the old one's hard links, permissions, Finder tags or Windows access lists, and the rename can be refused where an in-place write would not be — and choosing it, and naming the temporary, are decisions the page owns. So when a caller first needs an atomic save, the plan is disk_rename beside these, one line over fs::rename, and the page writes a temporary of its own naming and renames it over the real file, leaving at worst a stray temporary and never a torn file. Building replacement on each platform's own call, ReplaceFileW on Windows and replaceItemAtURL on the Mac, which keep the old file's attributes, was weighed and set aside as too much platform-specific code for what it adds.
*/
