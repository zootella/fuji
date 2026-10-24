use serde::Serialize;
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_ulong, c_void, CStr, CString};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use tauri::ipc::Response;
use tauri::{command, AppHandle, Manager, State, WebviewWindow};
use crate::run_blocking;

/*
libmpv, the media library beneath fuji's sound and its video: mpv's client API, declared by hand below rather than through a crate or a bindgen. The module is named media for what fuji does with the library; mpv names the library itself, and its functions keep their own names. Fuji calls a handful of its functions, their signatures are fixed by the API's second major version, and the Rust crates over it have gone stale more than once, so the declarations live here, where a reader can check them against client.h in a minute. A handle is a player: mpv_handle in the header and an opaque pointer here, made by mpv_create, given its options as strings, started by mpv_initialize, and ended by mpv_terminate_destroy, which waits for the player's own threads to finish. Every function below may be called from any thread, so a command's body runs on the blocking pool like the rest.

**Four commands are the floor, and they are generic over mpv's own vocabulary.** Start makes a player for the window with the options the page chose; send runs one mpv command with its arguments; read answers one property as text; stop ends the player. The page says what plays, where it draws, when to seek and by how much, in mpv's own names, so a seek or a frame step on the file playing is a send and never a new command here. The player lives in a map in managed state keyed by the window's label, one per window, so Windows and Linux run the map with one entry and nothing branches.

**Two more commands ask a file a question rather than play it,** media_probe and media_frame, each answering one thing about one file from a player of its own; the essay above them says how.

**Every call into the library holds that player's lock for the whole of the call.** The handle is a raw pointer, and a stop that destroyed it while a read was inside the library would be a use after free; so a command takes the player's own mutex, checks the handle is still there, calls, and lets go, and a stop takes the same mutex to destroy and then leaves a null behind, which any command that was waiting for the lock finds and reports. The map's lock is held only to look a player up or put one in, never across a call, so one window's stop never waits on another's read.

**A window that closes takes its player with it,** through media_window_closed, which lib.rs calls as the window is asked to close. The destroy goes to the blocking pool rather than running on the window's thread: on Windows the video draws in a child of fuji's window that mpv made on a thread of its own, and tearing that down can send to the thread that owns the parent, so a destroy that blocked the window's thread while mpv waited on it would never finish. The page stops the player itself when its view goes away; this is for the window the user closes mid-play.

The library is one fuji builds itself from the same recipe on every platform, desktop/libmpv/build.py, with FFmpeg and libplacebo inside it and nothing from anywhere else. On Windows it is libmpv-2.dll, kept in src-tauri/binaries untracked and shipped beside the executable by the resources entry in tauri.windows.conf.json; the link attribute names the dll itself, as raw-dylib, so the linker writes the imports from the declarations below and no import library is made or kept. On the Mac it is libmpv.2.dylib in the same folder, which build.rs names for the linker and tauri.macos.conf.json carries into the bundle as a framework, found at launch by its rpath. On Linux it is libmpv.so.2, built by the same recipe inside the containers, named for the linker by build.rs the same way, and carried by the packages as a resource under the product's folder, where the rpath build.rs gives the binary finds it.

**Off Windows a wid is the surface window.rs made, and the video is drawn through the render API**, four more declarations against render.h, with an essay of their own below.
*/

type Handle = *mut c_void;//mpv_handle, opaque in the header too

#[cfg_attr(target_os = "macos", link(name = "mpv"))]//libmpv.dylib, in the folder build.rs names
#[cfg_attr(target_os = "linux", link(name = "mpv"))]//libmpv.so, in the same folder, and at launch libmpv.so.2 by the rpath build.rs gives the binary
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
	fn mpv_command_ret(ctx: Handle, args: *const *const c_char, result: *mut Node) -> c_int;//one command as mpv_command runs it, its answer written into result as a node the caller frees with mpv_free_node_contents; 0, or an error code below it
	fn mpv_free_node_contents(node: *mut Node);//what the library allocated inside a node it answered, though not the node itself
	fn mpv_wait_event(ctx: Handle, timeout: f64) -> *mut Event;//the player's next event, waiting up to timeout seconds; never null, an event whose id is none when nothing came, owned by the library and good until the next call here on this player
	//the render API, on every platform but Windows, which hands mpv a window instead; the essay below says how the two differ
	#[cfg(not(target_os = "windows"))] fn mpv_render_context_create(res: *mut *mut c_void, mpv: Handle, params: *mut RenderParam) -> c_int;//a renderer for the player's core against the GL context current on this thread; 0, or an error code below it
	#[cfg(not(target_os = "windows"))] fn mpv_render_context_set_update_callback(ctx: *mut c_void, callback: Option<extern "C" fn(*mut c_void)>, cb_ctx: *mut c_void);//called from a thread of mpv's own when a frame is ready, and allowed to call nothing of the library
	#[cfg(not(target_os = "windows"))] fn mpv_render_context_render(ctx: *mut c_void, params: *mut RenderParam) -> c_int;//the current frame into the framebuffer the parameters name, with the GL context current; 0, or an error code below it
	#[cfg(not(target_os = "windows"))] fn mpv_render_context_free(ctx: *mut c_void);//the renderer, with the GL context current, before the player it was made for
}
#[cfg(not(target_os = "windows"))]
#[repr(C)] struct RenderParam { kind: c_int, data: *mut c_void }//mpv_render_param: a type from render.h's enum, 1 the api, 2 the GL init parameters, 3 the framebuffer, 4 the flip, and 0 ending a list, and a pointer to its value
#[cfg(not(target_os = "windows"))]
#[repr(C)] struct OpenglInitParams { get_proc_address: extern "C" fn(*mut c_void, *const c_char) -> *mut c_void, get_proc_address_ctx: *mut c_void }//mpv_opengl_init_params: how the renderer finds each GL function
#[cfg(not(target_os = "windows"))]
#[repr(C)] struct OpenglFbo { fbo: c_int, w: c_int, h: c_int, internal_format: c_int }//mpv_opengl_fbo: the framebuffer to draw into and its size in pixels
#[repr(C)] struct Event { event_id: c_int, error: c_int, reply_userdata: u64, data: *mut c_void }//mpv_event: which event, by the numbers below, an error for a reply, and the event's own data
#[repr(C)] struct EndFile { reason: c_int, error: c_int }//the start of mpv_event_end_file: why the file ended, and the error when the reason is one; the fields after these are never read
#[repr(C)] struct Node { u: NodeValue, format: c_int }//mpv_node: a value, and which of the union's members holds it, by the formats below
#[repr(C)] union NodeValue { string: *mut c_char, flag: c_int, int64: i64, double: f64, list: *mut NodeList, ba: *mut ByteArray }
#[repr(C)] struct NodeList { num: c_int, values: *mut Node, keys: *mut *mut c_char }//mpv_node_list: num values, and for a map num keys beside them
#[repr(C)] struct ByteArray { data: *mut c_void, size: usize }//mpv_byte_array
const EVENT_START_FILE: c_int = 6;//the numbers of client.h's mpv_event_id that the questions wait on
const EVENT_END_FILE: c_int = 7;
const EVENT_FILE_LOADED: c_int = 8;
const EVENT_PLAYBACK_RESTART: c_int = 21;
const END_FILE_ERROR: c_int = 4;//mpv_end_file_reason's error
const FORMAT_INT64: c_int = 4;//and mpv_format's, for reading the node screenshot-raw answers
const FORMAT_NODE_MAP: c_int = 8;
const FORMAT_BYTE_ARRAY: c_int = 9;

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
pub async fn media_version() -> Result<Version, String> {
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
pub async fn media_start(window: WebviewWindow, options: Vec<(String, String)>, players: State<'_, Players>) -> Result<(), String> {
	let map = players.0.clone();
	let label = window.label().to_string();
	run_blocking(move || {
		if let Some(old) = map_take(&map, &label) { player_destroy(&old) }
		#[cfg(not(target_os = "windows"))]
		let (options, surface) = video_surface(options, &label)?;//the wid off Windows is the view window.rs made, drawn through the render API rather than handed to mpv; the essay says how
		let player = unsafe { mpv_create() };
		if player.is_null() { return Err("mpv_create answered null".into()) }
		for (name, value) in &options {
			let set = unsafe { mpv_set_option_string(player, text(name)?.as_ptr(), text(value)?.as_ptr()) };
			if set < 0 { unsafe { mpv_terminate_destroy(player) }; return Err(format!("mpv_set_option_string {name}={value}: {}", error_text(set))) }
		}
		let started = unsafe { mpv_initialize(player) };
		if started < 0 { unsafe { mpv_terminate_destroy(player) }; return Err(format!("mpv_initialize: {}", error_text(started))) }
		#[cfg(not(target_os = "windows"))]
		if let Some((layer, gl)) = surface { if let Err(error) = render_create(player, layer, gl) { unsafe { mpv_terminate_destroy(player) }; return Err(error) } }
		map_lock(&map).insert(label, Arc::new(Mutex::new(Player(player))));
		Ok(())
	}).await
}

/// Run one mpv command on this window's player: its name and then its arguments, as mpv's manual lists them, like loadfile and a path, or cycle and pause; answers when the command has run
#[command]
pub async fn media_send(window: WebviewWindow, args: Vec<String>, players: State<'_, Players>) -> Result<(), String> {
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

/// One property of this window's player as text, by its name in mpv's manual, like time-pos or pause; blank for a property that has no value yet, which time-pos has none of before a file is loaded; and for a property whose value is a structure, like track-list, JSON, which is how mpv prints one as text, so the page reads a file's tracks, their codecs, sizes and channels, with no command of their own
#[command]
pub async fn media_read(window: WebviewWindow, name: String, players: State<'_, Players>) -> Result<String, String> {
	let player = player_of(&players, window.label())?;
	run_blocking(move || {
		let handle = player_handle(&player)?;
		Ok(property_text(handle.0, &text(&name)?))
	}).await
}

/// Stop this window's player and destroy it; nothing, if the window has none
#[command]
pub async fn media_stop(window: WebviewWindow, players: State<'_, Players>) -> Result<(), String> {
	let taken = map_take(&players.0, window.label());
	run_blocking(move || { if let Some(player) = taken { player_destroy(&player) } Ok(()) }).await
}

/// The window with this label is closing: destroy its player, if it has one, on the blocking pool, so the window's thread stays free to answer whatever mpv's own threads ask of it on the way down
pub fn media_window_closed(app: &AppHandle, label: &str) {
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
	#[cfg(not(target_os = "windows"))]
	render_free(guard.0);//its render context first, which the render API wants gone before the player
	unsafe { mpv_terminate_destroy(guard.0) };
	guard.0 = std::ptr::null_mut();
}

/*
The render API, on the Mac and on Linux, where mpv is handed no window.

mpv takes a window to draw in on X11, Windows and Android and nowhere else, so off Windows the square is a GL surface of fuji's own that the platform asks to draw, a Core Animation layer on the Mac and a GTK GL area on Linux, which window.rs makes and the page hands back as the wid. Start takes that wid out of the options and puts vo=libmpv in its place, the output that draws only when asked, and once the player is up makes a render context against the surface's GL context. Every call on that context wants the GL context current on the calling thread, and what current means is window.rs's to arrange behind video_gl_current, since the two platforms differ: the Mac lends its CGL context to the pool under the lock CGL offers, and Linux runs the call on the main thread, which alone may make GTK's context current, with the pool waiting.

A frame ready is said from a thread of mpv's own through the callback, which may call nothing of the library, so it asks window.rs to display the surface, and the platform draws on the main thread with its context current, through video_render. The list below holds each context with its surface, its GL context and its player, and the list's lock is what makes the draws and the free take turns: a render holds it through the draw, a free takes the context out under it and so waits for a draw in flight, and a draw after finds nothing to draw. The free comes before the player's destroy, as the API requires, and on Linux also as the surface is unrealized, since there the context can be current only while the area stands.

video-timing-offset is zero for a player drawing this way. By default mpv announces a frame fifty milliseconds early and the render blocks until the frame's time, which on the main thread would hold the page's events for most of every frame; at zero the frame is announced at its time, the render returns at once, and the platform shows it at the next refresh, within a sixtieth of a second, which render.h offers as the way to do this.
*/
#[cfg(not(target_os = "windows"))]
struct Render { layer: isize, gl: isize, ctx: isize, player: isize }//one render context: the surface it draws, the GL context it was made against, the context itself, and the player it belongs to, as addresses so the list can be shared; on the Mac the surface is the layer and the context CGL's, and on Linux both are the GL area, whose context is GTK's
#[cfg(not(target_os = "windows"))]
static RENDERS: Mutex<Vec<Render>> = Mutex::new(Vec::new());
#[cfg(not(target_os = "windows"))]
fn renders() -> MutexGuard<'static, Vec<Render>> { RENDERS.lock().unwrap_or_else(|p| p.into_inner()) }
#[cfg(not(target_os = "windows"))]
fn video_surface(options: Vec<(String, String)>, label: &str) -> Result<(Vec<(String, String)>, Option<(isize, isize)>), String> {//the wid out and the render API's output in, answering the surface and GL context the render context is made against once the player is up; options with no wid pass untouched
	if !options.iter().any(|(name, _)| name == "wid") { return Ok((options, None)) }
	let Some((layer, gl)) = crate::window::video_gl(label) else { return Err("the window has no video view for the wid it was given".into()) };
	let mut options: Vec<_> = options.into_iter().filter(|(name, _)| name != "wid").collect();
	options.push(("vo".into(), "libmpv".into()));
	options.push(("video-timing-offset".into(), "0".into()));//the essay says why
	Ok((options, Some((layer, gl))))
}
#[cfg(not(target_os = "windows"))]
fn render_create(player: Handle, layer: isize, gl: isize) -> Result<(), String> {//the context made with the GL context current, wherever window.rs runs the call, and listed once it is made, so a frame announced in between finds nothing to draw and the next finds it; the player crosses as an address, since the call may run on another thread
	crate::window::video_gl_retain(gl);//a reference of the render's own, so the context outlives the view if the window goes first
	let player_at = player as isize;
	let made = crate::window::video_gl_current(gl, move || {
		let mut init = OpenglInitParams { get_proc_address: gl_address, get_proc_address_ctx: std::ptr::null_mut() };
		let mut params = [RenderParam { kind: 1, data: c"opengl".as_ptr() as *mut c_void }, RenderParam { kind: 2, data: &mut init as *mut OpenglInitParams as *mut c_void }, RenderParam { kind: 0, data: std::ptr::null_mut() }];
		let mut ctx = std::ptr::null_mut();
		let made = unsafe { mpv_render_context_create(&mut ctx, player_at as Handle, params.as_mut_ptr()) };
		if made < 0 || ctx.is_null() { return Err(format!("mpv_render_context_create: {}", error_text(made))) }
		unsafe { mpv_render_context_set_update_callback(ctx, Some(render_update), layer as *mut c_void) };
		Ok(ctx as isize)
	});
	match made {
		Ok(ctx) => { renders().push(Render { layer, gl, ctx, player: player_at }); Ok(()) }
		Err(error) => { crate::window::video_gl_release(gl); Err(error) }
	}
}
#[cfg(not(target_os = "windows"))]
fn render_free(player: Handle) {//the player's, if it has one: out of the list under its lock, and ended outside it, since the free may wait for the core to take the video down
	let taken = { let mut list = renders(); list.iter().position(|r| r.player == player as isize).map(|at| list.remove(at)) };
	let Some(render) = taken else { return };
	render_end(render)
}
#[cfg(not(target_os = "windows"))]
fn render_end(render: Render) {//a context already out of the list: no frame announced for it again, the free with the GL context current, and the reference given back
	unsafe { mpv_render_context_set_update_callback(render.ctx as *mut c_void, None, std::ptr::null_mut()) };
	let ctx = render.ctx;
	crate::window::video_gl_current(render.gl, move || unsafe { mpv_render_context_free(ctx as *mut c_void) });
	crate::window::video_gl_release(render.gl);
}
/// This surface is going, said by window.rs on the main thread with its GL context current: free whatever draws it now, while there is a context to be current; nothing, for a surface no player draws
#[cfg(target_os = "linux")]
pub fn video_unrealize(layer: isize) {
	let taken = { let mut list = renders(); list.iter().position(|r| r.layer == layer).map(|at| list.remove(at)) };
	if let Some(render) = taken { render_end(render) }
}
#[cfg(not(target_os = "windows"))]
extern "C" fn render_update(layer: *mut c_void) { crate::window::video_display(layer as isize) }//a frame is ready, said from mpv's thread, and nothing of the library may be called from here
#[cfg(target_os = "macos")]
extern "C" fn gl_address(_ctx: *mut c_void, name: *const c_char) -> *mut c_void {//how the renderer finds each GL function: by name in the OpenGL framework, which CGL already loaded into the process; except glFlush, which answers a function that does nothing, since Core Animation flushes the layer's context itself after each draw, the arrangement mpv's own Mac layer makes
	static OPENGL: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
	let framework = *OPENGL.get_or_init(|| unsafe { libc::dlopen(c"/System/Library/Frameworks/OpenGL.framework/OpenGL".as_ptr(), libc::RTLD_LAZY) } as usize);
	if framework == 0 { return std::ptr::null_mut() }
	if unsafe { CStr::from_ptr(name) } == c"glFlush" { return gl_flush_nothing as *mut c_void }
	unsafe { libc::dlsym(framework as *mut c_void, name) }
}
#[cfg(target_os = "macos")]
extern "C" fn gl_flush_nothing() {}
#[cfg(target_os = "linux")]
extern "C" fn gl_address(_ctx: *mut c_void, name: *const c_char) -> *mut c_void { crate::window::video_gl_address(name) }//how the renderer finds each GL function: through the loader of whichever GL the display has, which window.rs found with the display, EGL's on Wayland and GLX's on X11
/// The current frame of the player drawing this surface into the framebuffer the platform bound, at the surface's size in pixels, answering whether there was a player to draw it; called by window.rs on the main thread with the GL context current
#[cfg(not(target_os = "windows"))]
pub fn video_render(layer: isize, framebuffer: i32, width: i32, height: i32) -> bool {
	let list = renders();//held through the draw, so a free waits for it
	let Some(render) = list.iter().find(|r| r.layer == layer) else { return false };
	let mut target = OpenglFbo { fbo: framebuffer, w: width, h: height, internal_format: 0 };
	let mut flip: c_int = 1;//GL's framebuffer has its origin at the bottom, and the frame is drawn the right way up for it
	let mut params = [RenderParam { kind: 3, data: &mut target as *mut OpenglFbo as *mut c_void }, RenderParam { kind: 4, data: &mut flip as *mut c_int as *mut c_void }, RenderParam { kind: 0, data: std::ptr::null_mut() }];
	unsafe { mpv_render_context_render(render.ctx as *mut c_void, params.as_mut_ptr()) };//an error here is a frame not drawn, which the next frame answers
	true
}
/// Put the C library's numeric locale back to C, which mpv_create requires and answers null without: GTK set the whole locale from the environment when the runtime started it, and in a French or a German locale the number parsing mpv and FFmpeg share would read a comma where the code writes a point. Called once on Linux as the application sets up, on the main thread before any player is made; on Windows and the Mac nothing sets the locale and it stays C
#[cfg(target_os = "linux")]
pub fn media_locale() { unsafe { libc::setlocale(libc::LC_NUMERIC, c"C".as_ptr()) }; }

/*
Asking a file a question rather than playing it: its running time, its format and its tracks, or one frame of it.

Each question gets a player of its own, made for it and destroyed after, so a window's playing player is never touched, nothing is kept between calls, and the page holds no state for the asking; making and destroying a player cost about a millisecond and a half a file, measured on the Mac mini on 2026-10-15, beside the file's own reading. Every one is started with the options that keep a file from reaching past itself, the list below: FFmpeg's demuxers alone, so mpv's own readers for Matroska, playlists, edit lists, cue sheets and mf addresses never see it; no references to other files, no segments from other files in its folder, and no subtitles or cover art read from beside it; and no saved position looked for.

A load finishes on mpv's own time, so a question waits on the player's events up to the deadline the page gives: start-file says the load has begun, then file-loaded that the file opened or end-file that it did not, and for a frame playback-restart that the frame at the time asked is decoded. The deadline covers those waits. A file that gives no answer by it is answered with an error, and its player is destroyed on another thread of the pool, since mpv_terminate_destroy waits for the player's own threads, and one stuck in a read on a dead disk would hold the answer too; that thread waits as long as it must, which is containment rather than cancellation.
*/
const ASK_ALWAYS: [(&str, &str); 5] = [//the options every question's player is started with
	("demuxer", "lavf"),//FFmpeg's demuxers alone
	("access-references", "no"),//no references to other files
	("ordered-chapters", "no"),//no segments loaded from other files in the folder
	("autoload-files", "no"),//no subtitle files or cover art read from the folder
	("resume-playback", "no"),//no saved position looked for
];

#[derive(Serialize)]
pub struct Probe {
	pub(crate) duration: String,//the running time in seconds as mpv writes it, like 5.000000, or blank for a file with none
	pub(crate) format: String,//the demuxer's name for what the bytes are, like mov,mp4,m4a,3gp,3g2,mj2
	pub(crate) tracks: String,//the track list as mpv's own JSON: each track's type and codec, and for a video track whichever of demux-w, demux-h, demux-par and demux-rotation the demuxer filled in from the file's headers; MPEG-4 Part 2 inside MP4 and MOV comes with no size, where the same codec in Matroska and AVI and H.264 in MP4 come with theirs
}

/// What the media library makes of a file without playing it: its running time, its format and its tracks, from a player made for the question that selects no track, so no decoder starts and nothing is drawn; an error for a file it cannot open, and for one that gives no answer within seconds
#[command]
pub async fn media_probe(path: String, seconds: f64) -> Result<Probe, String> {
	run_blocking(move || probe(&path, seconds)).await
}
pub(crate) fn probe(path: &str, seconds: f64) -> Result<Probe, String> {//the question itself, for a body already on the blocking pool, like the inventory's
	let deadline = ask_deadline(seconds)?;
	let player = ask_player(&[("track-auto-selection", "no")])?;//every track left unselected: the file opens and waits with nothing decoded, where with video and audio both turned off it would end as it loaded
	if let Err(ask) = ask_load(&player, path, deadline, &[EVENT_START_FILE, EVENT_FILE_LOADED]) { return Err(ask_end(player, ask, seconds)) }
	let probe = Probe { duration: property_text(player.0, c"duration"), format: property_text(player.0, c"file-format"), tracks: property_text(player.0, c"track-list") };
	unsafe { mpv_terminate_destroy(player.0) };
	Ok(probe)
}

/// One frame of a video at a time in seconds, as one buffer: its width and height as little-endian unsigned 32-bit numbers, then its pixels as RGBA, row after row, at the shape the video shows at, with its turn and the stretch of pixels that are not square already applied. From a player made for the question, paused, decoding in software and drawing nowhere, which seeks exactly to the time as it opens the file; an error for a file with no video, and for one that gives no answer within seconds
#[command]
pub async fn media_frame(path: String, at: f64, seconds: f64) -> Result<Response, String> {
	run_blocking(move || {
		if !(at.is_finite() && at >= 0.0) { return Err(format!("media_frame: expected a time in seconds, zero or more: {at}")) }
		let deadline = ask_deadline(seconds)?;
		let start = format!("{at}");
		let player = ask_player(&[("aid", "no"), ("sid", "no"), ("vo", "null"), ("pause", "yes"), ("hwdec", "no"), ("hr-seek", "yes"), ("start", start.as_str())])?;//the video alone, drawn nowhere, the null output keeping the current frame for screenshot-raw; paused, so the player decodes the frame at start and stops there, and hr-seek makes that the frame at the time rather than the keyframe before it
		if let Err(ask) = ask_load(&player, &path, deadline, &[EVENT_START_FILE, EVENT_FILE_LOADED, EVENT_PLAYBACK_RESTART]) { return Err(ask_end(player, ask, seconds)) }
		let frame = ask_screenshot(player.0);
		unsafe { mpv_terminate_destroy(player.0) };
		Ok(Response::new(frame?))
	}).await
}

enum Ask { Refused(String), Late }//how a question ended without an answer: the file refused, with the library's reason, or no answer by the deadline

fn ask_deadline(seconds: f64) -> Result<Instant, String> {//the moment a question gives up, from the seconds the page allows it
	if !(seconds.is_finite() && seconds > 0.0) { return Err(format!("expected a deadline in seconds above zero: {seconds}")) }
	Ok(Instant::now() + Duration::from_secs_f64(seconds))
}

fn ask_player(options: &[(&str, &str)]) -> Result<Player, String> {//a player for one question, started with the options every question has and then this one's own
	let named = ASK_ALWAYS.iter().chain(options).map(|(name, value)| Ok((text(name)?, text(value)?))).collect::<Result<Vec<_>, String>>()?;//all of them as c text before the player exists, so a bad one leaves nothing to destroy
	let player = unsafe { mpv_create() };
	if player.is_null() { return Err("mpv_create answered null".into()) }
	for (name, value) in &named {
		let set = unsafe { mpv_set_option_string(player, name.as_ptr(), value.as_ptr()) };
		if set < 0 { unsafe { mpv_terminate_destroy(player) }; return Err(format!("mpv_set_option_string {name:?}={value:?}: {}", error_text(set))) }
	}
	let started = unsafe { mpv_initialize(player) };
	if started < 0 { unsafe { mpv_terminate_destroy(player) }; return Err(format!("mpv_initialize: {}", error_text(started))) }
	Ok(Player(player))
}

fn ask_load(player: &Player, path: &str, deadline: Instant, steps: &[c_int]) -> Result<(), Ask> {//load the file, then wait for each event of steps in order, each before the deadline; an end-file along the way is the file refused
	let path = text(path).map_err(Ask::Refused)?;
	let args = [c"loadfile".as_ptr(), path.as_ptr(), std::ptr::null()];
	let ran = unsafe { mpv_command(player.0, args.as_ptr()) };//answers once the load is queued, not once it is done
	if ran < 0 { return Err(Ask::Refused(format!("loadfile: {}", error_text(ran)))) }
	for &step in steps {
		loop {
			let left = deadline.saturating_duration_since(Instant::now()).as_secs_f64();
			if left <= 0.0 { return Err(Ask::Late) }
			let event = unsafe { &*mpv_wait_event(player.0, left) };
			if event.event_id == step { break }
			if event.event_id == EVENT_END_FILE {
				let end = unsafe { &*(event.data as *const EndFile) };
				if end.reason == END_FILE_ERROR { return Err(Ask::Refused(format!("the media library could not open it: {}", error_text(end.error)))) }
				return Err(Ask::Refused("the file ended before it answered".into()))
			}
		}
	}
	Ok(())
}

fn ask_end(player: Player, ask: Ask, seconds: f64) -> String {//the question's player gone, and the reason it has no answer: destroyed here for a file refused, which leaves the player idle, and on another thread of the pool for one late, since the destroy waits for whatever the player's threads are stuck in
	match ask {
		Ask::Refused(reason) => { unsafe { mpv_terminate_destroy(player.0) }; reason }
		Ask::Late => {
			tauri::async_runtime::spawn_blocking(move || { let player = player; unsafe { mpv_terminate_destroy(player.0) } });//the whole player moved in, rather than the raw handle inside it, which could not cross threads
			format!("no answer within {seconds} seconds")
		}
	}
}

fn ask_screenshot(player: Handle) -> Result<Vec<u8>, String> {//the decoded frame through screenshot-raw, with nothing drawn on it, as RGBA, packed into the buffer media_frame answers. Its video mode applies the file's turn and pixel shape: a DVD-style clip stored at 720 by 480 came back 853 by 480, and a video stored at 320 by 240 with a quarter turn came back 240 by 320, on the Mac mini on 2026-10-16
	let args = [c"screenshot-raw".as_ptr(), c"video".as_ptr(), c"rgba".as_ptr(), std::ptr::null()];
	let mut node = Node { u: NodeValue { int64: 0 }, format: 0 };
	let ran = unsafe { mpv_command_ret(player, args.as_ptr(), &mut node) };
	if ran < 0 { return Err(format!("screenshot-raw: {}", error_text(ran))) }
	let packed = node_frame(&node);//copied out before the node is freed, since the pixels are the library's until then
	unsafe { mpv_free_node_contents(&mut node) };
	packed
}

fn node_frame(node: &Node) -> Result<Vec<u8>, String> {//the map screenshot-raw answers, w, h, stride and data, checked and packed: every size in it comes from the file in the end, so each is checked against the bytes really there before a row is copied
	if node.format != FORMAT_NODE_MAP { return Err("screenshot-raw answered something other than a map".into()) }
	let list = unsafe { &*node.u.list };
	let (mut w, mut h, mut stride, mut data): (i64, i64, i64, &[u8]) = (0, 0, 0, &[]);
	for i in 0..usize::try_from(list.num).unwrap_or(0) {
		let key = unsafe { CStr::from_ptr(*list.keys.add(i)) }.to_bytes();
		let value = unsafe { &*list.values.add(i) };
		match (key, value.format) {
			(b"w", FORMAT_INT64) => w = unsafe { value.u.int64 },
			(b"h", FORMAT_INT64) => h = unsafe { value.u.int64 },
			(b"stride", FORMAT_INT64) => stride = unsafe { value.u.int64 },
			(b"data", FORMAT_BYTE_ARRAY) => {
				let bytes = unsafe { &*value.u.ba };
				if !bytes.data.is_null() { data = unsafe { std::slice::from_raw_parts(bytes.data as *const u8, bytes.size) } }
			}
			_ => {}
		}
	}
	let too_big = || format!("screenshot-raw answered a frame of {w} by {h} with a stride of {stride} in {} bytes", data.len());
	let (Ok(width), Ok(height), Ok(stride)) = (u32::try_from(w), u32::try_from(h), usize::try_from(stride)) else { return Err(too_big()) };
	if width == 0 || height == 0 { return Err(too_big()) }
	let row = (width as usize).checked_mul(4).ok_or_else(too_big)?;
	let need = stride.checked_mul(height as usize - 1).and_then(|n| n.checked_add(row)).ok_or_else(too_big)?;//the last row needs only its own pixels, not a whole stride
	if stride < row || data.len() < need { return Err(too_big()) }
	let mut packed = Vec::with_capacity(row.checked_mul(height as usize).and_then(|n| n.checked_add(8)).ok_or_else(too_big)?);
	packed.extend_from_slice(&width.to_le_bytes());
	packed.extend_from_slice(&height.to_le_bytes());
	for y in 0..height as usize { packed.extend_from_slice(&data[y * stride..y * stride + row]) }//within data, by the check on need above
	Ok(packed)
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
