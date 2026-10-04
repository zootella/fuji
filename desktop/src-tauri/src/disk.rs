use serde::Serialize;
use std::fs;
use std::time::UNIX_EPOCH;
use tauri::command;
use crate::run_blocking;

/*
The design contract of this module: these commands hand the interface the full, standard power a desktop application has over the disk — the same power a native Mac or Windows app wields through its file APIs. They follow POSIX semantics faithfully, sharp edges included: disk_copy overwrites an existing destination, just like cp and std::fs::copy do, and disk_write truncates one. Code that calls these commands must be careful, exactly as native application code must.

The commands take any path and hold no guard, on purpose: the page alone knows what a path means and whether writing it is right, and a guard here would be a second copy of that knowledge. So the safety of the whole application rests on walls outside this file, and lib.rs has the long version of what they are.

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
	}).await
}

/// POSIX-like `stat(2)` metadata
//no caller in the page yet: a date sort needs one of these per file, which is the shape the listing will have to grow to carry
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
			ctime:      millis(meta.created()),
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
