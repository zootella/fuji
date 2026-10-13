use serde::Serialize;
use std::ffi::{c_char, c_int, c_ulong, c_void, CStr};
use tauri::command;
use crate::run_blocking;

/*
libmpv, the media library beneath fuji's sound and, later, its video: mpv's client API, declared by hand below rather than through a crate or a bindgen. Fuji calls a handful of its functions, their signatures are fixed by the API's second major version, and the Rust crates over it have gone stale more than once, so the declarations live here, where a reader can check them against client.h in a minute. A handle is a player: mpv_handle in the header and an opaque pointer here, made by mpv_create, given its options as strings, started by mpv_initialize, and ended by mpv_terminate_destroy, which waits for the player's own threads to finish. Every function below may be called from any thread, so a command's body runs on the blocking pool like the rest.

On the Mac the library is Homebrew's for now, gathered with everything under it into a folder build.rs names, by frameworks.js, which also says how the copies travel into the bundle. On Windows it is one dll, libmpv-2.dll, built from the tagged releases by pnpm libmpv with FFmpeg and libplacebo inside it, kept in src-tauri/binaries untracked and shipped beside the executable by the resources entry in tauri.windows.conf.json; the link attribute names the dll itself, as raw-dylib, so the linker writes the imports from the declarations below and no import library is made or kept. Where the library comes from on each platform in the end is the video spike's subject. On Linux nothing links yet, so lib.rs compiles a stand-in that says so.
*/

type Handle = *mut c_void;//mpv_handle, opaque in the header too

#[cfg_attr(target_os = "macos", link(name = "mpv"))]//libmpv.dylib, in the folder build.rs names
#[cfg_attr(target_os = "windows", link(name = "libmpv-2", kind = "raw-dylib"))]//libmpv-2.dll beside the executable, found by the loader at launch with no import library
extern "C" {
	fn mpv_client_api_version() -> c_ulong;//the api version the library was built with, major in the high sixteen bits and minor in the low
	fn mpv_create() -> Handle;//a player with no options set and not yet started, or null
	fn mpv_initialize(ctx: Handle) -> c_int;//start the player; 0, or an error code below it
	fn mpv_get_property_string(ctx: Handle, name: *const c_char) -> *mut c_char;//a property as text, to give back with mpv_free, or null for a name the player has no property by
	fn mpv_free(data: *mut c_void);//what the library allocated for the caller
	fn mpv_error_string(error: c_int) -> *const c_char;//what an error code means, in text the library keeps
	fn mpv_terminate_destroy(ctx: Handle);//stop the player and free it, once its threads have finished
}

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

fn property_text(player: Handle, name: &CStr) -> String {//a property as text, or blank where the player has none by that name
	let text = unsafe { mpv_get_property_string(player, name.as_ptr()) };
	if text.is_null() { return String::new() }
	let owned = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();
	unsafe { mpv_free(text as *mut c_void) };
	owned
}

fn error_text(code: c_int) -> String { unsafe { CStr::from_ptr(mpv_error_string(code)) }.to_string_lossy().into_owned() }//what the library says an error code means
