use std::sync::Mutex;
use tauri::command;

/*
Fuji's log: one text file per run, holding every line the page or Rust chose to keep. A run is a stretch with windows open rather than the life of the process — the file is written when the last one closes, and on macOS, where fuji stays in the dock after that, a window opened later starts a new file. log.js is the page's half and carries the essay on why a file beats a console. This is the half that holds the text and writes it, because only Rust sees a quit coming — desktop.rs has why RunEvent::Exit is the one place a file can still be written.

Two callers add lines. The page hands its lines down through log_append, a batch at a time. Rust code calls log(text), from anywhere, and it lands in the same text. Neither side promises an exact order against the other, and console.log never did either, so each side starts every line with who wrote it and when: the page with its window's label, and this file with rust----, then the utc time of day. On the mac several windows record into one run, and that prefix is what tells their lines apart.

Nothing here decides whether to record. The page reads the setting, and if it is off, log_start is never called, the path stays blank, and every line from either side is dropped. That is the whole switch: one setting, read once, nothing else to configure.

The state is a static rather than tauri's managed state, so that log(text) is a plain function call with no handle to thread through. A Mutex because commands arrive on tauri's pool threads and the exit write on the main one.
*/

struct Log {
	path: String,//where the file goes, handed down by the page; blank until then, which is how off looks from here
	text: String,//every line so far
}

static LOG: Mutex<Log> = Mutex::new(Log { path: String::new(), text: String::new() });

/// Name the file, unless a run is already recording; each window's page calls this once, and only when the setting says to record
#[command]
pub fn log_start(path: String) {
	let mut log = LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());//take the log even if a previous holder panicked; losing every line for that would be worse
	if log.path.is_empty() { log.path = path }//a second window on the mac joins the run the first one started, and its header lands mid-file to mark where; taking its path instead would throw away every line recorded so far. Nothing to clear either way, since text only grows while a path is set
}

/// Add lines from the page, already ending in a newline
#[command]
pub fn log_append(text: String) {
	let mut log = LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	if log.path.is_empty() { return }//not recording
	log.text.push_str(&text);
}

/// One line from rust; callable from anywhere, and a no-op when the page has not started a log
pub fn log(text: &str) {
	let mut log = LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	if log.path.is_empty() { return }
	log.text.push_str(&format!("rust---- {} {text}\n", log_clock()));//the same prefix the page puts on its lines, with rust where a window's label would be, since no window wrote this; the dashes make it as wide as window-1, so the times line up until a tenth window
}

//utc time of day to the millisecond, like 18:29:27.123, matching sayClock in log.js; the day is left out because the file name already carries it
fn log_clock() -> String {
	let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();//a clock set before 1970 reads as midnight rather than panicking inside a log line
	let seconds = since.as_secs() % 86_400;//seconds into today, since a unix day is exactly that long
	format!("{:02}:{:02}:{:02}.{:03}", seconds / 3600, seconds / 60 % 60, seconds % 60, since.subsec_millis())
}

/// Write the file; called when the last window closes and again from RunEvent::Exit, after which nothing above can be told how it went
pub fn log_write() {
	let mut log = LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	if log.path.is_empty() || log.text.is_empty() { return }
	if let Some(folder) = std::path::Path::new(&log.path).parent() { let _ = std::fs::create_dir_all(folder); }//the folder under home may not exist yet; if this fails, the write below says so
	if let Err(e) = std::fs::write(&log.path, log.text.as_bytes()) {
		eprintln!("could not write the log on the way out: {e}");//stderr, the only place left, and one nobody is likely watching
	}
	log.text.clear();//so nothing is written twice if this is somehow reached again
	log.path.clear();//and the run is over: a later window names a new file rather than appending here, which matters because this write truncates and a second one would keep only the lines that came after the first
}
