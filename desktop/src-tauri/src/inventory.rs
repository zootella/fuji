//an inventory: a folder's whole tree, every file in it hashed and scouted, a line each in a text file

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{AppHandle, Manager, State, WebviewWindow};
use ::scout::walk::{Item, Kind};
use zootella_ergo::{has_tick, hash_file, size, tick_to_text, Blake3, Sha1, Sha256};
use crate::scout::{open_file, tip_read};

/*
An inventory walks a folder's whole tree in one call, the exception to a command taking one thing, for the reason find.rs makes its own: many files take less time to read and hash than a trip across the bridge, so asking per file would spend the run on the asking. The page gives the folder and the file to write, reads the counts with inventory_progress each animation frame while its panel shows, and stops it with inventory_stop. Rust holds an inventory only while it runs, and the window that started it stops it by closing, so none runs on with no page that knows of it.

What it trusts: every read is made in fuji's own process, so a read the kernel never finishes, on a dropped share or a dying disk, holds the inventory for good, and only quitting fuji ends it. A process of its own, which can be killed, is the answer to that. No file's bytes can steer the code, since the scout is safe Rust and the hashes interpret nothing.
*/

#[derive(Default)]
pub struct Inventories(Mutex<HashMap<String, Arc<Inventory>>>);//the inventories under way, by the path of the file each writes
fn inventories_lock(inventories: &Inventories) -> MutexGuard<'_, HashMap<String, Arc<Inventory>>> { inventories.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) }//a map after a panic under its lock is still a map, as media.rs reasons

#[derive(Default)]
struct Inventory {//one inventory's counts, written by its thread and read from the window's
	window:  String,//the label of the window that started it
	folders: AtomicU64,//entered, the one it began at among them
	files:   AtomicU64,//every name the walker said was a file, read or not
	bytes:   AtomicU64,//the sizes of the files done, and with hashes checked, how far into the one being read
	size:    AtomicU64,//of the file being read now
	read:    AtomicU64,//how far into it
	path:    Mutex<String>,
	stop:    AtomicBool,
}
impl Inventory {
	fn progress(&self) -> Progress {
		Progress {
			folders: self.folders.load(Relaxed),
			files:   self.files.load(Relaxed),
			bytes:   self.bytes.load(Relaxed),
			size:    self.size.load(Relaxed),
			read:    self.read.load(Relaxed),
			path:    self.path.lock().map(|path| path.clone()).unwrap_or_default(),
			stopped: self.stop.load(Relaxed),
		}
	}
}

#[derive(Deserialize, Clone, Copy)]
pub struct Options {//what the user checked on the panel: which whole-file hashes to compute, which take a read of every byte, and whether the JSON is indented across lines
	sha1:   bool,
	sha256: bool,
	blake3: bool,
	indent: bool,
}
impl Options { fn whole(&self) -> bool { self.sha1 || self.sha256 || self.blake3 } }

#[derive(Serialize)]
pub struct Progress {
	folders: u64,
	files:   u64,
	bytes:   u64,
	size:    u64,
	read:    u64,
	path:    String,
	stopped: bool,
}

/// Take an inventory of the tree under folder into the file at output, written over if it is there, a line for every name, as options say; answers the counts once it ends, done or stopped
#[tauri::command]
pub async fn inventory_take(window: WebviewWindow, inventories: State<'_, Inventories>, folder: String, output: String, options: Options) -> Result<Progress, String> {
	let inventory = Arc::new(Inventory { window: window.label().to_string(), ..Default::default() });
	{
		let mut map = inventories_lock(&inventories);
		if map.contains_key(&output) { return Err(format!("inventory: one is already writing {output}")) }
		map.insert(output.clone(), inventory.clone());
	}
	let key = output.clone();
	let body = inventory.clone();
	let result = crate::run_blocking(move || inventory_run(&folder, &output, options, &body)).await;
	inventories_lock(&inventories).remove(&key);
	result.map(|()| inventory.progress())
}

/// How the inventory writing output is going; nothing once it has ended
#[tauri::command]
pub fn inventory_progress(inventories: State<'_, Inventories>, output: String) -> Option<Progress> {//plain rather than async, so it answers at once on the window's thread, never waiting on the inventory's
	inventories_lock(&inventories).get(&output).map(|inventory| inventory.progress())
}

/// Ask the inventory writing output to stop, which it does at its next read
#[tauri::command]
pub fn inventory_stop(inventories: State<'_, Inventories>, output: String) {
	if let Some(inventory) = inventories_lock(&inventories).get(&output) { inventory.stop.store(true, Relaxed) }
}

/// The window with this label is closing: stop every inventory it started
pub fn inventory_window_closed(app: &AppHandle, label: &str) {
	let Some(inventories) = app.try_state::<Inventories>() else { return };
	for inventory in inventories_lock(&inventories).values() { if inventory.window == label { inventory.stop.store(true, Relaxed) } }
}

const HASH_BUFFER: usize = size::MB as usize;//a read at a time, so the progress within a large file moves every few milliseconds
const THREAD_PIECE: usize = 64 * size::KB as usize;//the least a piece takes to be worth three threads, which cost about 28 µs to start on the Mac mini's M2, more than hashing anything smaller

fn inventory_run(folder: &str, output: &str, options: Options, inventory: &Inventory) -> Result<(), String> {
	let mut out = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(output).map_err(|e| format!("inventory: {output}: {e}"))?;
	let mut buffer = vec![0u8; HASH_BUFFER];//sized by fuji, never by a file
	for item in ::scout::walk::walk(folder) {
		if inventory.stop.load(Relaxed) { break }
		if item.kind == Kind::Folder { inventory.folders.fetch_add(1, Relaxed); continue }
		if item.path == Path::new(output) { continue }//its own output, when the folder holds it
		let line = if item.kind == Kind::File {
			inventory.files.fetch_add(1, Relaxed);
			let read = file_read(&item, inventory, options, &mut buffer);
			if options.whole() && inventory.stop.load(Relaxed) { break }//stopped partway through hashing a file, whose hash of part of it gets no line
			match read {
				Ok((fuji, size, said)) => line_say(&fuji, &item.path, size, &json_say(&said, options.indent)),
				Err(e) => line_say("-", &item.path, item.size, &json_say(&serde_json::json!({"error": e.to_string()}), options.indent)),
			}
		} else {
			let mut said = serde_json::json!({"kind": item.kind});//a name the walker never opens, and why
			if !item.error.is_empty() { said["error"] = serde_json::json!(item.error) }
			line_say("-", &item.path, item.size, &json_say(&said, options.indent))
		};
		out.write_all(line.as_bytes()).map_err(|e| format!("inventory: {output}: {e}"))?;//unbuffered, so the file is complete up to the file being read
	}
	Ok(())
}

fn file_read(item: &Item, inventory: &Inventory, options: Options, buffer: &mut [u8]) -> std::io::Result<(String, u64, serde_json::Value)> {//one file: its fuji tip hash, its size, and its tip, stat and whichever whole-file hashes options ask for
	let mut file = open_file(&item.path)?;
	let meta = file.metadata()?;
	let stat = stat_said(&file, &meta);//before any byte is read, so the access time is the file's own
	let size = meta.len();
	if let Ok(mut path) = inventory.path.lock() { *path = path_say(&item.path) }
	inventory.size.store(size, Relaxed);
	inventory.read.store(0, Relaxed);
	let tip = tip_read(&mut file, size)?;
	let mut said = serde_json::json!({"tip": tip_said(&tip), "stat": stat});
	if options.whole() { said["hash"] = whole_hash(&mut file, size, options, inventory, buffer)? } else { inventory.bytes.fetch_add(size, Relaxed); }
	let tip_hash = match size { 0 => "-".into(), _ => hash_file(&mut file, size)?.tip_hash.map_or_else(|| "-".into(), |hash| hash.base32()) };//ergo's start, middle and end stripes, which a full read just left in the cache; an empty file has none
	Ok((tip_hash, size, said))
}
fn whole_hash(file: &mut std::fs::File, size: u64, options: Options, inventory: &Inventory, buffer: &mut [u8]) -> std::io::Result<serde_json::Value> {//the hashes options ask for, from one read of every byte; a stop ends it early
	file.seek(SeekFrom::Start(0))?;
	let (mut sha1, mut sha256, mut blake3) = (options.sha1.then(Sha1::new), options.sha256.then(Sha256::new), options.blake3.then(Blake3::new));
	let mut hashed = 0u64;
	while !inventory.stop.load(Relaxed) {
		let n = match file.read(buffer) {
			Ok(0) => break,
			Ok(n) => n,
			Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,//a signal interrupted the read; read again
			Err(e) => return Err(e),
		};
		let piece: &[u8] = &buffer[..n];
		let hashers: Vec<&mut (dyn Write + Send)> = [sha1.as_mut().map(|h| h as &mut (dyn Write + Send)), sha256.as_mut().map(|h| h as _), blake3.as_mut().map(|h| h as _)].into_iter().flatten().collect();
		let threaded = n >= THREAD_PIECE && hashers.len() > 1;//a core each, twice as fast at a full buffer on the Mac mini's M2
		std::thread::scope(|s| {
			for hasher in hashers { if threaded { s.spawn(move || { let _ = hasher.write_all(piece); }); } else { let _ = hasher.write_all(piece); } }
		});
		hashed += n as u64;
		inventory.read.store(hashed, Relaxed);
		inventory.bytes.fetch_add(n as u64, Relaxed);
	}
	if hashed != size && !inventory.stop.load(Relaxed) { return Err(std::io::Error::other(format!("the file was {size} bytes when opened and {hashed} when read"))) }//changed while read, so no hash of it holds
	let mut said = serde_json::Map::new();
	if let Some(hasher) = sha1 { said.insert("sha1".into(), hasher.finalize().base16().into()); }
	if let Some(hasher) = sha256 { said.insert("sha256".into(), hasher.finalize().base32().into()); }
	if let Some(hasher) = blake3 { said.insert("blake3".into(), hasher.finalize().base32().into()); }
	Ok(serde_json::Value::Object(said))
}

fn line_say(hash: &str, path: &Path, size: u64, json: &str) -> String {//the path as a JSON string, so a quote or a newline in a name stays inside its quotes
	format!("{hash} {} {size} {json}\n", serde_json::Value::from(path_say(path)))
}
fn tip_said(tip: &::scout::tip::Tip) -> serde_json::Value {//the scout's answer without its reading, which millions of entries have no room for
	let mut said = serde_json::to_value(tip).unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
	if let Some(fields) = said.as_object_mut() { fields.remove("reading"); }
	said
}
fn stat_said(file: &std::fs::File, meta: &std::fs::Metadata) -> serde_json::Value {//each part left out where the platform or the file system has none
	let mut said = serde_json::Map::new();
	let mut time = |name: &str, tick: Option<i64>| { if let Some(tick) = tick.filter(|tick| *tick > 0 && has_tick(*tick)) { said.insert(name.into(), tick_to_text(tick).into()); } };//left out at or before 1970, where it can only be a placeholder or a mistake, and past ergo's range, where the text would panic
	time("created", tick(meta.created()));//made here, or carried by a copy that keeps times
	time("modified", tick(meta.modified()));//the contents, a time any program can set
	time("changed", changed(file, meta));//anything about the file, a time only the system stamps
	time("accessed", tick(meta.accessed()));
	let names = links(file, meta);
	if names > 1 { said.insert("links".into(), names.into()); }
	if hidden(meta) { said.insert("hidden".into(), true.into()); }
	serde_json::Value::Object(said)
}

fn tick(time: std::io::Result<std::time::SystemTime>) -> Option<i64> {//every platform hands file times out in UTC, so one line serves them all; FAT's are local clock readings the reading computer converts with its own zone, taken as given
	i64::try_from(time.ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_millis()).ok()
}
#[cfg(unix)]
fn changed(_: &std::fs::File, meta: &std::fs::Metadata) -> Option<i64> {
	use std::os::unix::fs::MetadataExt;
	meta.ctime().checked_mul(1000)?.checked_add(meta.ctime_nsec() / 1_000_000)
}
#[cfg(windows)]
fn changed(file: &std::fs::File, _: &std::fs::Metadata) -> Option<i64> {
	use windows::Win32::Storage::FileSystem::{FileBasicInfo, FILE_BASIC_INFO};
	handle_info::<FILE_BASIC_INFO>(file, FileBasicInfo)?.ChangeTime.checked_div(10_000)?.checked_sub(11_644_473_600_000)//hundred-nanosecond steps since 1601, to milliseconds since 1970
}
#[cfg(not(any(unix, windows)))]
fn changed(_: &std::fs::File, _: &std::fs::Metadata) -> Option<i64> { None }

#[cfg(unix)]
fn links(_: &std::fs::File, meta: &std::fs::Metadata) -> u64 { use std::os::unix::fs::MetadataExt; meta.nlink() }
#[cfg(windows)]
fn links(file: &std::fs::File, _: &std::fs::Metadata) -> u64 {
	use windows::Win32::Storage::FileSystem::{FileStandardInfo, FILE_STANDARD_INFO};
	handle_info::<FILE_STANDARD_INFO>(file, FileStandardInfo).map_or(1, |standard| standard.NumberOfLinks.into())
}
#[cfg(not(any(unix, windows)))]
fn links(_: &std::fs::File, _: &std::fs::Metadata) -> u64 { 1 }

#[cfg(windows)]
fn handle_info<T: Default>(file: &std::fs::File, class: windows::Win32::Storage::FileSystem::FILE_INFO_BY_HANDLE_CLASS) -> Option<T> {//what Windows keeps about an open file that Rust's standard library reads only in its unstable builds, the class naming which struct T is
	use std::os::windows::io::AsRawHandle;
	use windows::Win32::Foundation::HANDLE;
	use windows::Win32::Storage::FileSystem::GetFileInformationByHandleEx;
	let mut info = T::default();
	unsafe { GetFileInformationByHandleEx(HANDLE(file.as_raw_handle()), class, &mut info as *mut T as *mut core::ffi::c_void, std::mem::size_of::<T>() as u32) }.ok()?;
	Some(info)
}

#[cfg(target_os = "macos")]
const UF_HIDDEN: u32 = 0x0000_8000;//by its name in sys/stat.h
#[cfg(target_os = "macos")]
fn hidden(meta: &std::fs::Metadata) -> bool { use std::os::macos::fs::MetadataExt; meta.st_flags() & UF_HIDDEN != 0 }
#[cfg(windows)]
const FILE_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;//by its name in winnt.h
#[cfg(windows)]
fn hidden(meta: &std::fs::Metadata) -> bool { use std::os::windows::fs::MetadataExt; meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 }
#[cfg(not(any(target_os = "macos", windows)))]
fn hidden(_: &std::fs::Metadata) -> bool { false }//Linux has no hidden flag, and a dot name is not one

fn json_say(value: &impl Serialize, indent: bool) -> String {//indented across lines, or all on the line
	let said = if indent { serde_json::to_string_pretty(value) } else { serde_json::to_string(value) };
	said.unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}).to_string())
}
fn path_say(path: &Path) -> String {//a name that is not UTF-8 loses those characters here, after the file was read
	let text = path.to_string_lossy();
	if cfg!(windows) { text.replace('\\', "/") } else { text.into_owned() }
}
