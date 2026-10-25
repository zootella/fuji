//the media library, libmpv: mpv's client api declared by hand, and the questions fuji asks a file through it without a window, the probe and the frame, each from a player made for the question; the players that draw in a window are brass's, in its media.rs, over these declarations

use serde::Serialize;
use std::ffi::{c_char, c_int, c_ulong, c_void, CStr, CString};
use std::time::{Duration, Instant};

/*
libmpv, the media library beneath fuji's sound and its video: mpv's client API, declared by hand below rather than through a crate or a bindgen. The module is named media for what fuji does with the library; mpv names the library itself, and its functions keep their own names. Fuji calls a handful of its functions, their signatures are fixed by the API's second major version, and the Rust crates over it have gone stale more than once, so the declarations live here, where a reader can check them against client.h in a minute. A handle is a player: mpv_handle in the header and an opaque pointer here, made by mpv_create, given its options as strings, started by mpv_initialize, and ended by mpv_terminate_destroy, which waits for the player's own threads to finish. Every function below may be called from any thread, so a command's body runs on the blocking pool like the rest.

The library is one fuji builds itself from the same recipe on every platform, desktop/libmpv/build.py, with FFmpeg and libplacebo inside it and nothing from anywhere else. On Windows it is libmpv-2.dll, kept in src-tauri/binaries untracked and shipped beside the executable by the resources entry in tauri.windows.conf.json; the link attribute names the dll itself, as raw-dylib, so the linker writes the imports from the declarations below and no import library is made or kept. On the Mac it is libmpv.2.dylib in the same folder, which build.rs names for the linker and tauri.macos.conf.json carries into the bundle as a framework, found at launch by its rpath. On Linux it is libmpv.so.2, built by the same recipe inside the containers, named for the linker by build.rs the same way, and carried by the packages as a resource under the product's folder, where the rpath build.rs gives the binary finds it.
*/

pub type Handle = *mut c_void;//mpv_handle, opaque in the header too

#[cfg_attr(target_os = "macos", link(name = "mpv"))]//libmpv.dylib, in the folder build.rs names
#[cfg_attr(target_os = "linux", link(name = "mpv"))]//libmpv.so, in the same folder, and at launch libmpv.so.2 by the rpath build.rs gives the binary
#[cfg_attr(target_os = "windows", link(name = "libmpv-2", kind = "raw-dylib"))]//libmpv-2.dll beside the executable, found by the loader at launch with no import library
extern "C" {
	fn mpv_client_api_version() -> c_ulong;//the api version the library was built with, major in the high sixteen bits and minor in the low
	pub fn mpv_create() -> Handle;//a player with no options set and not yet started, or null
	pub fn mpv_set_option_string(ctx: Handle, name: *const c_char, data: *const c_char) -> c_int;//one option as text, before mpv_initialize; 0, or an error code below it
	pub fn mpv_initialize(ctx: Handle) -> c_int;//start the player; 0, or an error code below it
	pub fn mpv_command(ctx: Handle, args: *const *const c_char) -> c_int;//one command, its name and arguments as a null-terminated array of strings, run before this returns; 0, or an error code below it
	fn mpv_get_property_string(ctx: Handle, name: *const c_char) -> *mut c_char;//a property as text, to give back with mpv_free, or null for a name the player has no property by
	fn mpv_free(data: *mut c_void);//what the library allocated for the caller
	fn mpv_error_string(error: c_int) -> *const c_char;//what an error code means, in text the library keeps
	pub fn mpv_terminate_destroy(ctx: Handle);//stop the player and free it, once its threads have finished
	fn mpv_command_ret(ctx: Handle, args: *const *const c_char, result: *mut Node) -> c_int;//one command as mpv_command runs it, its answer written into result as a node the caller frees with mpv_free_node_contents; 0, or an error code below it
	fn mpv_free_node_contents(node: *mut Node);//what the library allocated inside a node it answered, though not the node itself
	fn mpv_wait_event(ctx: Handle, timeout: f64) -> *mut Event;//the player's next event, waiting up to timeout seconds; never null, an event whose id is none when nothing came, owned by the library and good until the next call here on this player
	//the render API, on every platform but Windows, which hands mpv a window instead; the essay below says how the two differ
	#[cfg(not(target_os = "windows"))] pub fn mpv_render_context_create(res: *mut *mut c_void, mpv: Handle, params: *mut RenderParam) -> c_int;//a renderer for the player's core against the GL context current on this thread; 0, or an error code below it
	#[cfg(not(target_os = "windows"))] pub fn mpv_render_context_set_update_callback(ctx: *mut c_void, callback: Option<extern "C" fn(*mut c_void)>, cb_ctx: *mut c_void);//called from a thread of mpv's own when a frame is ready, and allowed to call nothing of the library
	#[cfg(not(target_os = "windows"))] pub fn mpv_render_context_render(ctx: *mut c_void, params: *mut RenderParam) -> c_int;//the current frame into the framebuffer the parameters name, with the GL context current; 0, or an error code below it
	#[cfg(not(target_os = "windows"))] pub fn mpv_render_context_free(ctx: *mut c_void);//the renderer, with the GL context current, before the player it was made for
}
#[cfg(not(target_os = "windows"))]
#[repr(C)] pub struct RenderParam { pub kind: c_int, pub data: *mut c_void }//mpv_render_param: a type from render.h's enum, 1 the api, 2 the GL init parameters, 3 the framebuffer, 4 the flip, and 0 ending a list, and a pointer to its value
#[cfg(not(target_os = "windows"))]
#[repr(C)] pub struct OpenglInitParams { pub get_proc_address: extern "C" fn(*mut c_void, *const c_char) -> *mut c_void, pub get_proc_address_ctx: *mut c_void }//mpv_opengl_init_params: how the renderer finds each GL function
#[cfg(not(target_os = "windows"))]
#[repr(C)] pub struct OpenglFbo { pub fbo: c_int, pub w: c_int, pub h: c_int, pub internal_format: c_int }//mpv_opengl_fbo: the framebuffer to draw into and its size in pixels
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

pub struct Player(pub Handle);//one running player: its handle, or null once it has been destroyed, which a command that was waiting for the lock finds
unsafe impl Send for Player {}//the pointer may move between threads because every function above may be called from any thread; the mutex around it is what keeps the calls one at a time

#[derive(Serialize)]
pub struct Version {
	mpv: String,//the library's own version text, which names itself first, like mpv v0.41.0
	api: String,//the client api it was built against, major.minor
	ffmpeg: String,//the FFmpeg it runs on, which decodes everything it plays
}

/// The media library's versions, read from a player made for the purpose and destroyed after: its own, the client api it was built against, and the FFmpeg beneath it
pub fn version() -> Result<Version, String> {
	let api = unsafe { mpv_client_api_version() };
	let player = unsafe { mpv_create() };
	if player.is_null() { return Err("mpv_create answered null".into()) }
	let started = unsafe { mpv_initialize(player) };
	if started < 0 { unsafe { mpv_terminate_destroy(player) }; return Err(format!("mpv_initialize: {}", error_text(started))) }
	let mpv = property_text(player, c"mpv-version");
	let ffmpeg = property_text(player, c"ffmpeg-version");
	unsafe { mpv_terminate_destroy(player) };
	Ok(Version { mpv, api: format!("{}.{}", api >> 16, api & 0xffff), ffmpeg })
}

/// Put the C library's numeric locale back to C, which mpv_create requires and answers null without: GTK set the whole locale from the environment when the runtime started it, and in a French or a German locale the number parsing mpv and FFmpeg share would read a comma where the code writes a point. Called once on Linux as the application sets up, on the main thread before any player is made; on Windows and the Mac nothing sets the locale and it stays C
#[cfg(target_os = "linux")]
pub fn locale() { unsafe { libc::setlocale(libc::LC_NUMERIC, c"C".as_ptr()) }; }

/*
Asking a file a question rather than playing it: its running time, its format and its tracks, or one frame of it.

Each question gets a player of its own, made for it and destroyed after, so a window's playing player is never touched, nothing is kept between calls, and the page holds no state for the asking; making and destroying a player cost about a millisecond and a half a file, measured on the Mac mini on 2026-10-15, beside the file's own reading. Every one is started with the options that keep a file from reaching past itself, the list below; with them an ffconcat file and an HLS playlist came back unrecognized whether the files they name sat in their folder or outside it, measured on the Mac mini on 2026-10-24: FFmpeg's demuxers alone, so mpv's own readers for Matroska, playlists, edit lists, cue sheets and mf addresses never see it; no references to other files, no segments from other files in its folder, and no subtitles or cover art read from beside it; and no saved position looked for.

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
	pub duration: String,//the running time in seconds as mpv writes it, like 5.000000, or blank for a file with none
	pub format: String,//the demuxer's name for what the bytes are, like mov,mp4,m4a,3gp,3g2,mj2
	pub tracks: String,//the track list as mpv's own JSON: each track's type and codec, and for a video track whichever of demux-w, demux-h, demux-par and demux-rotation the demuxer filled in from the file's headers; MPEG-4 Part 2 inside MP4 and MOV comes with no size, where the same codec in Matroska and AVI and H.264 in MP4 come with theirs
}

pub fn probe(path: &str, seconds: f64) -> Result<Probe, String> {//the question media_probe asks, for a body already on a thread that may wait
	let deadline = ask_deadline(seconds)?;
	let player = ask_player(&[("track-auto-selection", "no")])?;//every track left unselected: the file opens and waits with nothing decoded, where with video and audio both turned off it would end as it loaded
	if let Err(ask) = ask_load(&player, path, deadline, &[EVENT_START_FILE, EVENT_FILE_LOADED]) { return Err(ask_end(player, ask, seconds)) }
	let probe = Probe { duration: property_text(player.0, c"duration"), format: property_text(player.0, c"file-format"), tracks: property_text(player.0, c"track-list") };
	unsafe { mpv_terminate_destroy(player.0) };
	Ok(probe)
}

/*
A frame of an MPEG program stream or transport stream file does not come back today, and the finding is kept here until it is run down. Asked for a frame of steel's video.ts, dvd-16x9.vob, dvd-4x3.vob and vcd.mpg, at 0, 0.1 and 0.5 seconds, the player reaches playback-restart and screenshot-raw then answers "error running command", which frame returns as its refusal, while probe answers all four with their format, codec and running time, and video.mpg, an MPEG-1 program stream, gives its frame at every one of those times. video.rm gives its first frame and refuses the two later times the same way. Measured on the Mac mini on 2026-10-24 with the library the recipe builds. These are whole files on disk, as every file fuji reads is; the word stream is the container's name and nothing more. What leaves the output without a frame, the seek to the asked time, the paused start or the null output, was not determined, and nothing was changed: today only the test panel asks for a frame, and when a fix is tried the four files above are its test.
*/
/// One frame of a video at a time in seconds, as one buffer: its width and height as little-endian unsigned 32-bit numbers, then its pixels as RGBA, row after row, at the shape the video shows at, with its turn and the stretch of pixels that are not square already applied; from a player made for the question, paused, decoding in software and drawing nowhere, which seeks exactly to the time as it opens the file. An error for a file with no video, and for one that gives no answer within seconds
pub fn frame(path: &str, at: f64, seconds: f64) -> Result<Vec<u8>, String> {
	if !(at.is_finite() && at >= 0.0) { return Err(format!("media_frame: expected a time in seconds, zero or more: {at}")) }
	let deadline = ask_deadline(seconds)?;
	let start = format!("{at}");
	let player = ask_player(&[("aid", "no"), ("sid", "no"), ("vo", "null"), ("pause", "yes"), ("hwdec", "no"), ("hr-seek", "yes"), ("start", start.as_str())])?;//the video alone, drawn nowhere, the null output keeping the current frame for screenshot-raw; paused, so the player decodes the frame at start and stops there, and hr-seek makes that the frame at the time rather than the keyframe before it
	if let Err(ask) = ask_load(&player, path, deadline, &[EVENT_START_FILE, EVENT_FILE_LOADED, EVENT_PLAYBACK_RESTART]) { return Err(ask_end(player, ask, seconds)) }
	let frame = ask_screenshot(player.0);
	unsafe { mpv_terminate_destroy(player.0) };
	frame
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

fn ask_end(player: Player, ask: Ask, seconds: f64) -> String {//the question's player gone, and the reason it has no answer: destroyed here for a file refused, which leaves the player idle, and on a thread of its own for one late, since the destroy waits for whatever the player's threads are stuck in
	match ask {
		Ask::Refused(reason) => { unsafe { mpv_terminate_destroy(player.0) }; reason }
		Ask::Late => {
			std::thread::spawn(move || { let player = player; unsafe { mpv_terminate_destroy(player.0) } });//the whole player moved in, rather than the raw handle inside it, which could not cross threads
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

pub fn text(s: &str) -> Result<CString, String> { CString::new(s).map_err(|_| format!("a string with a null byte in it: {s:?}")) }//c text for the library; a path or an option with a null in it cannot be said in c, and is refused rather than cut short

pub fn property_text(player: Handle, name: &CStr) -> String {//a property as text, or blank where the player has none by that name
	let text = unsafe { mpv_get_property_string(player, name.as_ptr()) };
	if text.is_null() { return String::new() }
	let owned = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();
	unsafe { mpv_free(text as *mut c_void) };
	owned
}

pub fn error_text(code: c_int) -> String { unsafe { CStr::from_ptr(mpv_error_string(code)) }.to_string_lossy().into_owned() }//what the library says an error code means

#[cfg(test)]
mod tests {
	use super::*;
	fn sample(name: &str) -> String { format!("{}/../steel/samples/{name}", env!("CARGO_MANIFEST_DIR")) }
	#[test]
	fn the_library_says_its_versions() {
		let v = version().unwrap();
		assert!(v.mpv.starts_with("mpv ") && v.api.starts_with("2.") && v.ffmpeg.contains('.'), "{} {} {}", v.mpv, v.api, v.ffmpeg);
	}
	#[test]
	fn a_frame_comes_back_at_the_shape_the_video_shows_at() {//the first frame of a 32 by 20 sample, decoded in software and drawn nowhere: the header's two sizes, then that many RGBA pixels
		let bytes = frame(concat!(env!("CARGO_MANIFEST_DIR"), "/../steel/samples/taken.mp4"), 0.0, 10.0).unwrap();
		let (w, h) = (u32::from_le_bytes(bytes[0..4].try_into().unwrap()), u32::from_le_bytes(bytes[4..8].try_into().unwrap()));
		assert_eq!((w, h, bytes.len()), (32, 20, 8 + 32 * 20 * 4));
		assert!(frame(concat!(env!("CARGO_MANIFEST_DIR"), "/../steel/samples/audio.wav"), 0.0, 10.0).is_err(), "a sound file has no frame");
		let shape = |name: &str| frame(&sample(name), 0.0, 10.0).map(|b| (u32::from_le_bytes(b[0..4].try_into().unwrap()), u32::from_le_bytes(b[4..8].try_into().unwrap())));
		assert_eq!(shape("ccw-90.mp4").unwrap(), (20, 32), "a phone's quarter turn, applied");
		assert_eq!(shape("dvd-16x9.mp4").unwrap(), (853, 480), "a DVD's pixels are not square, so its 720 by 480 shows at 853");
		for (name, size) in [("video.avi", (32, 20)), ("video.flv", (32, 20)), ("video.wmv", (32, 20)), ("video.rm", (32, 20)), ("video.mov", (32, 20)), ("video.mkv", (32, 20)), ("video.webm", (32, 20)), ("video.mpg", (32, 20)), ("dvd-16x9.webm", (853, 480)), ("animated.gif", (32, 20))] { assert_eq!(shape(name).unwrap(), size, "{name}") }//one sample a codec, mpeg4, flv1, wmv2, rv20, vp8, vp9, mpeg1video and gif: the probe below names a codec from the container alone, and only a decode proves the recipe built its decoder
	}
	#[test]
	fn a_file_the_library_cannot_open_is_refused_with_its_reason() {
		let refused = probe(concat!(env!("CARGO_MANIFEST_DIR"), "/../steel/samples/text.txt"), 10.0).err().unwrap();
		assert!(refused.contains("could not open"), "{refused}");
		assert!(probe("/no/such/file.mp4", 10.0).is_err());
		assert!(probe(concat!(env!("CARGO_MANIFEST_DIR"), "/../steel/samples/video.ts"), 0.0).is_err(), "a deadline of zero is a caller's mistake");
		assert!(probe("a\0b.mp4", 10.0).err().unwrap().contains("null byte"), "a path c cannot say is refused rather than cut short");
		assert!(frame(&sample("video.mkv"), -1.0, 10.0).is_err() && frame(&sample("video.mkv"), f64::NAN, 10.0).is_err(), "a time before the start, or no time");
	}

	#[test]
	fn the_probe_opens_every_container_a_collection_holds() {//one sample of each container the type table calls video or audio, against the demuxer's name, the codec it names and the running time FFmpeg gives, through the library fuji builds from its own recipe: a row failing here is a demuxer the recipe left out, which is the one way that library can lose a format the system's own players have. The running time is held to a hundredth, so a new FFmpeg that counts a frame differently is not a failure here
		let rows: &[(&str, &str, &str, &str)] = &[
			("video.ts",      "mpegts",                  "mpeg2video", "1.0"),
			("video.mpg",     "mpeg",                    "mpeg1video", "0.82"),
			("dvd-16x9.vob",  "mpeg",                    "mpeg2video", "0.934267"),
			("video.avi",     "avi",                     "mpeg4",      "1.0"),
			("video.flv",     "flv",                     "flv1",       "1.0"),
			("video.wmv",     "asf",                     "wmv2",       "1.0"),
			("video.rm",      "rm",                      "rv20",       "1.0"),
			("video.mov",     "mov,mp4,m4a,3gp,3g2,mj2", "mpeg4",      "1.0"),
			("moov-last.mp4", "mov,mp4,m4a,3gp,3g2,mj2", "mpeg4",      "1.0"),
			("dvd-16x9.mov",  "mov,mp4,m4a,3gp,3g2,mj2", "h264",       "1.0"),
			("video.mkv",     "matroska,webm",           "mpeg4",      "1.0"),
			("video.webm",    "matroska,webm",           "vp8",        "1.0"),
			("dvd-16x9.webm", "matroska,webm",           "vp9",        "1.0"),
			("animated.gif",  "gif",                     "gif",        "0.2"),
			("audio.wma",     "asf",                     "wmav2",      "1.021"),
			("audio.aac",     "aac",                     "aac",        "1.044917"),
			("audio.m4a",     "mov,mp4,m4a,3gp,3g2,mj2", "aac",        "1.0"),
			("audio.aiff",    "aiff",                    "pcm_s16be",  "1.0"),
			("audio.au",      "au",                      "pcm_s16be",  "1.0"),
			("audio.wav",     "wav",                     "pcm_s16le",  "1.0"),
			("plain.mp3",     "mp3",                     "mp3",        "1.044875"),
			("audio.flac",    "flac",                    "flac",       "1.0"),
			("audio.ogg",     "ogg",                     "vorbis",     "1.001361"),
			("audio.opus",    "ogg",                     "opus",       "1.0065"),
			("picture.png",   "png_pipe",                "png",        ""),
		];
		let mut wrong = vec![];
		for &(name, format, codec, seconds) in rows {
			let p = match probe(&sample(name), 10.0) { Ok(p) => p, Err(e) => { wrong.push(format!("{name}: {e}")); continue } };
			let close = match (p.duration.parse::<f64>(), seconds.parse::<f64>()) { (Ok(a), Ok(b)) => (a - b).abs() < 0.01, (Err(_), Err(_)) => true, _ => false };
			if p.format != format || !close || !p.tracks.contains(&format!("\"codec\":\"{codec}\"")) { wrong.push(format!("{name}: {} {} {} against {format} {seconds} {codec}", p.format, p.duration, p.tracks)) }
		}
		assert!(wrong.is_empty(), "{}", wrong.join("\n"));
	}
	#[test]
	fn the_probe_reads_a_sample() {//one question of one of steel's sample files, which proves this crate's test binary links the media library and finds it at run time through the rpath build.rs gives it
		let p = probe(concat!(env!("CARGO_MANIFEST_DIR"), "/../steel/samples/video.ts"), 10.0).unwrap();
		assert_eq!((p.format.as_str(), p.duration.parse::<f64>().unwrap()), ("mpegts", 1.0));
	}
}
