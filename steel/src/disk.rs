//the disk: the standard library's calls on files and folders, and fs4's on a volume's space, under steel's rules; brass's disk.rs holds the commands over these, and the contract they keep

use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::UNIX_EPOCH;

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
pub struct FileStat {
	pub is_file:    bool,//true if this path is a regular file
	pub is_dir:     bool,//true if this path is a directory
	pub is_symlink: bool,//true if this path is a symbolic link
	pub size:       u64,//size in bytes
	pub atime:      u128,//last access time, in milliseconds since the unix epoch; 0 when the filesystem has no answer
	pub mtime:      u128,//last modification time, in milliseconds since the unix epoch; 0 when the filesystem has no answer
	pub birthtime:  u128,//creation time, in milliseconds since the unix epoch; 0 when the filesystem has no answer, which is common on linux. Named for the birth time rather than ctime, which in posix is when the metadata last changed, a different time rust does not offer portably
}

#[derive(Serialize)]
pub struct Space {
	pub available:   u64,//bytes this user may write on the volume now, which is the room a copy has: f_bavail blocks on the Mac and Linux, and GetDiskFreeSpaceExW's bytes available to the caller on Windows, which per-user quotas can make smaller. On the Mac this is less than Finder and Disk Utility call available, since they count the purgeable space macOS clears on its own as it needs room, and a copy held to this number never depends on that clearing
	pub free:        u64,//bytes free on the volume, the blocks it holds back for the system included: f_bfree, and the volume's free bytes on Windows
	pub total:       u64,//the volume's size in bytes: f_blocks, and its clusters on Windows
	pub granularity: u64,//the unit the volume allocates a file's space in, so a file takes its size rounded up to it: f_frsize, and the cluster on Windows
}

/// The entries of one folder, shallow, each with its kind, size and modified time from the metadata the listing already read; an entry that cannot be read or stated is left out rather than failing the folder
pub fn readdir(path: &str) -> std::io::Result<Vec<DirEntry>> {
	let mut results = Vec::new();
	for entry in fs::read_dir(path)? {
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
}

/// A glance at a folder: how many of its visible files have one of these extensions, given lowercased with their dots, and the names of its visible subfolders
pub fn peek(path: &str, extensions: Vec<String>) -> std::io::Result<Peek> {
	let wanted: HashSet<String> = extensions.into_iter().collect();
	let (files, folders) = glance(Path::new(path), &wanted)?;
	Ok(Peek { files, folders })
}

//one reading of a folder, for the peek here and for find.rs: how many visible regular files have one of the wanted extensions, and the names of the visible subfolders. Visible means what ls shows without -a, names not starting with a dot; a symlink is neither a file nor a folder here, so a walk over the tree never follows one into a loop; and an extension is compared lowercased with its dot, like .jpg. library.js applies the same rule to a full listing in _listImages, and the two must agree, since the pager counts a folder's images through this and cuts its buckets from that
pub(crate) fn glance(path: &Path, wanted: &HashSet<String>) -> std::io::Result<(u32, Vec<String>)> {
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
			if wanted.contains(&extension) { files = files.saturating_add(1) }
		}
	}
	Ok((files, folders))
}

/// What the file system says of one path without following a link: its kind, size and three times
pub fn stat(path: &str) -> std::io::Result<FileStat> {
	let meta  = fs::symlink_metadata(path)?;
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
}
fn millis(time: std::io::Result<std::time::SystemTime>) -> u128 {//milliseconds since the unix epoch, or 0 when the filesystem cannot say
	time.ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis()).unwrap_or(0)
}

/// POSIX-like `statvfs`: the space on the volume holding a path that is there, as the system reports it, through fs4
pub fn statvfs(path: &str) -> std::io::Result<Space> {
	fs::metadata(path)?;//refused when the path is not there, as statvfs refuses it on the Mac and Linux, since Windows' GetVolumePathNameW ignores a missing end of a path and answers for the volume
	let s = fs4::statvfs(path)?;
	Ok(Space {
		available:   s.available_space(),
		free:        s.free_space(),
		total:       s.total_space(),
		granularity: s.allocation_granularity(),
	})
}

/// The whole file, read into memory
pub fn read(path: &str) -> std::io::Result<Vec<u8>> { fs::read(path) }

/// Text added to the end of a file, made if it is not there
pub fn append(path: &str, text: &str) -> std::io::Result<()> {
	fs::OpenOptions::new().append(true).create(true).open(path).and_then(|mut file| file.write_all(text.as_bytes()))
}

/// One file copied over another, as cp copies: the destination written over if it is there, and nothing of the bytes passing through this process
pub fn copy(source: &str, destination: &str) -> std::io::Result<()> { fs::copy(source, destination).map(|_| ()) }
/*
Bytes in a Tauri application live in three places: the kernel's page cache, the Rust process, and the webview's JS heap. Reading a file drags them through all three — disk, page cache, Rust buffer, the transfer, JS heap — and every one of those is a copy. That is the cost disk_read above pays, and the Response change was about removing the worst step from it rather than the steps themselves.

disk_copy never pays it at all. std::fs::copy reaches CopyFileEx on Windows and fclonefileat or fcopyfile on macOS, and both do the whole thing inside the kernel: no user-mode buffer is allocated in the Tauri process, and no byte of the file enters the Rust or JS heaps. It is as fast as the hardware allows for a file of any size.

What it cannot do is report. It is fire and forget — nothing above can learn that it is halfway done, or pause it, or cancel it. Tauri's plugin-fs answers that with streaming file handles, reading and writing in 64 KiB chunks that become a ReadableStream on the JS side, which buys progress and cancellation at the price of hauling every chunk up through all three layers.

There is a third shape, if fuji ever needs a progress bar on a large copy: keep the copy in the kernel and lift only the counter. CopyFileExW takes a progress callback on Windows, copyfile takes a status callback on macOS, and everywhere else a 64 KiB loop checking an AtomicBool would do. That is roughly two screenfuls of Rust, and it would let the page watch a copy it never touches.
*/

/// The file written whole, truncated first
pub fn write(path: &str, data: &[u8]) -> std::io::Result<()> { fs::write(path, data) }

/// The file's name removed, as rm removes it, and its bytes left for the file system to reuse; a symbolic link removed itself, never what it names
pub fn unlink(path: &str) -> std::io::Result<()> { fs::remove_file(path) }

/// A file or folder given a new name or a new folder on the same volume, written over whatever file already has that name, as mv does
pub fn rename(source: &str, destination: &str) -> std::io::Result<()> { fs::rename(source, destination) }

/// One new folder, in a folder that is there
pub fn mkdir(path: &str) -> std::io::Result<()> { fs::create_dir(path) }

/// One folder removed, which has to be empty
pub fn rmdir(path: &str) -> std::io::Result<()> { fs::remove_dir(path) }
/*
disk_write truncates and writes in place, on purpose, and is not an atomic save. A process that ends mid-write, a quit landing while a write runs on the blocking pool, can leave the file torn; the usual cure is to write beside the file and rename over it. That is a different operation rather than a safer version of this one — the result is a new file, without the old one's hard links, permissions, Finder tags or Windows access lists, and the rename can be refused where an in-place write would not be — and choosing it, and naming the temporary, are decisions the page owns. So when a caller first needs an atomic save, the page writes a temporary of its own naming and renames it over the real file with disk_rename, leaving at worst a stray temporary and never a torn file. Building replacement on each platform's own call, ReplaceFileW on Windows and replaceItemAtURL on the Mac, which keep the old file's attributes, was weighed and set aside as too much platform-specific code for what it adds.
*/

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::arithmetic_side_effects, clippy::panic)]//a test may index, unwrap and add: a mistake in one fails the test, which is what a test is for
mod tests {
	use super::*;
	use crate::testing::Tree;

	#[test]
	fn a_listing_names_every_entry_with_its_kind_and_size() {
		let t = Tree::new("disk-readdir");
		for f in ["b.txt", "a.jpg", ".hidden"] { t.file(f) }
		t.folder("d");
		let mut entries = readdir(&t.text("")).unwrap();
		entries.sort_by(|a, b| a.name.cmp(&b.name));
		let seen: Vec<(String, bool, bool, u64)> = entries.iter().map(|e| (e.name.clone(), e.is_file, e.is_dir, e.size)).collect();
		assert_eq!(seen, vec![(".hidden".into(), true, false, 7), ("a.jpg".into(), true, false, 5), ("b.txt".into(), true, false, 5), ("d".into(), false, true, entries[3].size)]);//a listing shows hidden names, as ls -a does; a file's size is its name's length, and a folder's is whatever the file system says
		let now = millis(Ok(std::time::SystemTime::now()));
		assert!(entries.iter().all(|e| now - 60_000 < e.mtime && e.mtime <= now + 5_000), "every entry was modified this minute: {:?} against {now}", entries.iter().map(|e| e.mtime).collect::<Vec<_>>());
		assert!(readdir(&t.text("missing")).is_err());
	}

	#[test]
	fn a_glance_counts_the_wanted_files_and_names_the_visible_folders() {
		let t = Tree::new("disk-peek");
		for f in ["one.jpg", "two.JPG", "three.jpeg", "four.txt", ".hidden.jpg", "d/inside.jpg", ".hidden-folder/x.jpg"] { t.file(f) }
		t.folder("e");
		#[cfg(unix)]
		std::os::unix::fs::symlink(t.path("d"), t.path("link")).unwrap();//a link to a folder is neither a file nor a folder to the glance, so a walk never follows one into a loop
		let p = peek(&t.text(""), vec![".jpg".into()]).unwrap();
		let mut folders = p.folders.clone(); folders.sort();
		assert_eq!((p.files, folders), (2, vec!["d".to_string(), "e".to_string()]));//one.jpg and two.JPG, compared lowercased; not .jpeg, not the hidden file, and not the files inside d
		assert_eq!(peek(&t.text(""), vec![".jpg".into(), ".jpeg".into(), ".txt".into()]).unwrap().files, 4);
		assert_eq!(peek(&t.text(""), vec![]).unwrap().files, 0);
	}

	#[test]
	fn a_stat_describes_a_path_without_following_a_link() {
		let t = Tree::new("disk-stat");
		t.file("f.txt"); t.folder("d");
		let f = stat(&t.text("f.txt")).unwrap();
		assert_eq!((f.is_file, f.is_dir, f.is_symlink, f.size), (true, false, false, 5));
		let now = millis(Ok(std::time::SystemTime::now()));
		assert!(now - 60_000 < f.mtime && f.mtime <= now + 5_000 && f.atime > 0, "the times the file system keeps: {} {} against {now}", f.mtime, f.atime);
		fs::File::options().write(true).open(t.path("f.txt")).unwrap().set_modified(UNIX_EPOCH - std::time::Duration::from_secs(1)).unwrap();//a time before 1970, which a file from a clock that was never set or an archive written by a careless tool can carry
		assert_eq!(stat(&t.text("f.txt")).unwrap().mtime, 0, "no answer, rather than a number that wrapped");
		let d = stat(&t.text("d")).unwrap();
		assert_eq!((d.is_file, d.is_dir, d.is_symlink), (false, true, false));
		#[cfg(unix)]
		{
			std::os::unix::fs::symlink(t.path("f.txt"), t.path("link")).unwrap();
			let l = stat(&t.text("link")).unwrap();
			assert_eq!((l.is_file, l.is_symlink), (false, true));
		}
		assert!(stat(&t.text("missing")).is_err());
	}

	#[test]
	fn statvfs_answers_for_the_volume_holding_a_path() {
		let t = Tree::new("disk-statvfs");
		t.file("f.txt");
		let s = statvfs(&t.text("")).unwrap();
		assert!(s.total > 0 && s.granularity > 0 && s.available <= s.total && s.free <= s.total, "available {} free {} total {} granularity {}", s.available, s.free, s.total, s.granularity);
		assert_eq!(statvfs(&t.text("f.txt")).unwrap().total, s.total);//a file names its volume as a folder does
		assert!(statvfs(&t.text("missing")).is_err());//on every platform, Windows included
	}

	#[test]
	fn read_write_append_and_copy_move_the_bytes_they_say() {
		let t = Tree::new("disk-bytes");
		write(&t.text("a.bin"), &[1, 2, 3, 255]).unwrap();
		assert_eq!(read(&t.text("a.bin")).unwrap(), vec![1, 2, 3, 255]);
		write(&t.text("a.bin"), &[9]).unwrap();//truncated first, as the essay says
		assert_eq!(read(&t.text("a.bin")).unwrap(), vec![9]);
		append(&t.text("log.txt"), "one\n").unwrap();//made when it is not there
		append(&t.text("log.txt"), "two\n").unwrap();
		assert_eq!(read(&t.text("log.txt")).unwrap(), b"one\ntwo\n");
		copy(&t.text("log.txt"), &t.text("copy.txt")).unwrap();
		assert_eq!(read(&t.text("copy.txt")).unwrap(), b"one\ntwo\n");
		copy(&t.text("a.bin"), &t.text("copy.txt")).unwrap();//written over without asking, as cp does
		assert_eq!(read(&t.text("copy.txt")).unwrap(), vec![9]);
		assert!(read(&t.text("missing")).is_err());
		assert!(copy(&t.text("missing"), &t.text("nowhere")).is_err());
	}
}
