//./src-tauri/src/io.rs

use serde::Serialize;
use std::fs;
//use std::path::Path;
use std::time::UNIX_EPOCH;
use tauri::command;

#[derive(Serialize)]
pub struct DirEntry {
	pub name:       String,//Base name of the entry (not including parent path)
	pub is_file:    bool,//True if this entry is a regular file
	pub is_dir:     bool,//True if this entry is a directory
	pub is_symlink: bool,//True if this entry is a symbolic link
	pub size:       u64,//Size in bytes (for files; typically 0 for dirs and symlinks)
}

#[derive(Serialize)]
pub struct FileStat {
	pub is_file:    bool,//True if this path is a regular file
	pub is_dir:     bool,//True if this path is a directory
	pub is_symlink: bool,//True if this path is a symbolic link
	pub size:       u64,//Size in bytes
	pub atime:      u128,//Last access time, in milliseconds since the UNIX epoch
	pub mtime:      u128,//Last modification time, in milliseconds since the UNIX epoch
	pub ctime:      u128,//Creation time, in milliseconds since the UNIX epoch
}

/// POSIX-like `readdir`, shallow only
#[command]
pub fn io_readdir(path: String) -> Result<Vec<DirEntry>, String> {
	let mut results = Vec::new();
	for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
		let entry = entry.map_err(|e| e.to_string())?;
		let meta  = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
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
#[command]
pub fn io_stat(path: String) -> Result<FileStat, String> {
	let meta  = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
	let ft    = meta.file_type();
	let atime = meta
		.accessed()
		.map_err(|e| e.to_string())?
		.duration_since(UNIX_EPOCH)
		.map_err(|e| e.to_string())?
		.as_millis();
	let mtime = meta
		.modified()
		.map_err(|e| e.to_string())?
		.duration_since(UNIX_EPOCH)
		.map_err(|e| e.to_string())?
		.as_millis();
	let ctime = meta
		.created()
		.map_err(|e| e.to_string())?
		.duration_since(UNIX_EPOCH)
		.map_err(|e| e.to_string())?
		.as_millis();
	Ok(FileStat {
		is_file:    ft.is_file(),
		is_dir:     ft.is_dir(),
		is_symlink: ft.is_symlink(),
		size:       meta.len(),
		atime,
		mtime,
		ctime,
	})
}

/// POSIX-like `open` + `read` + `close`
#[command]
pub fn io_read(path: String) -> Result<Vec<u8>, String> {
	std::fs::read(&path).map_err(|e| e.to_string())
}
/*
note that this reads the whole file into memory
fuji will have the file in memory three times: Rust + IPC + JS!
plugin-fs does streaming by:
- on the Rust side, reading parts of the file in 64 KB chunks
- on the JS side, presenting that using the Web Streams API
so this will be fine for images, but for big files, you'll have to use plugin-fs or implement a fancier read function here of our own!
*/

/// “cp” (shallow, files only)  
#[tauri::command]
pub fn io_copy(source: String, destination: String) -> Result<(), String> {
	fs::copy(&source, &destination).map(|_| ()).map_err(|e| e.to_string())
}
/*
a block of memory in a Tauri app like Fuji can exist in these layers:
2. WebView renderer process (JS heap, JSON RPC strings)
1. Rust backend process (fallback 8 KiB buffer, IPC deserialization)
0. Kernel mode (page cache, zero‐copy)

when io_read above gets the bytes of an image onto the screen, the memory is copied many times:
disk -> kernel page cache -> Rust heap buffer -> Rust JSON buffer -> WebView IPC buffer -> JS heap

io_copy, on the other hand is far more efficient
On Windows 10 and later, io_copy (via std::fs::copy) invokes the Win32 CopyFileEx API,
which performs the entire copy inside the kernel’s page cache
no user-mode buffer is ever allocated in your Tauri process 
On modern macOS, it uses the kernel’s fclonefileat/fcopyfile primitives
to clone or copy the file data entirely in kernel space,
so again no bytes of the file ever enter your Rust or JS heaps

io_copy is as fast and efficient as the hardware allows for files of any size
but is a fire and forget pattern--there's no way for an app above to know that it's halfway done
or pause or cancel its operation

Tauri's plugin-fs offers copyFile, which works the same as our io_copy above
to enable progress and cancel copying a huge file, a developer using plugin-fs would use the streaming apis for read and write
When you use plugin-fs's file handle methods (open(), file.read(), file.write(),
the Rust plugin reads or writes in fixed-size chunks (by default 64 KiB at a time) via std::fs or tokio::fs 
Those chunks are pushed into a Tauri IPC “response” whose body is a streaming iterator
On the JS side that becomes a standard ReadableStream<Uint8Array>, so your frontend can pull each 64 KiB chunk,
report progress, and even cancel by aborting the stream.

each streamed chunk makes a round trip through all three layers!

so what if we wanted to design a copy api which would report progress and allow pause and cancel
but did not need to see the bytes (just count them) as they moved across, far below?
we could write rust code which:
- On Windows we call CopyFileExW with a progress callback—no user-mode buffers.
- On macOS we use copyfile + a status callback—again all in kernel mode.
- Everywhere else we fall back to a 64 KiB async read/write loop, checking a shared AtomicBool for cancellation, and emitting progress after each chunk.
which comes to about two screenfuls of Rust
and could result in js code on top that's as simple as this:

import { copyWithProgress } from 'fuji-io';
let controller; // will hold the AbortController
async function runCopy() {
  controller = new AbortController();
  const source      = '/path/to/large-file.bin';
  const destination = '/path/to/destination.bin';
  try {
    await copyWithProgress(
      source,
      destination,
      // onProgress: called with total bytes copied so far
      copiedBytes => {
        console.log(`Copied ${copiedBytes} bytes`);
      },
      controller.signal, // pass the signal so we can abort
    );
    console.log('✅ Copy complete!');
  } catch (err) {
    if (err.name === 'AbortError') {
      console.log('⏸ Copy cancelled by user');
    } else {
      console.error('❌ Copy failed:', err);
    }
  }
}

for compare, the rust function will have to bring blocks of both files into its memory,
the OS doesn't have an api like copy-on-clone
*/

/*
more to add later...

/// POSIX `open` with `O_TRUNC|O_CREAT` + `write` + `close`  
#[tauri::command]
pub fn io_write(path: String, data: Vec<u8>) -> Result<(), String> {
	fs::write(&path, data).map_err(|e| e.to_string())
}

/// POSIX `rename(2)`  
#[tauri::command]
pub fn io_rename(source: String, destination: String) -> Result<(), String> {
	fs::rename(&source, &destination).map_err(|e| e.to_string())
}

/// POSIX `unlink(2)` for files  
#[tauri::command]
pub fn io_unlink(path: String) -> Result<(), String> {
	fs::remove_file(&path).map_err(|e| e.to_string())
}

/// POSIX `rmdir(2)` (fails if non-empty)  
#[tauri::command]
pub fn io_rmdir(path: String) -> Result<(), String> {
	fs::remove_dir(&path).map_err(|e| e.to_string())
}

/// POSIX `mkdir(2)` (single level only)  
#[tauri::command]
pub fn io_mkdir(path: String) -> Result<(), String> {
	fs::create_dir(&path).map_err(|e| e.to_string())
}
*/
