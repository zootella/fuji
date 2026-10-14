use serde::Serialize;
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_ulong, c_void, CStr, CString};
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{command, AppHandle, Manager, State, WebviewWindow};
use crate::run_blocking;

/*
libmpv, the media library beneath fuji's sound and its video: mpv's client API, declared by hand below rather than through a crate or a bindgen. Fuji calls a handful of its functions, their signatures are fixed by the API's second major version, and the Rust crates over it have gone stale more than once, so the declarations live here, where a reader can check them against client.h in a minute. A handle is a player: mpv_handle in the header and an opaque pointer here, made by mpv_create, given its options as strings, started by mpv_initialize, and ended by mpv_terminate_destroy, which waits for the player's own threads to finish. Every function below may be called from any thread, so a command's body runs on the blocking pool like the rest.

**Four commands are the floor, and they are generic over mpv's own vocabulary.** Start makes a player for the window with the options the page chose; send runs one mpv command with its arguments; read answers one property as text; stop ends the player. The page says what plays, where it draws, when to seek and by how much, in mpv's own names, so a seek, a frame step or a still from the middle of a file is a send and never a new command here. The player lives in a map in managed state keyed by the window's label, one per window, so Windows and Linux run the map with one entry and nothing branches.

**Every call into the library holds that player's lock for the whole of the call.** The handle is a raw pointer, and a stop that destroyed it while a read was inside the library would be a use after free; so a command takes the player's own mutex, checks the handle is still there, calls, and lets go, and a stop takes the same mutex to destroy and then leaves a null behind, which any command that was waiting for the lock finds and reports. The map's lock is held only to look a player up or put one in, never across a call, so one window's stop never waits on another's read.

**A window that closes takes its player with it,** through mpv_window_closed, which lib.rs calls as the window is asked to close. The destroy goes to the blocking pool rather than running on the window's thread: on Windows the video draws in a child of fuji's window that mpv made on a thread of its own, and tearing that down can send to the thread that owns the parent, so a destroy that blocked the window's thread while mpv waited on it would never finish. The page stops the player itself when its view goes away; this is for the window the user closes mid-play.

On the Mac the library is Homebrew's for now, gathered with everything under it into a folder build.rs names, by frameworks.js, which also says how the copies travel into the bundle. On Windows it is one dll, libmpv-2.dll, built from the tagged releases by pnpm libmpv with FFmpeg and libplacebo inside it, kept in src-tauri/binaries untracked and shipped beside the executable by the resources entry in tauri.windows.conf.json; the link attribute names the dll itself, as raw-dylib, so the linker writes the imports from the declarations below and no import library is made or kept. Where the library comes from on each platform in the end is the video spike's subject. On Linux nothing links yet, so lib.rs compiles a stand-in that says so.
*/

type Handle = *mut c_void;//mpv_handle, opaque in the header too

#[cfg_attr(target_os = "macos", link(name = "mpv"))]//libmpv.dylib, in the folder build.rs names
#[cfg_attr(target_os = "windows", link(name = "libmpv-2", kind = "raw-dylib"))]//libmpv-2.dll beside the executable, found by the loader at launch with no import library
extern "C" {
	fn mpv_client_api_version() -> c_ulong;//the api version the library was built with, major in the high sixteen bits and minor in the low
	fn mpv_create() -> Handle;//a player with no options set and not yet started, or null
	fn mpv_set_option_string(ctx: Handle, name: *const c_char, data: *const c_char) -> c_int;//one option as text, before mpv_initialize; 0, or an error code below it
	fn mpv_initialize(ctx: Handle) -> c_int;//start the player; 0, or an error code below it
	fn mpv_command(ctx: Handle, args: *const *const c_char) -> c_int;//one command, its name and arguments as a null-terminated array of strings, run before this returns; 0, or an error code below it
	fn mpv_get_property_string(ctx: Handle, name: *const c_char) -> *mut c_char;//a property as text, to give back with mpv_free, or null for a name the player has no property by
	fn mpv_free(data: *mut c_void);//what the library allocated for the caller
	fn mpv_error_string(error: c_int) -> *const c_char;//what an error code means, in text the library keeps
	fn mpv_terminate_destroy(ctx: Handle);//stop the player and free it, once its threads have finished
}

pub struct Player(Handle);//one running player: its handle, or null once it has been destroyed, which a command that was waiting for the lock finds
unsafe impl Send for Player {}//the pointer may move between threads because every function above may be called from any thread; the mutex around it is what keeps the calls one at a time

type Shared = Arc<Mutex<Player>>;//a player as the map holds it and a command takes it out: shared, so the map's lock is let go before the call, and locked, so the call and a stop take turns
type Map = Arc<Mutex<HashMap<String, Shared>>>;

#[derive(Default)]
pub struct Players(Map);//the players, one per window by its label, in managed state

#[derive(Serialize)]
pub struct Version {
	mpv: String,//the library's own version text, which names itself first, like mpv v0.41.0
	api: String,//the client api it was built against, major.minor
	ffmpeg: String,//the FFmpeg it runs on, which decodes everything it plays
}

/// The media library's versions, read from a player made for the purpose and destroyed after: its own, the client api it was built against, and the FFmpeg beneath it
#[command]
pub async fn mpv_version() -> Result<Version, String> {
	run_blocking(|| {
		let api = unsafe { mpv_client_api_version() };
		let player = unsafe { mpv_create() };
		if player.is_null() { return Err("mpv_create answered null".into()) }
		let started = unsafe { mpv_initialize(player) };
		if started < 0 { unsafe { mpv_terminate_destroy(player) }; return Err(format!("mpv_initialize: {}", error_text(started))) }
		let mpv = property_text(player, c"mpv-version");
		let ffmpeg = property_text(player, c"ffmpeg-version");
		unsafe { mpv_terminate_destroy(player) };
		Ok(Version { mpv, api: format!("{}.{}", api >> 16, api & 0xffff), ffmpeg })
	}).await
}

/// Start a player for this window with these options, each a name and a value as mpv's manual writes them, set before the player starts; a window that had a player gets a new one, and the old one is destroyed first
#[command]
pub async fn mpv_start(window: WebviewWindow, options: Vec<(String, String)>, players: State<'_, Players>) -> Result<(), String> {
	let map = players.0.clone();
	let label = window.label().to_string();
	run_blocking(move || {
		if let Some(old) = map_take(&map, &label) { player_destroy(&old) }
		let player = unsafe { mpv_create() };
		if player.is_null() { return Err("mpv_create answered null".into()) }
		for (name, value) in &options {
			let set = unsafe { mpv_set_option_string(player, text(name)?.as_ptr(), text(value)?.as_ptr()) };
			if set < 0 { unsafe { mpv_terminate_destroy(player) }; return Err(format!("mpv_set_option_string {name}={value}: {}", error_text(set))) }
		}
		let started = unsafe { mpv_initialize(player) };
		if started < 0 { unsafe { mpv_terminate_destroy(player) }; return Err(format!("mpv_initialize: {}", error_text(started))) }
		map_lock(&map).insert(label, Arc::new(Mutex::new(Player(player))));
		Ok(())
	}).await
}

/// Run one mpv command on this window's player: its name and then its arguments, as mpv's manual lists them, like loadfile and a path, or cycle and pause; answers when the command has run
#[command]
pub async fn mpv_send(window: WebviewWindow, args: Vec<String>, players: State<'_, Players>) -> Result<(), String> {
	let player = player_of(&players, window.label())?;
	run_blocking(move || {
		let handle = player_handle(&player)?;
		let owned = args.iter().map(|a| text(a)).collect::<Result<Vec<_>, _>>()?;
		let mut pointers: Vec<*const c_char> = owned.iter().map(|a| a.as_ptr()).collect();
		pointers.push(std::ptr::null());//the array ends with a null, which is how mpv_command counts it
		let ran = unsafe { mpv_command(handle.0, pointers.as_ptr()) };
		if ran < 0 { return Err(format!("mpv_command {}: {}", args.first().map(String::as_str).unwrap_or(""), error_text(ran))) }
		Ok(())
	}).await
}

/// One property of this window's player as text, by its name in mpv's manual, like time-pos or pause; blank for a property that has no value yet, which time-pos has none of before a file is loaded
#[command]
pub async fn mpv_read(window: WebviewWindow, name: String, players: State<'_, Players>) -> Result<String, String> {
	let player = player_of(&players, window.label())?;
	run_blocking(move || {
		let handle = player_handle(&player)?;
		Ok(property_text(handle.0, &text(&name)?))
	}).await
}

/// Stop this window's player and destroy it; nothing, if the window has none
#[command]
pub async fn mpv_stop(window: WebviewWindow, players: State<'_, Players>) -> Result<(), String> {
	let taken = map_take(&players.0, window.label());
	run_blocking(move || { if let Some(player) = taken { player_destroy(&player) } Ok(()) }).await
}

/// The window with this label is closing: destroy its player, if it has one, on the blocking pool, so the window's thread stays free to answer whatever mpv's own threads ask of it on the way down
pub fn mpv_window_closed(app: &AppHandle, label: &str) {
	let Some(players) = app.try_state::<Players>() else { return };
	if let Some(player) = map_take(&players.0, label) { tauri::async_runtime::spawn_blocking(move || player_destroy(&player)); }
}

fn map_lock(map: &Map) -> MutexGuard<'_, HashMap<String, Shared>> { map.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) }//the map after a panic under its lock is still a map, as window.rs and log.rs reason
fn map_take(map: &Map, label: &str) -> Option<Shared> { map_lock(map).remove(label) }//out of the map, so no later command finds it; destroying it is the caller's
fn player_of(players: &Players, label: &str) -> Result<Shared, String> { map_lock(&players.0).get(label).cloned().ok_or_else(|| "this window has no player".into()) }
fn player_handle(player: &Shared) -> Result<MutexGuard<'_, Player>, String> {//the player under its lock, for one call into the library; the lock is held until the guard drops
	let guard = player.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	if guard.0.is_null() { return Err("the player has been stopped".into()) }
	Ok(guard)
}
fn player_destroy(player: &Shared) {//under the lock, so a call in flight finishes first, and leaving a null behind for any command that was waiting
	let mut guard = player.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	if guard.0.is_null() { return }
	unsafe { mpv_terminate_destroy(guard.0) };
	guard.0 = std::ptr::null_mut();
}

fn text(s: &str) -> Result<CString, String> { CString::new(s).map_err(|_| format!("a string with a null byte in it: {s:?}")) }//c text for the library; a path or an option with a null in it cannot be said in c, and is refused rather than cut short

fn property_text(player: Handle, name: &CStr) -> String {//a property as text, or blank where the player has none by that name
	let text = unsafe { mpv_get_property_string(player, name.as_ptr()) };
	if text.is_null() { return String::new() }
	let owned = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();
	unsafe { mpv_free(text as *mut c_void) };
	owned
}

fn error_text(code: c_int) -> String { unsafe { CStr::from_ptr(mpv_error_string(code)) }.to_string_lossy().into_owned() }//what the library says an error code means
