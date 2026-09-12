use std::sync::Mutex;
use tauri::{command, AppHandle, Emitter, Manager, State, Url};

/*
A file the operating system handed fuji, because the user double-clicked a picture rather than opening fuji and going looking. This module holds those paths until the page asks for them, and that holding is the whole reason it exists.

The two platforms deliver the same intention by completely different means, and neither can be answered on the spot. macOS keeps one copy of an application running and sends it an Apple event, which Tauri raises as RunEvent::Opened — at launch, and again every time the user opens another picture while fuji is already up. Windows has no event at all: Explorer runs the executable again with the path as an argument, a new process every time, so the whole mechanism is the command line. Linux does what Windows does.

Neither can be handed straight to the page. At launch the event or the argument arrives before the webview exists, and there is nothing to deliver it to; the page is built after. So the paths wait in a list here, and the page drains the list when it mounts. The while-running case on macOS then needs nothing new — the same push happens and an event says there is something to collect, and the page drains the same list by the same call. One path through this instead of two.

The list is drained rather than read, so nothing is shown twice. That matters more than it looks: the page calls this on mount, and a reload during development would otherwise reopen whatever the last launch was given.

What this deliberately does not do is decide anything. It does not look at the extension, does not ask whether fuji can show the file, does not choose a view. The page owns all of that, which is the rule in CLAUDE.md about the two layers. A path arrives here and leaves here unexamined, except for the one mechanical filter below that Windows makes necessary.
*/

//a tuple struct, reached below as .0; the Mutex is required because tauri may hand this to commands on different threads
#[derive(Default)]
pub struct OpenFiles(pub Mutex<Vec<String>>);

/// The paths the operating system has handed fuji since the page last asked, emptying the list as it answers
#[command]
pub fn open_files(files: State<'_, OpenFiles>) -> Vec<String> {//State borrows what lib.rs manages
	let mut files = files.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());//take the list even if a previous holder panicked
	files.drain(..).collect()
}

/// Remember paths for the page to collect, and say that there are some
pub fn open_hold(app: &AppHandle, paths: Vec<String>) {
	if paths.is_empty() { return }
	let state = app.state::<OpenFiles>();
	let mut files = state.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	files.extend(paths);
	let _ = app.emit("open", ());//nothing is listening at launch, and that is fine: the page drains the list when it mounts, and this event is only for the times it is already mounted. The payload is empty on purpose, so there is one way to get a path rather than two
}

/// The paths on the command line, which is how windows and linux hand over a double-clicked file
pub fn open_argv(app: &AppHandle) {
	let paths = std::env::args_os()
		.skip(1)//the first is fuji itself
		.filter_map(|argument| argument.into_string().ok())//an argument that is not utf-8 cannot be a path fuji carries, since everything above here is a rust String and then a javascript string
		.filter(|argument| std::path::Path::new(argument).is_file())//the only examination this module does, and it is mechanical rather than a judgement: a command line also carries switches and a dev build's own arguments, and a path that names a file is what the shell sends and they are not
		.collect();
	open_hold(app, paths)
}

/// The urls macOS sends when the user opens pictures with fuji; called from RunEvent::Opened
pub fn open_urls(app: &AppHandle, urls: Vec<Url>) {
	let paths = urls.iter()
		.filter_map(|url| url.to_file_path().ok())//they are file:// urls, and a url that is not a file is not something this application was ever registered for
		.filter_map(|path| path.to_str().map(|s| s.to_string()))
		.collect();
	open_hold(app, paths)
}
