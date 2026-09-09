use serde::Serialize;
use std::fs;
use std::time::UNIX_EPOCH;
use tauri::command;

/*
The design contract of this module: these commands hand the interface the full, standard power a desktop application has over the disk — the same power a native Mac or Windows app wields through its file APIs. They follow POSIX semantics faithfully, sharp edges included: disk_copy overwrites an existing destination, just like cp and std::fs::copy do, and disk_write truncates one. Code that calls these commands must be careful, exactly as native application code must.

The commands take any path and hold no guard, so the safety of the whole application rests on walls outside this file. First, every path originates from a user gesture — a drag onto the window, a choice in a dialog — never from outside content. Second, untrusted text (file names, file contents, metadata) reaches the page only through Vue's escaping interpolation, so it can never become script that calls these commands. Third, the Content-Security-Policy in tauri.conf.json keeps foreign script out of the webview even if a first wall someday cracks.

Two of these commands now change the disk rather than read it. disk_copy writes a file the user asked for; disk_write is how settings and the performance log reach the disk at all, and both of its callers hand it a path they built themselves rather than one that came from the page. That is the current line, and it is thinner than it was when this contract was first written.

Commands here are named for the POSIX call they stand on — disk_readdir, disk_stat, disk_read — so the next ones write themselves: disk_rename over fs::rename, disk_unlink over fs::remove_file, disk_mkdir over fs::create_dir. Each is a line of std::fs and a map_err, which is why none of them is sitting here waiting.

When the delete family arrives, hold a guard here as well: a Rust-side registry of allowed roots, recording folders the user has actually dragged in or chosen, with commands refusing paths outside them. Read and copy trust their caller; unlink should trust less.
*/

#[derive(Serialize)]
pub struct DirEntry {
	pub name:       String,//base name of the entry, without the parent path
	pub is_file:    bool,//true if this entry is a regular file
	pub is_dir:     bool,//true if this entry is a directory
	pub is_symlink: bool,//true if this entry is a symbolic link
	pub size:       u64,//size in bytes; typically 0 for directories and symlinks
}

#[derive(Serialize)]
pub struct FileStat {
	pub is_file:    bool,//true if this path is a regular file
	pub is_dir:     bool,//true if this path is a directory
	pub is_symlink: bool,//true if this path is a symbolic link
	pub size:       u64,//size in bytes
	pub atime:      u128,//last access time, in milliseconds since the unix epoch; 0 when the filesystem has no answer
	pub mtime:      u128,//last modification time, in milliseconds since the unix epoch; 0 when the filesystem has no answer
	pub ctime:      u128,//creation time, in milliseconds since the unix epoch; 0 when the filesystem has no answer, which is common on linux
}

/// POSIX-like `readdir`, shallow only
#[command]
pub fn disk_readdir(path: String) -> Result<Vec<DirEntry>, String> {
	let mut results = Vec::new();
	for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
		let entry = match entry { Ok(entry) => entry, Err(_) => continue };//skip an entry rather than fail the whole folder over it
		let meta  = match fs::symlink_metadata(entry.path()) { Ok(meta) => meta, Err(_) => continue };//same for one we can't stat, like a locked file
		let ft    = meta.file_type();
		results.push(DirEntry {
			name:       entry.file_name().to_string_lossy().into_owned(),
			is_file:    ft.is_file(),
			is_dir:     ft.is_dir(),
			is_symlink: ft.is_symlink(),
			size:       meta.len(),
		});
	}
	Ok(results)
}

/// POSIX-like `stat(2)` metadata
//no caller in the page yet: a date sort needs one of these per file, which sort.md says is the shape the listing will have to grow to carry
#[command]
pub fn disk_stat(path: String) -> Result<FileStat, String> {
	let meta  = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
	let ft    = meta.file_type();
	Ok(FileStat {
		is_file:    ft.is_file(),
		is_dir:     ft.is_dir(),
		is_symlink: ft.is_symlink(),
		size:       meta.len(),
		atime:      millis(meta.accessed()),
		mtime:      millis(meta.modified()),
		ctime:      millis(meta.created()),
	})
}
fn millis(time: std::io::Result<std::time::SystemTime>) -> u128 {//milliseconds since the unix epoch, or 0 when the filesystem cannot say
	time.ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis()).unwrap_or(0)
}

/// POSIX-like `open` + `read` + `close`
#[command]
pub fn disk_read(path: String) -> Result<tauri::ipc::Response, String> {
	std::fs::read(&path).map(tauri::ipc::Response::new).map_err(|e| e.to_string())//Response carries the bytes raw; the essay below has the cost
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
pub fn disk_copy(source: String, destination: String) -> Result<(), String> {
	fs::copy(&source, &destination).map(|_| ()).map_err(|e| e.to_string())
}
/*
Bytes in a Tauri application live in three places: the kernel's page cache, the Rust process, and the webview's JS heap. Reading a file drags them through all three — disk, page cache, Rust buffer, the transfer, JS heap — and every one of those is a copy. That is the cost disk_read above pays, and the Response change was about removing the worst step from it rather than the steps themselves.

disk_copy never pays it at all. std::fs::copy reaches CopyFileEx on Windows and fclonefileat or fcopyfile on macOS, and both do the whole thing inside the kernel: no user-mode buffer is allocated in the Tauri process, and no byte of the file enters the Rust or JS heaps. It is as fast as the hardware allows for a file of any size.

What it cannot do is report. It is fire and forget — nothing above can learn that it is halfway done, or pause it, or cancel it. Tauri's plugin-fs answers that with streaming file handles, reading and writing in 64 KiB chunks that become a ReadableStream on the JS side, which buys progress and cancellation at the price of hauling every chunk up through all three layers.

There is a third shape, if fuji ever needs a progress bar on a large copy: keep the copy in the kernel and lift only the counter. CopyFileExW takes a progress callback on Windows, copyfile takes a status callback on macOS, and everywhere else a 64 KiB loop checking an AtomicBool would do. That is roughly two screenfuls of Rust, and it would let the page watch a copy it never touches.
*/

/// POSIX `open` with `O_TRUNC|O_CREAT` + `write` + `close`
#[command]
pub fn disk_write(path: String, data: Vec<u8>) -> Result<(), String> {
	fs::write(&path, data).map_err(|e| e.to_string())
}
