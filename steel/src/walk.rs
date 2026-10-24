//the walker: every name in a folder tree with what kind of thing it is, which is steel's knowledge of storage the way tip.rs is its knowledge of formats

use serde::Serialize;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::PathBuf;

/*
The walker: every name in a folder tree, each with what kind of thing it is, in the same order on every run. The reader in tip.rs knows the formats of the files a collection holds; this file knows the storage they sit on, which is as varied and as old: three operating systems, a dozen file systems, drives from thirty years of computers, and the cloud services that now appear as folders. Each of those has its own way to send a naive walk somewhere it should not go, and the account of each sits on the line that guards against it, below, so the knowledge and the code that acts on it are read, and corrected, together.

The rule. The walker looks at everything that is there, and avoids only what would interrupt the walk, what is not really there, and what is somewhere else. Hidden names are there. They signal that they are not for the user, but they hold bytes on the drive and are safe to look at, so the walker lists and enters them, and a listing without them would misreport what a drive holds. That is a different rule from the one the contact sheet's find.rs follows, which leaves dot names out of what the sheet shows, and each keeps its own. What the walker avoids falls in three groups: a name that points somewhere else, which could send a walk off the drive or around a loop forever; a file whose contents are not on the disk, which a read would fetch, changing the disk; and an entry that is not a file at all, which an open could wait on forever.

The answer. Every name the walk meets comes back as an Item with its kind, the avoided ones included, so a walk accounts for every name it met, and a caller opens and reads an item only when its kind is File. The walker itself never opens a file. It lists folders and reads each name's metadata, the one stat a listing already makes, which never follows a link, and decides everything from the name and that stat.

What it cannot prevent. Every listing and every stat is a call into the system, and on a dying disk, a scratched disc retrying a sector, a network share that dropped, a USB drive waking from sleep or a permission prompt nobody answers, that call can wait for minutes or forever. The walker has no answer to that, since a thread waiting in the kernel cannot be cancelled: a caller checks for a stop between items, and the lasting answer is a process of its own, which can be killed, as the overnight scan will have.
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Kind {//what a name in the tree is, and so what a caller may do with it: read a File, and only list every other kind. Each is decided where its check is commented below
	File,//a regular file, which a caller may open and read
	Folder,//a folder the walker entered, whose entries follow it
	Link,//a symbolic link, or on Windows a junction or a volume's mount point: listed, never followed
	Pipe,//a named pipe, whose open waits for a writer
	Socket,//a socket, which no open reaches
	Device,//a block or character device, which can read without end
	Cloud,//a file or folder whose contents are on a cloud service's server, which a read would fetch
	Reparse,//on Windows, a reparse point that is neither a link nor marked as in the cloud, with a filter behind it the walker cannot see
	Unreachable,//on Windows, a name its ordinary paths reach something else by: a device's name, or one ending in a dot or a space
	Mount,//a folder where another file system is mounted, not entered, so a walk stays on the drive it began on
	Seen,//a folder already walked, reached again without a link, not entered again
	Refused,//a folder or an entry the system would not list or describe, with its words in error
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {//one name the walk met
	pub path: PathBuf,//the start joined with the names below it, kept as the system's own rather than as text, since Linux allows a name that is not valid UTF-8 and Windows one with an unpaired surrogate, and old collections hold both; so a caller can open what the walker found, and loses only those characters when it makes text of the path
	pub kind: Kind,
	pub size: u64,//bytes, for anything the stat calls a regular file, whatever kind the walker gives it, so a Cloud file comes with the size its service reports and a Reparse point with its own; 0 for everything else
	pub error: String,//the system's words, for Refused; blank for every other kind
}

/// Walk the tree under a folder: every name in it as an Item, beginning with the folder itself, as Folder, or as Refused when it is missing, is not a folder or will not list. A start with forward slashes, as every path fuji holds has, works on Windows too, which takes either
pub fn walk(start: impl Into<PathBuf>) -> Walk {
	Walk { start: Some(start.into()), stack: Vec::new(), device: None, seen: HashSet::new() }
}

pub struct Walk {//the walk under way, read one item at a time as an iterator; a caller stops by dropping it
	start: Option<PathBuf>,//the folder the walk begins at, until the first item enters it
	stack: Vec<Level>,//the folders entered and not yet finished, the deepest last: a stack of the walker's own rather than a call per folder, so a tree as deep as a drive allows cannot exhaust the thread's stack
	device: Option<u64>,//the device the walk began on, on the Mac and Linux; none on Windows, where identity says why none is needed
	seen: HashSet<(u64, u64)>,//the device and inode of every folder entered, on the Mac and Linux; empty on Windows
}
struct Level { leaves: std::vec::IntoIter<Item>, folders: std::vec::IntoIter<Pending> }//one folder's listing: its entries that are not folders, then its folders, and the walk's place in each
struct Pending { item: Item, identity: Option<(u64, u64)> }//a folder in a listing, not yet reached: its item, and its device and inode where the platform has them

impl Iterator for Walk {
	type Item = Item;
	fn next(&mut self) -> Option<Item> {
		if let Some(start) = self.start.take() { return Some(self.begin(start)) }
		loop {
			let level = self.stack.last_mut()?;//no folder left unfinished: the walk is done
			if let Some(item) = level.leaves.next() { return Some(item) }
			if let Some(pending) = level.folders.next() { return Some(self.arrive(pending)) }
			self.stack.pop();
		}
	}
}

impl Walk {
	fn begin(&mut self, start: PathBuf) -> Item {//the folder the walk was asked to begin at: its device is the walk's, and it is the first folder seen
		let meta = match fs::metadata(&start) { Ok(meta) => meta, Err(e) => return refused(start, &e) };//followed, unlike every stat below, since the user chose this folder by name, a link to it included, and the walk is of where it leads
		if !meta.is_dir() { return Item { path: start, kind: Kind::Refused, size: 0, error: "not a folder".into() } }
		if cloud(&meta) { return Item { path: start, kind: Kind::Cloud, size: 0, error: String::new() } }//chosen or not, a folder whose listing would be fetched is not entered
		if let Some(id) = identity(&meta) { self.device = Some(id.0); self.seen.insert(id); }
		self.enter(start)
	}

	fn arrive(&mut self, pending: Pending) -> Item {//a folder from a listing, reached in its turn: entered when it is on the walk's drive and not walked already, and otherwise answered as what it is
		let Pending { item, identity } = pending;
		if item.kind != Kind::Folder { return item }//a folder the listing already named as Cloud, Reparse or Unreachable, which is never entered

		//another drive: a walk stays on the device it began on, as find -xdev does. On the Mac and Linux a folder on another device is where another file system is mounted: a second volume, a network share, a phone or a share that Linux's gvfs mounts under /run/user, any FUSE file system, a cloud drive mounted as one, a btrfs subvolume, a device of its own to this test, which is how Fedora and openSUSE keep /home and openSUSE its snapshots, and Linux's /proc and /sys, whose files can block a read. The Mac's boot drive is one device to this test, though two volumes underneath: the sealed system volume and the data volume beside it report the same device, since the system joins them into the one tree the user sees, so a walk of the boot drive crosses between them as the user does, while Preboot, VM and the rest under /System/Volumes are devices of their own
		if let (Some((device, _)), Some(start)) = (identity, self.device) { if device != start { return Item { kind: Kind::Mount, ..item } } }

		//a folder reached twice with no link to say so: Time Machine on HFS+, the Mac's backups until Big Sur moved new ones to APFS, made directory hard links, so every backup on such a drive shares the folders unchanged since the one before; a Linux bind mount shows one folder at a second place, possibly inside itself; the Mac's firmlinks show its data volume's folders at a second name, /Users and /System/Volumes/Data/Users; and the Mac's /dev/fd lists every file the process reading it has open, the folder it is listing included. The first arrival is entered and every later one is Seen, so an old backup drive is walked once and none of these can loop
		if let Some(id) = identity { if !self.seen.insert(id) { return Item { kind: Kind::Seen, ..item } } }
		self.enter(item.path)
	}

	fn enter(&mut self, folder: PathBuf) -> Item {//list a folder, keep its entries on the stack to be walked, and answer the folder, or answer it refused when it will not list
		let listing = match fs::read_dir(&folder) { Ok(listing) => listing, Err(e) => return refused(folder, &e) };//some folders say no, and the walk goes on past them: System Volume Information at the root of every Windows drive and the other users' folders in $RECYCLE.BIN; the places Apple protects, where Desktop, Documents, Downloads, removable drives and network volumes ask the user once, and Mail, Messages and Safari's data answer "Operation not permitted" without Full Disk Access; lost+found at the root of a Linux drive
		let (mut leaves, mut folders) = (Vec::new(), Vec::new());
		for entry in listing {//every entry, hidden or not, since the rule looks at everything there: a dot name, a Windows file with the hidden or system attribute, a Mac file with the hidden flag
			let entry = match entry { Ok(entry) => entry, Err(e) => { leaves.push(refused(folder.clone(), &e)); break } };//an entry the listing could not hand over, which has no name, so the folder stands in for it, and the listing ends there: a listing that has failed once is not asked again, and the walk goes on with the names it had
			let path = entry.path();
			let meta = match entry.metadata() { Ok(meta) => meta, Err(e) => { leaves.push(refused(path, &e)); continue } };//the one stat, which never follows a link, and on Windows comes from the listing itself with no call of its own. A file in use, like the pagefile.sys at the root of a Windows system drive, describes itself here, since nothing opens it, and refuses its reader instead
			let kind = classify(&entry.file_name(), &meta);
			let error = if kind == Kind::Refused { "not a file, a folder, a link, a pipe, a socket or a device".into() } else { String::new() };
			let item = Item { path, kind, size: if meta.is_file() { meta.len() } else { 0 }, error };
			if meta.is_dir() { folders.push(Pending { item, identity: identity(&meta) }) } else { leaves.push(item) }
		}
		leaves.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));//a folder's own files together, ahead of everything below it, and each group by name compared by bytes, as find.rs sorts, so the order is the same on every run whatever order the disk hands the names over in
		folders.sort_by(|a, b| a.item.path.file_name().cmp(&b.item.path.file_name()));
		self.stack.push(Level { leaves: leaves.into_iter(), folders: folders.into_iter() });
		Item { path: folder, kind: Kind::Folder, size: 0, error: String::new() }
	}
}

fn refused(path: PathBuf, error: &io::Error) -> Item { Item { path, kind: Kind::Refused, size: 0, error: error.to_string() } }

fn classify(name: &OsStr, meta: &fs::Metadata) -> Kind {//what an entry is, from its name and its stat, in the order the checks overlap: a link before anything it names, a cloud mark before the reparse point that carries it
	let t = meta.file_type();
	if t.is_symlink() { return Kind::Link }//a symbolic link on the Mac or Linux can name a file, a folder, nothing at all, or a folder above it, which is a loop: the Mac's /Volumes holds one back to the boot volume. On Windows rust calls a reparse point a link when its tag has the name-surrogate bit, which symbolic links, junctions and volume mount points all carry, so this one test keeps a walk out of all three. Every user profile holds junctions for programs written before Vista, My Documents, Application Data and C:\Documents and Settings among them, and AppData\Local\Application Data points back at AppData\Local
	if unreachable(name) { return Kind::Unreachable }//on Windows only, where name_unreachable says which names
	if cloud(meta) { return Kind::Cloud }//each platform's cloud below says how it marks one
	if reparse(meta) { return Kind::Reparse }//on Windows only, and after the two tests above, so only the reparse points neither of them names reach it
	if t.is_dir() { return Kind::Folder }//a folder to reach in its turn, where arrive may still find it on another drive or walked already
	if t.is_file() { return Kind::File }//a regular file, and with it two things that only point or repeat. A Windows shortcut and a Mac alias keep their pointer in a file of their own, which no file system follows, so a reader reads the pointer, not what it names. A hard link is one file under two names, on NTFS, APFS, HFS+ and ext4 alike, and nothing marks either, so the walk lists the file at both, and a hash of the contents is what tells a caller the two are one
	special(&t)
}

#[cfg(unix)]
fn identity(meta: &fs::Metadata) -> Option<(u64, u64)> { use std::os::unix::fs::MetadataExt; Some((meta.dev(), meta.ino())) }//a folder's device and inode, which name it however many places it appears
#[cfg(not(unix))]
fn identity(_: &fs::Metadata) -> Option<(u64, u64)> { None }//none on Windows, where none is needed: NTFS has no directory hard links, a junction is a link, and a drive mounted inside a folder is a volume mount point, a link too, so the test for links keeps a walk on its drive and out of every folder twice

#[cfg(unix)]
fn special(t: &fs::FileType) -> Kind {//what is left on the Mac and Linux: a named pipe makes an open wait for a writer, a socket refuses every open, and a block or character device can read without end. They live mostly in hidden folders and in /dev, which a restored backup of a Linux system can carry onto a data drive
	use std::os::unix::fs::FileTypeExt;
	if t.is_fifo() { Kind::Pipe } else if t.is_socket() { Kind::Socket } else if t.is_block_device() || t.is_char_device() { Kind::Device } else { Kind::Refused }
}
#[cfg(not(unix))]
fn special(_: &fs::FileType) -> Kind { Kind::Refused }//Windows has none the standard library tells apart: its pipes are not in the file system, and the subsystem for Linux's are reparse points

#[cfg(target_os = "macos")]
const SF_DATALESS: u32 = 0x4000_0000;//by its name in sys/stat.h
#[cfg(target_os = "macos")]
fn cloud(meta: &fs::Metadata) -> bool { use std::os::macos::fs::MetadataExt; meta.st_flags() & SF_DATALESS != 0 }//iCloud Drive, Dropbox, Google Drive and OneDrive can all show the Mac a file whose contents are still on a server, through Apple's File Provider, which marks such a file or folder dataless. Reading one makes the system fetch it, which changes the disk, can take minutes, and waits without end when the machine is offline. Before Sonoma, iCloud left a small .icloud stub in a file's place instead, which is an ordinary file and safe to read

#[cfg(windows)]
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;//Windows' attributes, by their names in winnt.h
#[cfg(windows)]
const FILE_ATTRIBUTE_OFFLINE: u32 = 0x0000_1000;
#[cfg(windows)]
const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x0004_0000;
#[cfg(windows)]
const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;
#[cfg(windows)]
fn cloud(meta: &fs::Metadata) -> bool { use std::os::windows::fs::MetadataExt; meta.file_attributes() & (FILE_ATTRIBUTE_OFFLINE | FILE_ATTRIBUTE_RECALL_ON_OPEN | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS) != 0 }//the Cloud Files API, OneDrive's and the other services', marks a placeholder whose open fetches it, as a cloud folder not yet listed is, or whose read does, as a cloud file not yet downloaded is, with the two recall attributes; and the older offline attribute marks a file a storage system has moved away
#[cfg(windows)]
fn reparse(meta: &fs::Metadata) -> bool { use std::os::windows::fs::MetadataExt; meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 }//a reparse point that is neither a link nor in the cloud has a filter behind it the walker cannot see: the subsystem for Linux's pipes, sockets and devices, which rust takes for regular files; the app execution aliases in WindowsApps, which refuse a read; Windows' compressed system files and Windows Server's deduplicated ones, which read normally; and a OneDrive file that has finished downloading, which stays a placeholder with its recall marks cleared. Keeping a reader from all of them costs the readable ones, a downloaded OneDrive file among them, and keeps the rest from doing whatever their filter does
#[cfg(windows)]
fn unreachable(name: &OsStr) -> bool { name_unreachable(&name.to_string_lossy()) }

#[cfg(not(any(target_os = "macos", windows)))]
fn cloud(_: &fs::Metadata) -> bool { false }//Linux keeps no general mark, and its cloud drives are FUSE mounts, which the test for another drive keeps a walk out of
#[cfg(not(windows))]
fn reparse(_: &fs::Metadata) -> bool { false }
#[cfg(not(windows))]
fn unreachable(_: &OsStr) -> bool { false }//the Mac and Linux open every name as it is

#[cfg_attr(not(windows), allow(dead_code))]//only Windows asks, and the tests ask everywhere
fn name_unreachable(name: &str) -> bool {//whether Windows' ordinary paths reach something other than the file by this name. Windows reserves con, prn, aux and nul, and com and lpt with a digit, as the names of devices, with any extension and with spaces before it, so a con.jpg or a com1.txt copied onto a shared drive from a Mac opens the console or a serial port, which can block, rather than the file. And Windows strips a dot or a space from the end of a name, so a file the Mac named "photo." is out of reach of its own path. Windows' verbatim paths, written after \\?\, reach both, and fuji writes none. Pure, so tested on every platform, though only Windows asks
	if name.ends_with('.') || name.ends_with(' ') { return true }
	let stem = name.split_once('.').map_or(name, |(stem, _)| stem).trim_end_matches(' ').to_uppercase();
	if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL") { return true }
	let numbered = stem.strip_prefix("COM").or_else(|| stem.strip_prefix("LPT"));//com and lpt, then one digit, the superscript ones Windows also reserves among them
	matches!(numbered, Some("0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"))
}

//the tests: each builds the hazards it can in a temporary folder of its own and walks it. The order and the hidden names run on every platform; links, a link loop, a dangling link, a start that is a link, a pipe, a socket and a refused folder on the Mac and Linux, and a device in their own /dev; a folder reached again through the Mac's own /dev/fd; and junctions and a junction loop on Windows, where pnpm test runs them. The rest cannot be built without mounting a drive, signing into a cloud service or being root, and is checked by hand: Mount with a second drive, Seen on a Time Machine disk from before Big Sur, Cloud with iCloud Drive on the Mac and OneDrive on Windows, and Unreachable with a file copied onto a shared drive from a Mac

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::arithmetic_side_effects, clippy::panic)]//a test may index, unwrap and add: a mistake in one fails the test, which is what a test is for
mod tests {
	use super::*;
	use std::path::Path;

	struct Tree(PathBuf);//a folder of one test's own under the system's temporary folder, made empty and removed when the test ends, so tests run side by side without meeting
	impl Tree {
		fn new(name: &str) -> Tree {
			let path = std::env::temp_dir().join(format!("steel-walk-{name}-{}", std::process::id()));
			let _ = fs::remove_dir_all(&path);
			fs::create_dir_all(&path).unwrap();
			Tree(path)
		}
		fn path(&self, relative: &str) -> PathBuf { self.0.join(relative) }
		fn file(&self, relative: &str) {//a small file whose contents are its own name, so its size is the name's length
			let path = self.path(relative);
			fs::create_dir_all(path.parent().unwrap()).unwrap();
			fs::write(path, relative).unwrap();
		}
		fn folder(&self, relative: &str) { fs::create_dir_all(self.path(relative)).unwrap() }
		fn listing(&self) -> Vec<(String, Kind)> { walk(&self.0).map(|item| (self.relative(&item.path), item.kind)).collect() }//every item as its path below the tree, with forward slashes, and its kind
		fn relative(&self, path: &Path) -> String { path.strip_prefix(&self.0).unwrap().to_string_lossy().replace('\\', "/") }
	}
	impl Drop for Tree { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
	fn row(path: &str, kind: Kind) -> (String, Kind) { (path.to_string(), kind) }

	#[test]
	fn folders_come_first_then_their_files_then_their_folders_by_name_hidden_ones_included() {
		let t = Tree::new("order");
		for f in ["b.txt", "a.txt", ".DS_Store", "z/c.txt", ".hidden/d.txt", "Q.txt"] { t.file(f) }
		t.folder("m");
		assert_eq!(t.listing(), vec![
			row("", Kind::Folder),
			row(".DS_Store", Kind::File),
			row("Q.txt", Kind::File),//by bytes, so capitals before lowercase
			row("a.txt", Kind::File),
			row("b.txt", Kind::File),
			row(".hidden", Kind::Folder),//entered, hidden or not
			row(".hidden/d.txt", Kind::File),
			row("m", Kind::Folder),//empty
			row("z", Kind::Folder),
			row("z/c.txt", Kind::File),
		]);
		let a = walk(&t.0).find(|item| item.path.ends_with("a.txt")).unwrap();
		assert_eq!((a.size, a.error.as_str()), (5, ""));
	}

	#[test]
	fn a_start_that_is_missing_or_not_a_folder_is_refused() {
		let t = Tree::new("start");
		t.file("a.txt");
		let missing: Vec<Item> = walk(t.path("nowhere")).collect();
		assert_eq!((missing.len(), missing[0].kind, missing[0].error.is_empty()), (1, Kind::Refused, false));
		let file: Vec<Item> = walk(t.path("a.txt")).collect();
		assert_eq!((file.len(), file[0].kind, file[0].error.as_str()), (1, Kind::Refused, "not a folder"));
	}

	#[test]
	fn windows_reaches_something_else_by_device_names_and_a_trailing_dot_or_space() {
		for name in ["con", "CON", "con.jpg", "aux.txt", "nul.tar.gz", "prn", "com1", "COM9.mp3", "lpt3", "com¹.txt", "LPT³", "con .txt", "photo.", "photo "] {
			assert!(name_unreachable(name), "{name} should be unreachable");
		}
		for name in ["console.jpg", "com10", "com", "lpt", "aux1.txt", "nullify", "con-tour.png", ".con", "photo.jpg", "a.b.c"] {
			assert!(!name_unreachable(name), "{name} should be reachable");
		}
	}

	#[cfg(unix)]
	fn link(t: &Tree, name: &str, target: &str) { std::os::unix::fs::symlink(target, t.path(name)).unwrap() }//a symbolic link at a name, to a target written as given, relative to the link's own folder when relative
	#[cfg(unix)]
	fn pipe(t: &Tree, name: &str) { assert!(std::process::Command::new("mkfifo").arg(t.path(name)).status().unwrap().success()) }//through the system's program, since the standard library has no call to make one and the crate forbids unsafe code, tests included

	#[cfg(unix)]
	#[test]
	fn a_link_is_listed_and_never_followed() {
		let t = Tree::new("links");
		t.file("real/inside.txt");
		link(&t, "to-file", "real/inside.txt");
		link(&t, "to-folder", "real");
		link(&t, "dangling", "nowhere");
		link(&t, "loop", ".");//its own folder, which a walk that followed links would enter forever
		link(&t, "real/up", "..");
		assert_eq!(t.listing(), vec![
			row("", Kind::Folder),
			row("dangling", Kind::Link),
			row("loop", Kind::Link),
			row("to-file", Kind::Link),
			row("to-folder", Kind::Link),
			row("real", Kind::Folder),//walked once, where it is
			row("real/inside.txt", Kind::File),
			row("real/up", Kind::Link),
		]);
		assert!(walk(&t.0).filter(|item| item.kind == Kind::Link).all(|item| item.size == 0));
		let chosen: Vec<(String, Kind)> = walk(t.path("to-folder")).map(|item| (t.relative(&item.path), item.kind)).collect();//a link the user chose as the start is followed, and the walk is of where it leads
		assert_eq!(chosen, vec![row("to-folder", Kind::Folder), row("to-folder/inside.txt", Kind::File), row("to-folder/up", Kind::Link)]);
	}

	#[cfg(unix)]
	#[test]
	fn a_device_is_listed_and_never_opened() {//the system's own /dev, on every Mac and Linux: each node is answered from its stat alone, where a walk that opened one, /dev/zero say, would read forever
		let items: Vec<Item> = walk("/dev").collect();
		assert_eq!(items[0].kind, Kind::Folder);
		assert!(items.iter().any(|item| item.kind == Kind::Device), "{items:?}");
	}

	#[cfg(unix)]
	#[test]
	fn a_pipe_and_a_socket_are_listed_and_never_opened() {//a walk that opened the pipe would wait here for a writer that never comes
		let t = Tree::new("specials");
		pipe(&t, "pipe");
		let _socket = std::os::unix::net::UnixListener::bind(t.path("socket")).unwrap();
		t.file("plain.txt");
		assert_eq!(t.listing(), vec![row("", Kind::Folder), row("pipe", Kind::Pipe), row("plain.txt", Kind::File), row("socket", Kind::Socket)]);
	}

	#[cfg(unix)]
	#[test]
	fn a_folder_that_refuses_is_listed_with_its_words_and_the_walk_goes_on() {
		use std::os::unix::fs::PermissionsExt;
		let t = Tree::new("refused");
		t.file("locked/secret.txt");
		t.file("open/plain.txt");
		fs::set_permissions(t.path("locked"), fs::Permissions::from_mode(0o000)).unwrap();
		let readable = fs::read_dir(t.path("locked")).is_ok();//root reads it anyway, which leaves nothing to test
		let items: Vec<Item> = walk(&t.0).collect();
		fs::set_permissions(t.path("locked"), fs::Permissions::from_mode(0o755)).unwrap();//so the tree can be removed
		if readable { return }
		let rows: Vec<(String, Kind)> = items.iter().map(|item| (t.relative(&item.path), item.kind)).collect();
		assert_eq!(rows, vec![row("", Kind::Folder), row("locked", Kind::Refused), row("open", Kind::Folder), row("open/plain.txt", Kind::File)]);
		assert!(!items[1].error.is_empty());
	}

	#[cfg(unix)]
	#[test]
	fn every_hazard_together() {//the tests above in one tree, nested, so the order holds across them all
		use std::os::unix::fs::PermissionsExt;
		let t = Tree::new("together");
		for f in [".DS_Store", "a.txt", "deep/.hidden/b.txt", "deep/c.txt", "locked/secret.txt"] { t.file(f) }
		link(&t, "deep/back", "..");
		link(&t, "deep/.hidden/gone", "missing");
		pipe(&t, "deep/pipe");
		let _socket = std::os::unix::net::UnixListener::bind(t.path("deep/.hidden/socket")).unwrap();
		fs::set_permissions(t.path("locked"), fs::Permissions::from_mode(0o000)).unwrap();
		let readable = fs::read_dir(t.path("locked")).is_ok();//root reads it anyway, as above
		let rows = t.listing();
		fs::set_permissions(t.path("locked"), fs::Permissions::from_mode(0o755)).unwrap();
		if readable { return }
		assert_eq!(rows, vec![
			row("", Kind::Folder),
			row(".DS_Store", Kind::File),
			row("a.txt", Kind::File),
			row("deep", Kind::Folder),
			row("deep/back", Kind::Link),
			row("deep/c.txt", Kind::File),
			row("deep/pipe", Kind::Pipe),
			row("deep/.hidden", Kind::Folder),
			row("deep/.hidden/b.txt", Kind::File),
			row("deep/.hidden/gone", Kind::Link),
			row("deep/.hidden/socket", Kind::Socket),
			row("locked", Kind::Refused),
		]);
	}

	#[cfg(target_os = "macos")]
	#[test]
	fn a_folder_reached_again_is_seen_and_not_entered() {//the Mac's /dev/fd lists the files this process has open, the folder the walker is listing among them, which a walk without the guard would enter forever. Other tests' open folders show up too, on another device, and come back as Mount
		let items: Vec<Item> = walk("/dev/fd").collect();
		assert_eq!(items[0].kind, Kind::Folder);
		assert!(items.iter().any(|item| item.kind == Kind::Seen), "{items:?}");
	}

	#[cfg(windows)]
	fn junction(t: &Tree, name: &str, target: &Path) {//through mklink, which makes a junction without an administrator, where a symbolic link needs one
		assert!(std::process::Command::new("cmd").arg("/C").arg("mklink").arg("/J").arg(t.path(name)).arg(target).status().unwrap().success());
	}

	#[cfg(windows)]
	#[test]
	fn a_junction_is_listed_and_never_followed() {
		let t = Tree::new("junctions");
		t.file("real/inside.txt");
		junction(&t, "to-folder", &t.path("real"));
		junction(&t, "loop", &t.0);//its own folder, the shape of AppData\Local\Application Data
		assert_eq!(t.listing(), vec![
			row("", Kind::Folder),
			row("loop", Kind::Link),
			row("to-folder", Kind::Link),
			row("real", Kind::Folder),
			row("real/inside.txt", Kind::File),
		]);
	}
}
