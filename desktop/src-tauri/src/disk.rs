use tauri::command;
use tauri::ipc::Response;
use crate::run_blocking;
use steel::disk::{DirEntry, FileStat, Peek};

/*
The design contract of this module: these commands hand the interface the full, standard power a desktop application has over the disk — the same power a native Mac or Windows app wields through its file APIs. They follow POSIX semantics faithfully, sharp edges included: disk_copy overwrites an existing destination, just like cp and std::fs::copy do, and disk_write truncates one. Code that calls these commands must be careful, exactly as native application code must.

The commands take any path and hold no guard, on purpose: the page alone knows what a path means and whether writing it is right, and a guard here would be a second copy of that knowledge. So the safety of the whole application rests on walls outside this file, and lib.rs has the long version of what they are.

Two of these commands now change the disk rather than read it. disk_copy writes a file the user asked for; disk_write is how settings and the performance log reach the disk at all, and both of its callers hand it a path they built themselves rather than one that came from the page. That is the current line, and it is thinner than it was when this contract was first written.

Commands here are named for the POSIX call they stand on — disk_readdir, disk_stat, disk_read — so the next ones write themselves: disk_rename over fs::rename, disk_unlink over fs::remove_file, disk_mkdir over fs::create_dir. Each is a line of std::fs and a map_err, which is why none of them is sitting here waiting. One is an exception to the naming: disk_peek, a readdir that answers a count and the subfolders' names rather than a record per entry, for a caller that passes through folders it will never show. The reading underneath it, glance, is crate-visible, because find.rs walks the folder tree on the same reading, and the rule for what a folder counts is stated once, here.

When the delete family arrives, hold a guard here as well: a Rust-side registry of allowed roots, recording folders the user has actually dragged in or chosen, with commands refusing paths outside them. Read and copy trust their caller; unlink should trust less.
*/

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
	run_blocking(move || steel::disk::readdir(&path).map_err(|e| e.to_string())).await
}

/// A glance at a folder: how many of its files have one of these extensions, and the names of its subfolders, from the directory read alone
//readdir without the stat per entry a listing costs and without a record per entry across the bridge: a cache folder of fifty thousand files answers in one call with a number and a few names, where disk_readdir stats every file and sends fifty thousand records, which the pager measured at seconds for one folder it would never show. glance below says what visible means and how an extension is compared
#[command]
pub async fn disk_peek(path: String, extensions: Vec<String>) -> Result<Peek, String> {
	run_blocking(move || steel::disk::peek(&path, extensions).map_err(|e| e.to_string())).await
}

/// POSIX-like `stat(2)` metadata
//the page asks it whether a path is there, and whether one handed to fuji is a folder; a listing carries a folder's sizes and modified times without one of these per file
#[command]
pub async fn disk_stat(path: String) -> Result<FileStat, String> {
	run_blocking(move || steel::disk::stat(&path).map_err(|e| e.to_string())).await
}

/// POSIX-like `open` + `read` + `close`
#[command]
pub async fn disk_read(path: String) -> Result<Response, String> {
	run_blocking(move || steel::disk::read(&path).map(Response::new).map_err(|e| e.to_string())).await//Response carries the bytes raw; the essay below has the cost
}
/*
Returning Response rather than Vec<u8> is the difference between a copy and a translation. A Vec<u8> is serialized as a JSON array — one decimal number per byte, written on the Rust side and parsed on the JS side — so a 2.5 MB photograph crosses as roughly two and a half million numbers. Fuji measured that at about 150ms per megabyte on an M2, linear in file size, and it was landing on the main thread in the middle of flips. Response hands the same bytes over as an ArrayBuffer instead. The JS side already wrapped the result in `new Uint8Array(...)`, which accepts either, so nothing above had to change.

Note that this still reads the whole file into memory, and fuji holds it more than once: Rust's buffer, the transfer, and the JS heap.
plugin-fs does streaming by:
- on the Rust side, reading parts of the file in 64 KB chunks
- on the JS side, presenting that using the Web Streams API
so this will be fine for images, but for big files, you'll have to use plugin-fs or implement a fancier read function here of our own!
*/

/// POSIX `open` with `O_APPEND|O_CREAT` + `write` + `close`
#[command]
pub async fn disk_append(path: String, text: String) -> Result<(), String> {
	run_blocking(move || steel::disk::append(&path, &text).map_err(|e| e.to_string())).await
}

/// "cp" (shallow, files only)
//also without a caller yet, and correct to be ready: copying is what a backup feature is made of, and the essay beside steel's copy is the research behind doing it well
#[command]
pub async fn disk_copy(source: String, destination: String) -> Result<(), String> {
	run_blocking(move || steel::disk::copy(&source, &destination).map_err(|e| e.to_string())).await
}

/// POSIX `open` with `O_TRUNC|O_CREAT` + `write` + `close`
#[command]
pub async fn disk_write(path: String, data: Vec<u8>) -> Result<(), String> {
	run_blocking(move || steel::disk::write(&path, &data).map_err(|e| e.to_string())).await
}
