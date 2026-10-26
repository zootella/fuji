use std::collections::HashMap;
use std::ffi::c_char;
#[cfg(not(target_os = "windows"))]
use std::ffi::{c_int, c_void};
#[cfg(target_os = "macos")]
use std::ffi::CStr;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::ipc::Response;
use tauri::{command, AppHandle, Manager, State, WebviewWindow};
use crate::run_blocking;
use copper::media::{error_text, mpv_command, mpv_create, mpv_initialize, mpv_set_option_string, mpv_terminate_destroy, property_text, text, Player};//the library's declarations and helpers, which copper holds; the questions asked of a file without a window are there too, and the commands below hand them their arguments
#[cfg(not(target_os = "windows"))]
use copper::media::{mpv_render_context_create, mpv_render_context_free, mpv_render_context_render, mpv_render_context_set_update_callback, Handle, OpenglFbo, OpenglInitParams, RenderParam};

/*

**Four commands are the floor, and they are generic over mpv's own vocabulary.** Start makes a player for the window with the options the page chose; send runs one mpv command with its arguments; read answers one property as text; stop ends the player. The page says what plays, where it draws, when to seek and by how much, in mpv's own names, so a seek or a frame step on the file playing is a send and never a new command here. The player lives in a map in managed state keyed by the window's label, one per window, so Windows and Linux run the map with one entry and nothing branches.

**Two more commands ask a file a question rather than play it,** media_probe and media_frame, each answering one thing about one file from a player of its own; the essay above the questions in copper's media.rs says how. The declarations against the library's header live there too, with the questions, because neither needs a window; this file holds what does: the players, one per window, and the render contexts that draw into fuji's own surfaces.

**Every call into the library holds that player's lock for the whole of the call.** The handle is a raw pointer, and a stop that destroyed it while a read was inside the library would be a use after free; so a command takes the player's own mutex, checks the handle is still there, calls, and lets go, and a stop takes the same mutex to destroy and then leaves a null behind, which any command that was waiting for the lock finds and reports. The map's lock is held only to look a player up or put one in, never across a call, so one window's stop never waits on another's read.

**A window that closes takes its player with it,** through media_window_closed, which lib.rs calls as the window is asked to close. The destroy goes to the blocking pool rather than running on the window's thread: on Windows the video draws in a child of fuji's window that mpv made on a thread of its own, and tearing that down can send to the thread that owns the parent, so a destroy that blocked the window's thread while mpv waited on it would never finish. The page stops the player itself when its view goes away; this is for the window the user closes mid-play.

**Off Windows a wid is the surface window.rs made, and the video is drawn through the render API**, four more declarations against render.h among copper's, with an essay of their own below.
*/

type Shared = Arc<Mutex<Player>>;//a player as the map holds it and a command takes it out: shared, so the map's lock is let go before the call, and locked, so the call and a stop take turns
type Map = Arc<Mutex<HashMap<String, Shared>>>;

#[derive(Default)]
pub struct Players(Map);//the players, one per window by its label, in managed state

/// The media library's versions, read from a player made for the purpose and destroyed after: its own, the client api it was built against, and the FFmpeg beneath it
#[command]
pub async fn media_version() -> Result<copper::media::Version, String> {
	run_blocking(copper::media::version).await
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

/// What the media library makes of a file without playing it: its running time, its format and its tracks, from a player made for the question that selects no track, so no decoder starts and nothing is drawn; an error for a file it cannot open, and for one that gives no answer within seconds
#[command]
pub async fn media_probe(path: String, seconds: f64) -> Result<copper::media::Probe, String> {
	run_blocking(move || copper::media::probe(&path, seconds)).await
}

/// One frame of a video at a time in seconds, as one buffer: its width and height as little-endian unsigned 32-bit numbers, then its pixels as RGBA, row after row, at the shape the video shows at, with its turn and the stretch of pixels that are not square already applied. From a player made for the question, paused, decoding in software and drawing nowhere, which seeks exactly to the time as it opens the file; an error for a file with no video, and for one that gives no answer within seconds
#[command]
pub async fn media_frame(path: String, at: f64, seconds: f64) -> Result<Response, String> {
	run_blocking(move || copper::media::frame(&path, at, seconds).map(Response::new)).await
}

