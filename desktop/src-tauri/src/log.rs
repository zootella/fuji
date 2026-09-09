use std::sync::Mutex;
use tauri::command;

/*
Fuji's log: one text file per run, written when the application exits, holding every line the page or Rust chose to keep. log.js is the page's half and carries the essay on why a file beats a console. This is the half that holds the text and writes it, because only Rust sees a quit coming — desktop.rs has why RunEvent::Exit is the one place a file can still be written.

Two callers add lines. The page hands its lines down through log_append, a batch at a time. Rust code calls log(text), from anywhere, and it lands in the same text. Neither side promises an exact order against the other, and console.log never did either.

Nothing here decides whether to record. The page reads the setting, and if it is off, log_start is never called, the path stays blank, and every line from either side is dropped. That is the whole switch: one setting, read once, nothing else to configure.

The state is a static rather than tauri's managed state, so that log(text) is a plain function call with no handle to thread through. A Mutex because commands arrive on tauri's pool threads and the exit write on the main one.
*/

struct Log {
	path: String,//where the file goes, handed down by the page; blank until then, which is how off looks from here
	text: String,//every line so far
}

static LOG: Mutex<Log> = Mutex::new(Log { path: String::new(), text: String::new() });

/// Name the file; the page calls this once, and only when the setting says to record
#[command]
pub fn log_start(path: String) {
	let mut log = LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());//take the log even if a previous holder panicked; losing every line for that would be worse
	log.path = path;
	log.text.clear();
}

/// Add lines from the page, already ending in a newline
#[command]
pub fn log_append(text: String) {
	let mut log = LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	if log.path.is_empty() { return }//not recording
	log.text.push_str(&text);
}

/// One line from rust; callable from anywhere, and a no-op when the page has not started a log
#[allow(dead_code)]//no caller yet; it is here so the next one is a call rather than a design
pub fn log(text: &str) {
	let mut log = LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	if log.path.is_empty() { return }
	log.text.push_str(text);
	log.text.push('\n');
}

/// Write the file; called from RunEvent::Exit, after which nothing above can be told how it went
pub fn log_write() {
	let mut log = LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	if log.path.is_empty() || log.text.is_empty() { return }
	if let Some(folder) = std::path::Path::new(&log.path).parent() { let _ = std::fs::create_dir_all(folder); }//the folder under home may not exist yet; if this fails, the write below says so
	if let Err(e) = std::fs::write(&log.path, log.text.as_bytes()) {
		eprintln!("fuji could not write its log on the way out: {e}");//stderr, the only place left, and one nobody is likely watching
	}
	log.text.clear();//so nothing is written twice if this is somehow reached again
}
