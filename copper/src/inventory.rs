//an inventory: a folder's whole tree, every file in it hashed and scouted, and the videos among them asked of FFmpeg, a line each in a text file, written by the loop here in the main process or in the sandbox process; brass's inventory.rs holds the commands that start one, read its counts and stop it

use serde::{Deserialize, Serialize};
use std::io::{BufRead, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::Mutex;
use steel::read::{open_file, tip_read};
use steel::walk::{Item, Kind};
use zootella_ergo::{has_tick, hash_file, size, tick_to_text, Blake3, Sha1, Sha256};

/*
Two processes can take an inventory, and the user chooses on the panel. In the main process the loop runs on the thread that calls run, so a read the kernel never finishes, on a dropped share or a dying disk, holds the inventory until fuji quits, and a crash in the media library's C ends fuji. In the sandbox process the same loop runs in a second copy of fuji's executable, started for the one inventory, which a crash ends alone and which the main process can kill: brass's run() sees SANDBOX_SWITCH as the first argument before Tauri starts and hands the process to sandbox_serve here, so it makes no window and runs nothing of brass after that line.

The two talk over the three standard streams. The request goes down stdin as one line of JSON, and stdin stays open after it, because its closing is how the sandbox process learns that the main process has gone, however it went. The lines go into the inventory file through stdout, which the main process opened and handed over, so the file has one writer. The counts come up stderr as a line of JSON when a file begins and as each piece of a hash is read, so a crash finds the counts naming the file in hand; anything else on stderr is the sandbox process's own words, a panic's among them, kept for the error. Stop on the panel and the window closing kill it.

One sandbox process takes the whole inventory rather than one a file, since fuji's executable takes 28 to 40 ms to reach main on the Mac mini, measured on 2026-10-13, against about a millisecond of FFmpeg's work for a file. And what the sandbox process wrote is untrusted bytes to whatever reads an inventory back.
*/

#[derive(Default)]
pub struct Inventory {//one inventory's counts, written by its thread and read from the window's
	folders: AtomicU64,//entered, the one it began at among them
	files:   AtomicU64,//every name the walker said was a file, read or not
	bytes:   AtomicU64,//the sizes of the files done, and with hashes checked, how far into the one being read
	size:    AtomicU64,//of the file being read now
	read:    AtomicU64,//how far into it
	path:    Mutex<String>,
	pub stop: AtomicBool,//set by the commands in brass, or by stdin closing under fuji --inventory; read before every file and every piece of a hash, and by the watcher that kills the sandbox process
	report: Option<Box<dyn Fn(&Progress) + Send + Sync>>,//in the sandbox process, how the counts go up to the main process
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
	fn moved(&self) { if let Some(report) = &self.report { report(&self.progress()) } }//a file begun or a piece of a hash read, which the main process hears of at once
	fn mirror(&self, progress: &Progress) {//the counts the sandbox process reported, kept here for the page to read as it reads its own; stop stays this process's
		self.folders.store(progress.folders, Relaxed);
		self.files.store(progress.files, Relaxed);
		self.bytes.store(progress.bytes, Relaxed);
		self.size.store(progress.size, Relaxed);
		self.read.store(progress.read, Relaxed);
		if let Ok(mut path) = self.path.lock() { path.clone_from(&progress.path) }
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Options {//what the panel sends with a run: the whole-file hashes the user checked, each a read of every byte, whether the JSON is indented across lines, the extensions the page's type table calls video, which get FFmpeg's reading, as does any file the scout calls video, and which process takes it
	sha1:    bool,
	sha256:  bool,
	blake3:  bool,
	indent:  bool,
	video:   Vec<String>,//lowercase, with their dots
	sandbox: bool,//in the sandbox process rather than the main one
	#[serde(default)]
	crash:   bool,//a test's and never the panel's: the sandbox process faults partway, so a walk meets the crash path on Windows too, which has no signal to send another process
	#[serde(default)]
	hang:    bool,//a test's too: the sandbox process stops partway and never goes on, as on a read the kernel never finishes, so a walk meets Stop against a process that has stopped listening
}
impl Options { fn whole(&self) -> bool { self.sha1 || self.sha256 || self.blake3 } }

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
const STOP_CHECK: std::time::Duration = std::time::Duration::from_millis(20);//how often the main process looks for a stop to kill the sandbox process on
const CRASH_AFTER: std::time::Duration = std::time::Duration::from_secs(1);//a test's crash, a second into the walk
const HANG_AFTER: std::time::Duration = std::time::Duration::from_secs(1);//a test's hang, a second into the walk, at the first report after it
const SAID_LAST: usize = 4;//lines of the sandbox process's own words kept for the error, the last ones, since a panic's are the last it writes and FFmpeg's logging can come before

pub fn run(folder: &str, output: &str, options: &Options, inventory: &Inventory) -> Result<(), String> {//the whole inventory, until the tree ends or stop is set: on the thread that calls it, or in the sandbox process when the options say so
	let mut out = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(output).map_err(|e| format!("inventory: {output}: {e}"))?;
	if options.sandbox { return sandbox_run(folder, output, out, options, inventory) }
	take(folder, output, &mut out, options, inventory).map_err(|e| format!("inventory: {output}: {e}"))
}

fn take(folder: &str, output: &str, out: &mut impl Write, options: &Options, inventory: &Inventory) -> std::io::Result<()> {//the loop, the same in either process: a line into out for every name the walk meets
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
		out.write_all(line.as_bytes())?;//unbuffered, so the file is complete up to the file being read
	}
	Ok(())
}

pub const SANDBOX_SWITCH: &str = "--inventory-sandbox";//the first argument that makes fuji's executable the sandbox process
pub const COMMAND_SWITCH: &str = "--inventory";//and the one that makes it take one inventory with no window, for a script or a test, from the request the sandbox process reads

#[derive(Serialize, Deserialize)]
struct Request { folder: String, output: String, options: Options }//what the main process hands the sandbox process down its stdin, and what fuji --inventory reads from its own

fn sandbox_run(folder: &str, output: &str, out: std::fs::File, options: &Options, inventory: &Inventory) -> Result<(), String> {//the main process's side: start the sandbox process writing into out, keep its counts, kill it at a stop, and say how it ended
	let fail = |e: &dyn std::fmt::Display| format!("inventory: the sandbox process: {e}");
	let request = serde_json::to_string(&Request { folder: folder.into(), output: output.into(), options: options.clone() }).map_err(|e| fail(&e))?;
	let program = std::env::current_exe().map_err(|e| fail(&e))?;//this program, wherever it was started from, a development build included
	let (mut child, mut stdin, stderr) = crate::sandbox::start(&program, SANDBOX_SWITCH, out).map_err(|e| fail(&e))?;
	if let Err(e) = writeln!(stdin, "{request}") { let _ = child.kill(); let _ = child.wait(); return Err(fail(&e)) }
	let child = Mutex::new(child);
	let ended = AtomicBool::new(false);
	let said = std::thread::scope(|s| {
		s.spawn(|| while !ended.load(Relaxed) {//the watcher: a stop kills the sandbox process, which no read stuck in the kernel and no demuxer's loop can hold off
			if inventory.stop.load(Relaxed) { if let Ok(mut child) = child.lock() { let _ = child.kill(); } break }
			std::thread::sleep(STOP_CHECK);
		});
		let said = reports_read(std::io::BufReader::new(stderr), inventory);//until the sandbox process ends, which closes its stderr
		ended.store(true, Relaxed);
		said
	});
	let status = child.into_inner().unwrap_or_else(|poisoned| poisoned.into_inner()).wait().map_err(|e| fail(&e))?;
	drop(stdin);//only now, after it has ended
	if status.success() || inventory.stop.load(Relaxed) { return Ok(()) }
	let path = inventory.progress().path;
	let reading = if path.is_empty() { String::new() } else { format!(", reading {path}") };//blank before the first file, as when the request itself was refused
	Err(format!("inventory: the sandbox process ended without finishing{reading}, {status}{said}"))
}
fn reports_read(input: impl BufRead, inventory: &Inventory) -> String {//each line the sandbox process writes on stderr: its counts, kept as the counts here, or its own words, the last few kept for the error
	let mut said = std::collections::VecDeque::new();
	for line in input.split(b'\n') {//by bytes, so a library's line that is not UTF-8 is read past rather than ending the reading, which would leave the sandbox process blocked on a full pipe
		let Ok(line) = line else { break };
		match serde_json::from_slice::<Progress>(&line) {
			Ok(progress) => inventory.mirror(&progress),
			Err(_) => { let words = String::from_utf8_lossy(&line).trim().to_string(); if !words.is_empty() { said.push_back(words); if said.len() > SAID_LAST { said.pop_front(); } } }
		}
	}
	said.into_iter().map(|words| format!("; {words}")).collect()
}

fn request_read(input: &mut impl BufRead) -> Request {//the first line on stdin, or an exit that says why there was none
	let mut line = String::new();
	let request = input.read_line(&mut line).map_err(|e| e.to_string()).and_then(|_| serde_json::from_str::<Request>(&line).map_err(|e| e.to_string()));
	match request { Ok(request) => request, Err(e) => { let _ = writeln!(std::io::stderr(), "the request: {e}"); std::process::exit(2) } }
}

/// An inventory from the command line, from brass's run(): the request off stdin, taken in this process or in the sandbox process as its options say, with stdin closing early as the panel's Stop; the counts at the end go to stdout as a line of JSON, and an error to stderr with exit status 1
pub fn command_serve() -> ! {
	let mut input = std::io::BufReader::new(std::io::stdin());
	let request = request_read(&mut input);
	let inventory = std::sync::Arc::new(Inventory::default());
	let stopping = inventory.clone();
	std::thread::spawn(move || { let _ = std::io::copy(&mut input, &mut std::io::sink()); stopping.stop.store(true, Relaxed) });
	match run(&request.folder, &request.output, &request.options, &inventory) {
		Ok(()) => { let _ = report_write(&mut std::io::stdout().lock(), &inventory.progress()); std::process::exit(0) }
		Err(e) => { let _ = writeln!(std::io::stderr(), "{e}"); std::process::exit(1) }
	}
}

/// The sandbox process's whole life, from brass's run(): the request off stdin, the inventory into stdout, its counts up stderr, and an exit that says how it went
pub fn sandbox_serve() -> ! {
	let mut input = std::io::BufReader::new(std::io::stdin());
	let request = request_read(&mut input);
	if let Err(e) = crate::sandbox::enter(&request.folder) { let _ = writeln!(std::io::stderr(), "{e}"); std::process::exit(3) }//before the walk opens a file, and never skipped: the user chose the sandbox process for its walls
	if request.options.crash { std::thread::spawn(|| { std::thread::sleep(CRASH_AFTER); unsafe { std::ptr::write_volatile(8 as *mut u8, 0) } }); }//an access to an address nothing maps, a fault on any platform
	std::thread::spawn(move || { let _ = std::io::copy(&mut input, &mut std::io::sink()); std::process::exit(0) });//stdin closes when the main process ends, however it ends, and the sandbox process ends with it
	let hang = request.options.hang.then(std::time::Instant::now);//a test's: the loop held at its first report past HANG_AFTER, as a read the kernel never finishes would hold it, with stdin still watched, so only the main process's kill ends it
	let inventory = Inventory { report: Some(Box::new(move |progress| { let _ = report_write(&mut std::io::stderr().lock(), progress); if hang.is_some_and(|began| began.elapsed() > HANG_AFTER) { loop { std::thread::sleep(HANG_AFTER) } } })), ..Default::default() };
	let taken = take(&request.folder, &request.output, &mut std::io::stdout().lock(), &request.options, &inventory);
	inventory.moved();//the last counts, with the folders and files after the last file begun
	let _ = std::io::stdout().flush();
	if let Err(e) = taken { let _ = writeln!(std::io::stderr(), "{}: {e}", request.output); std::process::exit(1) }
	std::process::exit(0)
}
fn report_write(out: &mut impl Write, progress: &Progress) -> std::io::Result<()> {//one line of JSON in one write, so the main process never reads half of one
	let mut text = serde_json::to_string(progress).map_err(std::io::Error::other)?;
	text.push('\n');
	out.write_all(text.as_bytes())
}

fn file_read(item: &Item, inventory: &Inventory, options: &Options, buffer: &mut [u8]) -> std::io::Result<(String, u64, serde_json::Value)> {//one file: its fuji tip hash, its size, and its tip, stat, whichever whole-file hashes options ask for, and FFmpeg's reading of a video
	if let Ok(mut path) = inventory.path.lock() { *path = path_say(&item.path) }
	inventory.size.store(item.size, Relaxed);//the walker's, until the open file says
	inventory.read.store(0, Relaxed);
	inventory.moved();//before the open, so a file that holds up the sandbox process from its first call is the one the counts name
	let mut file = open_file(&item.path)?;
	let meta = file.metadata()?;
	let stat = stat_said(&file, &meta);//before any byte is read, so the access time is the file's own
	let size = meta.len();
	inventory.size.store(size, Relaxed);
	let tip = tip_read(&mut file, size)?;
	let mut said = serde_json::json!({"tip": tip_said(&tip), "stat": stat});
	if options.whole() { said["hash"] = whole_hash(&mut file, size, options, inventory, buffer)? } else { inventory.bytes.fetch_add(size, Relaxed); }
	let tip_hash = match size { 0 => "-".into(), _ => hash_file(&mut file, size)?.tip_hash.map_or_else(|| "-".into(), |hash| hash.base32()) };//ergo's start, middle and end stripes, which a full read just left in the cache; an empty file has none
	if tip.kind == "video" || options.video.contains(&extension_say(&item.path)) { drop(file); said["ffmpeg"] = ffmpeg_said(&item.path) }//our handle closed first, since mpv opens the path for itself
	Ok((tip_hash, size, said))
}
fn extension_say(path: &Path) -> String { path.extension().map_or_else(String::new, |e| format!(".{}", e.to_string_lossy().to_lowercase())) }//as the page's type table spells one, lowercase with its dot; blank for a name with none
fn ffmpeg_said(path: &Path) -> serde_json::Value {//FFmpeg's reading of the file through mpv's player, as media_probe answers it, cut to what the scout also reads so the two sit side by side: the demuxer's name for the format, the running time, and each track's type, codec and demux facts, mpv's own names kept, with the flags that mark a picture track, cover art in a sound file, only where they are set. A refusal, or no answer within the question's timeout, is an error here, and the inventory goes on
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
		inventory.moved();
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
	use zootella_ergo::time;
	meta.ctime().checked_mul(time::SECOND)?.checked_add(meta.ctime_nsec() / 1_000_000)
}
#[cfg(windows)]
fn changed(file: &std::fs::File, _: &std::fs::Metadata) -> Option<i64> {
	use windows::Win32::Storage::FileSystem::{FileBasicInfo, FILE_BASIC_INFO};
	handle_info::<FILE_BASIC_INFO>(file, FileBasicInfo)?.ChangeTime.checked_div(10_000)?.checked_sub(11_644_473_600_000)//hundred-nanosecond steps since 1601, to milliseconds since 1970
}
#[cfg(not(any(unix, windows)))]
fn changed(_: &std::fs::File, _: &std::fs::Metadata) -> Option<i64> { None }

#[cfg(unix)]
pub(crate) fn links(_: &std::fs::File, meta: &std::fs::Metadata) -> u64 { use std::os::unix::fs::MetadataExt; meta.nlink() }
#[cfg(windows)]
pub(crate) fn links(file: &std::fs::File, _: &std::fs::Metadata) -> u64 {
	use windows::Win32::Storage::FileSystem::{FileStandardInfo, FILE_STANDARD_INFO};
	handle_info::<FILE_STANDARD_INFO>(file, FileStandardInfo).map_or(1, |standard| standard.NumberOfLinks.into())
}
#[cfg(not(any(unix, windows)))]
pub(crate) fn links(_: &std::fs::File, _: &std::fs::Metadata) -> u64 { 1 }

#[cfg(windows)]
pub(crate) fn handle_info<T: Default>(file: &std::fs::File, class: windows::Win32::Storage::FileSystem::FILE_INFO_BY_HANDLE_CLASS) -> Option<T> {//what Windows keeps about an open file that Rust's standard library reads only in its unstable builds, the class naming which struct T is
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
		let options = Options { sha1: true, sha256: false, blake3: false, indent: false, video: vec![".ts".into(), ".mp4".into()], sandbox: false, crash: false, hang: false };
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
		let options = Options { sha1: false, sha256: false, blake3: false, indent: true, video: vec![".txt".into()], sandbox: false, crash: false, hang: false };
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
		let options = Options { sha1: false, sha256: false, blake3: false, indent: false, video: vec![], sandbox: false, crash: false, hang: false };
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
		let options = Options { sha1: false, sha256: false, blake3: false, indent: false, video: vec![], sandbox: false, crash: false, hang: false };
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
		let options = Options { sha1: false, sha256: true, blake3: false, indent: false, video: vec![], sandbox: false, crash: false, hang: false };
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
		let options = Options { sha1: false, sha256: true, blake3: false, indent: false, video: vec![], sandbox: false, crash: false, hang: false };
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

	#[test]
	fn the_counts_the_sandbox_process_reports_are_the_counts_the_main_process_keeps() {//the loop as the sandbox process runs it, reporting into a buffer rather than up stderr, then that buffer read as the main process reads stderr, with a library's line before the counts and a panic's after: the main process ends with the loop's own counts and both lines of words. And the report naming each file comes before any of its bytes are read, which is how a crash is pinned on the file in hand
		let reported = std::sync::Arc::new(Mutex::new(Vec::new()));
		let into = reported.clone();
		let sandbox = Inventory { report: Some(Box::new(move |progress| { report_write(&mut *into.lock().unwrap(), progress).unwrap() })), ..Default::default() };
		let options = Options { sha1: true, sha256: false, blake3: false, indent: false, video: vec![], sandbox: true, crash: false, hang: false };
		let mut lines = Vec::new();
		take(concat!(env!("CARGO_MANIFEST_DIR"), "/../steel/samples"), "", &mut lines, &options, &sandbox).unwrap();
		sandbox.moved();
		let mut stderr = b"[swscaler @ 0x1] a library's line, before the counts\n".to_vec();
		stderr.extend_from_slice(&reported.lock().unwrap());
		stderr.extend_from_slice(b"thread 'main' panicked at the end\n\n");
		let main = Inventory::default();
		let said = reports_read(&stderr[..], &main);
		assert_eq!((main.progress(), said.as_str()), (sandbox.progress(), "; [swscaler @ 0x1] a library's line, before the counts; thread 'main' panicked at the end"));
		let begun: Vec<Progress> = stderr.split(|b| *b == b'\n').filter_map(|line| serde_json::from_slice(line).ok()).filter(|p: &Progress| p.read == 0 && !p.path.is_empty()).collect();
		assert_eq!(begun.len(), 72, "a report for every file as it begins, before its first byte");
	}
}
