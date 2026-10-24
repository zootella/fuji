//an inventory: a folder's whole tree, every file in it hashed and scouted, and the videos among them asked of FFmpeg, a line each in a text file, written by the loop here; brass's inventory.rs holds the commands that start one, read its counts and stop it

use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::Mutex;
use steel::read::{open_file, tip_read};
use steel::walk::{Item, Kind};
use zootella_ergo::{has_tick, hash_file, size, tick_to_text, Blake3, Sha1, Sha256};

/*
What it trusts: every read is made in fuji's own process, so a read the kernel never finishes, on a dropped share or a dying disk, holds the inventory for good, and only quitting fuji ends it. A process of its own, which can be killed, is the answer to that. The scout is safe Rust and the hashes interpret nothing, so no file's bytes can steer those; FFmpeg's reading of a video is C in this same process, which is what running the inventory in a restricted copy of fuji is for.
*/

#[derive(Default)]
pub struct Inventory {//one inventory's counts, written by its thread and read from the window's
	folders: AtomicU64,//entered, the one it began at among them
	files:   AtomicU64,//every name the walker said was a file, read or not
	bytes:   AtomicU64,//the sizes of the files done, and with hashes checked, how far into the one being read
	size:    AtomicU64,//of the file being read now
	read:    AtomicU64,//how far into it
	path:    Mutex<String>,
	pub stop: AtomicBool,//set by the commands in brass, read before every file and every piece of a hash
}
impl Inventory {
	pub fn progress(&self) -> Progress {
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

#[derive(Deserialize)]
pub struct Options {//what the panel sends with a run: the whole-file hashes the user checked, each a read of every byte, whether the JSON is indented across lines, and the extensions the page's type table calls video, which get FFmpeg's reading, as does any file the scout calls video
	sha1:   bool,
	sha256: bool,
	blake3: bool,
	indent: bool,
	video:  Vec<String>,//lowercase, with their dots
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

const HASH_BUFFER: usize = size::MB as usize;//a read at a time, so the progress within a large file moves every few milliseconds
const THREAD_PIECE: usize = 64 * size::KB as usize;//the least a piece takes to be worth three threads, which cost about 28 µs to start on the Mac mini's M2, more than hashing anything smaller
const PROBE_SECONDS: f64 = 10.0;//what a file gets to answer mpv before the inventory goes on without it, as the audio panel gives a dropped file

pub fn run(folder: &str, output: &str, options: &Options, inventory: &Inventory) -> Result<(), String> {//the whole inventory, on the thread that calls it, until the tree ends or stop is set
	let mut out = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(output).map_err(|e| format!("inventory: {output}: {e}"))?;
	let mut buffer = vec![0u8; HASH_BUFFER];//sized by fuji, never by a file
	for item in steel::walk::walk(folder) {
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

fn file_read(item: &Item, inventory: &Inventory, options: &Options, buffer: &mut [u8]) -> std::io::Result<(String, u64, serde_json::Value)> {//one file: its fuji tip hash, its size, and its tip, stat, whichever whole-file hashes options ask for, and FFmpeg's reading of a video
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
	if tip.kind == "video" || options.video.contains(&extension_say(&item.path)) { drop(file); said["ffmpeg"] = ffmpeg_said(&item.path) }//our handle closed first, since mpv opens the path for itself
	Ok((tip_hash, size, said))
}
fn extension_say(path: &Path) -> String { path.extension().map_or_else(String::new, |e| format!(".{}", e.to_string_lossy().to_lowercase())) }//as the page's type table spells one, lowercase with its dot; blank for a name with none
fn ffmpeg_said(path: &Path) -> serde_json::Value {//FFmpeg's reading of the file through mpv's player, as media_probe answers it, cut to what the scout also reads so the two sit side by side: the demuxer's name for the format, the running time, and each track's type, codec and demux facts, mpv's own names kept, with the flags that mark a picture track, cover art in a sound file, only where they are set. A refusal, or no answer by the deadline, is an error here, and the inventory goes on
	let probe = match crate::media::probe(&path.to_string_lossy(), PROBE_SECONDS) { Ok(probe) => probe, Err(e) => return serde_json::json!({"error": e}) };
	let mut said = serde_json::Map::new();
	if !probe.format.is_empty() { said.insert("format".into(), probe.format.into()); }
	if let Ok(seconds) = probe.duration.parse::<f64>() { said.insert("seconds".into(), seconds.into()); }
	if let Ok(serde_json::Value::Array(tracks)) = serde_json::from_str::<serde_json::Value>(&probe.tracks) {
		let kept = tracks.into_iter().map(|track| match track {
			serde_json::Value::Object(fields) => serde_json::Value::Object(fields.into_iter().filter(|(key, value)| key == "type" || key == "codec" || key.starts_with("demux-") || ((key == "image" || key == "albumart") && *value == serde_json::Value::Bool(true))).collect()),
			other => other,
		}).collect();
		said.insert("tracks".into(), serde_json::Value::Array(kept));
	}
	serde_json::Value::Object(said)
}
fn whole_hash(file: &mut std::fs::File, size: u64, options: &Options, inventory: &Inventory, buffer: &mut [u8]) -> std::io::Result<serde_json::Value> {//the hashes options ask for, from one read of every byte; a stop ends it early
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
fn tip_said(tip: &steel::tip::Tip) -> serde_json::Value {//the scout's answer without its reading, which millions of entries have no room for
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

#[cfg(test)]
mod tests {
	use super::*;

	struct Tree(std::path::PathBuf);//a folder of one test's own under the system's temporary folder, removed when the test ends, as steel's tests build theirs
	impl Tree {
		fn new(name: &str) -> Tree { let path = std::env::temp_dir().join(format!("copper-inventory-{name}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&path); std::fs::create_dir_all(&path).unwrap(); Tree(path) }
		fn file(&self, relative: &str) { let path = self.0.join(relative); std::fs::create_dir_all(path.parent().unwrap()).unwrap(); std::fs::write(path, relative).unwrap() }//a small file whose contents are its own name, so its size is the name's length
		fn text(&self, relative: &str) -> String { let text = self.0.join(relative).to_string_lossy().into_owned(); if cfg!(windows) { text.replace('\\', "/") } else { text } }//the path as the page would send it and as a line says it; on the Mac and Linux a backslash is a letter in a name
	}
	impl Drop for Tree { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }
	fn parse(line: &str) -> (String, String, u64, serde_json::Value) {//a line read back by its grammar: the hash, the path as one JSON string, the size, and the JSON
		let (hash, rest) = line.split_once(' ').unwrap();
		let mut values = serde_json::Deserializer::from_str(rest).into_iter::<serde_json::Value>();
		let path = values.next().unwrap().unwrap().as_str().unwrap().to_string();
		let (size, json) = rest[values.byte_offset()..].trim_start().split_once(' ').unwrap();
		(hash.to_string(), path, size.parse().unwrap(), serde_json::from_str(json).unwrap())
	}

	#[test]
	fn an_inventory_of_the_samples() {//the whole loop over steel's sample files into a file in the system's temporary folder: the counts, and the lines a video and a sound file get
		let output = std::env::temp_dir().join(format!("copper-inventory-{}.txt", std::process::id()));
		let options = Options { sha1: true, sha256: false, blake3: false, indent: false, video: vec![".ts".into(), ".mp4".into()] };
		let inventory = Inventory::default();
		run(concat!(env!("CARGO_MANIFEST_DIR"), "/../steel/samples"), output.to_str().unwrap(), &options, &inventory).unwrap();
		let text = std::fs::read_to_string(&output).unwrap();
		let _ = std::fs::remove_file(&output);
		assert_eq!((inventory.folders.load(Relaxed), inventory.files.load(Relaxed), text.lines().count()), (1, 72, 72));
		let line = |name: &str| text.lines().find(|l| l.contains(&format!("/{name}\" "))).unwrap().to_string();
		assert!(line("video.ts").contains("\"ffmpeg\":{\"format\":\"mpegts\"") && line("video.ts").contains("\"sha1\":"));
		assert!(!line("audio.wav").contains("\"ffmpeg\""));
		assert!(line("picture.png").contains("\"sha1\":\"ca340ae965cd24e99b458d3bfdf5466632658b19\""), "the SHA-1 shasum gives for the same file, in base16: {}", line("picture.png"));
		assert!(line("picture.png").contains("\"tip\":{") && line("picture.png").contains("\"kind\":\"image\"") && line("picture.png").contains("\"modified\":"), "the scout's answer and the file system's record on the same line");
	}

	#[test]
	fn the_probe_is_asked_by_the_scouts_kind_or_by_the_extension_and_a_refusal_is_written_as_an_error() {//with no extension named, video.ts still gets FFmpeg's reading, since the scout calls its bytes video; with .txt named, text.txt is asked too, and what FFmpeg says of a text file is an error under the same key; indent writes each file's JSON across lines
		let output = std::env::temp_dir().join(format!("copper-inventory-gating-{}.txt", std::process::id()));
		let options = Options { sha1: false, sha256: false, blake3: false, indent: true, video: vec![".txt".into()] };
		run(concat!(env!("CARGO_MANIFEST_DIR"), "/../steel/samples"), output.to_str().unwrap(), &options, &Inventory::default()).unwrap();
		let text = std::fs::read_to_string(&output).unwrap();
		let _ = std::fs::remove_file(&output);
		assert!(text.lines().count() > 72, "indented JSON spreads a file over several lines: {}", text.lines().count());
		let entry = |name: &str| { let at = text.find(&format!("/{name}\" ")).unwrap(); let end = text[at..].find("\n}\n").map(|e| at + e + 3).unwrap(); text[at..end].to_string() };
		assert!(entry("video.ts").contains("\"ffmpeg\": {") && entry("video.ts").contains("\"format\": \"mpegts\""), "{}", entry("video.ts"));
		assert!(entry("text.txt").contains("\"ffmpeg\": {\n    \"error\": \"the media library could not open it"), "{}", entry("text.txt"));
		assert!(!entry("picture.png").contains("\"ffmpeg\""), "{}", entry("picture.png"));
	}

	#[test]
	fn a_stop_set_before_the_first_file_leaves_an_empty_file_and_zero_counts() {//stop is read before every name, so one set before the walk begins ends it at the folder itself; the file is still made, as the page expects to append its footer to
		let output = std::env::temp_dir().join(format!("copper-inventory-stopped-{}.txt", std::process::id()));
		let options = Options { sha1: false, sha256: false, blake3: false, indent: false, video: vec![] };
		let inventory = Inventory::default();
		inventory.stop.store(true, Relaxed);
		run(concat!(env!("CARGO_MANIFEST_DIR"), "/../steel/samples"), output.to_str().unwrap(), &options, &inventory).unwrap();
		let text = std::fs::read_to_string(&output).unwrap();
		let _ = std::fs::remove_file(&output);
		assert_eq!((text.len(), inventory.folders.load(Relaxed), inventory.files.load(Relaxed), inventory.progress().stopped), (0, 0, 0, true));
	}

	#[test]
	fn a_folder_that_is_not_there_gives_one_refused_line_and_no_error() {//the walker answers a missing start as Refused, with the system's words, which the inventory writes as a line like any other name it cannot open
		let output = std::env::temp_dir().join(format!("copper-inventory-missing-{}.txt", std::process::id()));
		let options = Options { sha1: false, sha256: false, blake3: false, indent: false, video: vec![] };
		run("/no/such/folder/anywhere", output.to_str().unwrap(), &options, &Inventory::default()).unwrap();
		let text = std::fs::read_to_string(&output).unwrap();
		let _ = std::fs::remove_file(&output);
		assert_eq!(text.lines().count(), 1);
		assert!(text.starts_with("- ") && text.contains("\"kind\":\"Refused\"") && text.contains("\"error\":"), "{text}");
	}

	#[test]
	fn every_line_reads_back_whatever_a_name_holds() {//the names a collection holds, each a line that reads back by the grammar: a space, a letter past ascii, a name with no extension, and where the platform allows them a quote, a backslash, a newline and a tab; two names that are one file through a hard link, which their lines show by a links count of two and one hash; a file that refuses to open, which gets its line as an error with the size the walker saw; a time before 1970, left out of stat as the placeholder it is; a file the Mac flags hidden; and the inventory's own output inside the folder, which gets no line
		let t = Tree::new("names");
		let mut names = vec!["spa ce.gif", "café.png", "noextension", "deep/er/file.txt", "linked.jpg", "twin.jpg", "early.txt"];
		#[cfg(unix)] names.extend(["quote\" mark.jpg", "back\\slash.txt", "new\nline.bin", "tab\there.txt", "locked.txt"]);
		#[cfg(windows)] names.extend(["brackets [1].jpg", "percent %20.jpg", "hash #1.jpg"]);
		#[cfg(target_os = "macos")] names.push("hidden.txt");
		for name in &names { if *name != "twin.jpg" { t.file(name) } }
		std::fs::hard_link(t.0.join("linked.jpg"), t.0.join("twin.jpg")).unwrap();
		std::fs::File::options().write(true).open(t.0.join("early.txt")).unwrap().set_modified(std::time::UNIX_EPOCH - std::time::Duration::from_secs(1)).unwrap();
		#[cfg(unix)] let locked_opens = { use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(t.0.join("locked.txt"), std::fs::Permissions::from_mode(0o000)).unwrap(); std::fs::File::open(t.0.join("locked.txt")).is_ok() };//root opens it anyway, which leaves that line nothing to test
		#[cfg(target_os = "macos")] assert!(std::process::Command::new("chflags").arg("hidden").arg(t.0.join("hidden.txt")).status().unwrap().success());
		let options = Options { sha1: false, sha256: true, blake3: false, indent: false, video: vec![] };
		let inventory = Inventory::default();
		run(&t.text(""), &t.text("inventory.txt"), &options, &inventory).unwrap();
		let text = std::fs::read_to_string(t.0.join("inventory.txt")).unwrap();
		#[cfg(unix)] { use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(t.0.join("locked.txt"), std::fs::Permissions::from_mode(0o644)).unwrap() }//so the tree can be removed
		let lines: Vec<(String, String, u64, serde_json::Value)> = text.lines().map(parse).collect();
		let mut paths: Vec<&str> = lines.iter().map(|(_, path, _, _)| path.as_str()).collect(); paths.sort_unstable();
		let mut expected: Vec<String> = names.iter().map(|name| t.text(name)).collect(); expected.sort();
		assert_eq!(paths, expected, "every name once, and no line for inventory.txt");
		assert_eq!((inventory.folders.load(Relaxed), inventory.files.load(Relaxed) as usize), (3, names.len()));
		let root = format!("{}/", t.text("").trim_end_matches('/'));
		for (_, path, size, _) in &lines { if !path.ends_with("/twin.jpg") { assert_eq!(*size, path.strip_prefix(&root).unwrap().len() as u64, "{path}") } }//the size the line says is the file's, which is its name's length in bytes; the twin holds linked.jpg's name
		let line = |name: &str| lines.iter().find(|(_, path, _, _)| *path == t.text(name)).unwrap();
		let (hash, _, _, said) = line("linked.jpg");
		let (twin_hash, _, _, twin) = line("twin.jpg");
		assert_eq!((said["stat"]["links"].as_u64(), twin["stat"]["links"].as_u64(), said["hash"]["sha256"] == twin["hash"]["sha256"], hash == twin_hash), (Some(2), Some(2), true, true), "{said}\n{twin}");
		assert!(said["hash"]["sha256"].as_str().unwrap().len() == 52 && said["tip"].is_object() && said["stat"]["modified"].is_string(), "{said}");
		assert!(line("early.txt").3["stat"].get("modified").is_none(), "{}", line("early.txt").3);
		#[cfg(unix)] if !locked_opens { let (hash, _, size, said) = line("locked.txt"); assert_eq!((hash.as_str(), *size, said["error"].is_string(), said.get("tip").is_none()), ("-", 10, true, true), "{said}") }
		#[cfg(target_os = "macos")] assert_eq!((line("hidden.txt").3["stat"]["hidden"].as_bool(), line("early.txt").3["stat"].get("hidden").is_none()), (Some(true), true));
	}

	#[test]
	fn a_stop_set_partway_through_a_file_ends_the_run_there_with_no_line_for_the_part() {//stop is read before every name and between the pieces of a hash, so one set while a file is being hashed ends the hash, and the file gets no line, since a hash of part of it says nothing true. The test holds the lock on the path the loop records as it begins each file, so the run stands at the first file until the test has set stop and let go, which is how a stop lands partway without a clock
		let t = Tree::new("stop");
		for name in ["a.txt", "b.txt", "c.txt"] { t.file(name) }
		let options = Options { sha1: false, sha256: true, blake3: false, indent: false, video: vec![] };
		let inventory = Inventory::default();
		let held = inventory.path.lock().unwrap();
		std::thread::scope(|s| {
			let running = s.spawn(|| run(&t.text(""), &t.text("inventory.txt"), &options, &inventory));
			while inventory.files.load(Relaxed) < 1 { std::thread::yield_now() }//the loop has counted the first file and is waiting for the lock
			inventory.stop.store(true, Relaxed);
			drop(held);
			running.join().unwrap().unwrap();
		});
		let p = inventory.progress();
		assert_eq!((p.folders, p.files, p.bytes, p.stopped, std::fs::read_to_string(t.0.join("inventory.txt")).unwrap().len(), p.path.ends_with("/a.txt")), (1, 1, 0, true, 0, true), "{}", p.path);
	}
}
