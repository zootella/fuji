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
pub fn mpv_locale() { unsafe { libc::setlocale(libc::LC_NUMERIC, c"C".as_ptr()) }; }

fn text(s: &str) -> Result<CString, String> { CString::new(s).map_err(|_| format!("a string with a null byte in it: {s:?}")) }//c text for the library; a path or an option with a null in it cannot be said in c, and is refused rather than cut short

fn property_text(player: Handle, name: &CStr) -> String {//a property as text, or blank where the player has none by that name
	let text = unsafe { mpv_get_property_string(player, name.as_ptr()) };
	if text.is_null() { return String::new() }
	let owned = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();
	unsafe { mpv_free(text as *mut c_void) };
	owned
}

fn error_text(code: c_int) -> String { unsafe { CStr::from_ptr(mpv_error_string(code)) }.to_string_lossy().into_owned() }//what the library says an error code means
