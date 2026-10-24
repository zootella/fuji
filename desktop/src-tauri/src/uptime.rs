//the uptime command: how long this process has been running since the operating system started it, which copper reads, and since main began, which window.rs recorded

use serde::Serialize;
use std::time::SystemTime;
use tauri::command;

#[derive(Serialize)]
pub struct Uptime {
	process: u64,//milliseconds since the operating system started this process, or 0 when it would not say
	main: u64,//milliseconds since main began
}

/// How long this process has been running: since the operating system started it, which counts the loader's work before main, and since main began
#[command]
pub fn uptime_process() -> Uptime {
	let now = SystemTime::now();
	let since = |then: Option<SystemTime>| then.and_then(|then| now.duration_since(then).ok()).map(|d| d.as_millis() as u64).unwrap_or(0);
	Uptime { process: since(copper::uptime::started()), main: since(Some(crate::window::window_launched())) }
}
