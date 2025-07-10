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
here's our copy command, also great for small files
no file contents come into memory, this is us telling the system to do the copy
but, there's no way to see progress or cancel partway through
plugin-fs only does copies one way: by connecting read streams and write streams
this way you can see progress and cancel partway through
but for small files, this may be faster!
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
