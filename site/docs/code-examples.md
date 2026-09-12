# Code Examples

Fuji's own source, in enough shapes to judge the site's typography by.

A code fence is the one thing on these pages that has to hold somebody else's formatting exactly as it was written, and it is the hardest thing to style from a sample. VitePress's starter page demonstrates syntax highlighting on six lines of `msg: 'Highlighted!'`, which looks correct at any setting and therefore says nothing about whether a setting is right. The excerpts below are the real thing instead: tab-indented, never wrapped to a column, commented about as heavily as the code itself, and long enough in places that no window will hold a line of them.

::: warning Scaffolding
This page is here to look at while we settle the site's styling, and it will probably go once that is done. Nothing here explains how Fuji works — [the thumbnail pipeline](/thumbnail-pipeline) is where that sort of thing lives.
:::

Two languages, because Fuji is two halves: JavaScript for the page the user sees, Rust for the program that owns the window, the files, and anything the operating system will do on Fuji's behalf. Every excerpt comes out of the repository as it stands, with no reformatting, no trimming, and no tidying of the comments, and the one section that breaks that rule says so.

## One short function

The whole of `AlphabetSort.js`, the smallest file in the application, and the first of the orders Fuji will offer a folder in.

```js
//the first sort, and the plainest fuji will have: javascript's own sort(), capitals before lowercase and page10 before page9, with no locale and no opinion. sort.md carries the seven planned beside it

export default function alphabetSort(files) {//these image files as an ordered array of paths; a sort returns the order rather than a comparator, so a shuffle can be one too
	return files
		.map(file => file.path)//paths, because a path is what a view asks the cache with; within one folder, ordering these is ordering names
		.sort()//no comparator on purpose: here the platform's own answer is the wanted one
}
```

Three lines of code carrying three of prose, which is roughly the ratio the rest of the codebase keeps.

## Lines nobody wrapped

Two rules meet here and make the hardest case on the page. The house style forbids wrapping a line to a column width, because an artificial break turns a one-word change into a multi-line diff; and it asks for a comment on the end of the line it belongs to, rather than above it. The result is lines that are longer than any window and that nothing in the file will ever fold.

So the fence scrolls, which is VitePress's own behaviour and what the site keeps. We tried soft wrapping the blocks to their own width in its place and took it back out.

This is the whole of `disk.js`, the page's side of five Rust commands. The braces are aligned on purpose — when adjacent lines share a structure, the eye finds the differences instantly, and padding them into columns is information deliberately written into the whitespace. That alignment is also the thing a soft wrap cannot keep, since a wrapped line's tail begins at the left edge wherever its own line began.

```js
export function diskRead(path)                { return invoke('disk_read',    {path})                }//the whole file at once, as an ArrayBuffer; wrap it in a Uint8Array to use it
export function diskWrite(path, data)         { return invoke('disk_write',   {path, data})          }//creates the file, or truncates an existing one to nothing first; data is an array of byte values
export function diskReadDir(path)             { return invoke('disk_readdir', {path})                }//one folder, not its subfolders: name, is_file, is_dir, is_symlink, size. Skips any entry it cannot stat rather than failing the whole listing
export function diskStat(path)                { return invoke('disk_stat',    {path})                }//one path's metadata: the same three kind flags, size, and atime, mtime, ctime as milliseconds since 1970, each 0 where the filesystem has no answer. Describes a symlink itself rather than following it
export function diskCopy(source, destination) { return invoke('disk_copy',    {source, destination}) }//files only, overwrites the destination without asking, and cannot report progress or be cancelled
```

The module beside it does the same for the operating system's thumbnailer, and shows what a long trailing comment looks like when the code in front of it is short.

```js
export function thumbnailProbe(paths)                          { return invoke('thumbnail_probe',  {paths})                        }//one call for a card's paths, answered with {format, width, height, problem} for each, from the first bytes and the header alone; problem says why a file will not be shown, and is blank otherwise. Rejects only when the bridge itself failed
export function thumbnailRender(path, format, maximum, gamut) { return invoke('thumbnail_render', {path, format, maximum, gamut}) }//one ArrayBuffer: twelve bytes of header, then rgba. format is what the probe said the bytes are, and the render refuses a file that disagrees; maximum is the longer side in backing pixels, never enlarged; gamut is 'display-p3' or 'srgb'. Rejects on linux, for a file the operating system will not decode, and for a header claiming more than half the machine's memory
```

## A function of ordinary size

Most of Fuji's functions are this size. `flipCacheWindow` is the diamond table's whole policy about which images to keep decoded: the user is at some position in a folder, so hold everything within reach of it and let go of everything outside. The two highlighted lines are the ones that do that, and the highlight is VitePress's, not something in the file.

```js{6,8}
export function flipCacheWindow(list, index) {//the user is here: hold everything within reach and let go of everything outside it
	let want = new Map()
	for (let i = index - settings.flip.back; i <= index + settings.flip.forward; i++) {//read every time, so a number changed in fuji.toml means what it says
		if (i < 0 || i >= list.length) continue//the ends of a folder are simply a shorter window
		let path = list[i]
		want.set(path, flipCacheHeld.get(path) || cacheNeed(path, flipCacheHolder))//already held, or asked for now; the loads race each other and the operating system sorts them out
	}
	for (let path of flipCacheHeld.keys()) if (!want.has(path)) cacheRelease(path, flipCacheHolder)//out of reach, so the store may let it go if nobody else wants it
	flipCacheHeld = want
}
```

## A long one, with line numbers

`cacheNeed` is the front door of the image store, and the object literal in the middle of it is the store's entire idea of what it knows about a file. At twenty-four lines with a comment on nearly every one, it is a fair test of how a tall fence reads — and of whether the line numbers help or merely crowd.

```js:line-numbers
export function cacheNeed(path, holder, steps = {}) {//take a reference and get the image; asking and holding are the same act, so nothing can be had without saying who wants it. steps lets a caller leave out work it has no use for: {decode: false} answers with the bytes and the url and never builds an element, for a view that hands the engine an img of its own
	let entry = cacheEntries.get(path)
	if (!entry) {
		entry = {
			path,
			blob: null,//the file's bytes, kept for a decode at another size or a hash later; nothing reads it yet, and it costs nothing, because the url below holds these bytes either way
			url: '',//one object url over that blob, kept alive so re-showing the element is a rebuild from source rather than a full decode
			img: null,//the decoded element, which is the thing a table puts on screen; null for as long as nobody has asked for one
			blobBytes: 0, pixelBytes: 0,//counted on the entry so dropping it can subtract exactly what it added
			references: new Map(),//holder name to how many times that holder has asked
			requested: performance.now(), loaded: 0, rendered: 0,//two durations: getting the bytes, then decoding them. performance.now everywhere, because these are intervals and it cannot jump the way a wall clock can
			error: null,
			reading: null, decoding: null,//the two steps as promises, each made once and shared by everyone who arrives while it runs, which is bookkeeping rather than judgement; decoding stays null until the first caller asks for an element
		}
		cacheEntries.set(path, entry)
		entry.reading = cacheRead(entry)
	}
	entry.references.set(holder, (entry.references.get(holder) || 0) + 1)

	let decode = steps.decode ?? true//the one step a caller can leave out
	if (!decode) return entry.reading//the same entry, resolved as soon as the bytes and the url are in hand
	if (!entry.decoding) entry.decoding = cacheDecode(entry)//the first caller to want an element starts the decode, on an entry that may have been read long ago for someone who wanted only the url
	return entry.decoding
}
```

## The boundary, in Rust

`lib.rs` names everything the page is allowed to ask Rust to do. Anything absent from the list is unreachable from JavaScript however the page is coaxed, which makes this file both the map of Fuji's Rust and the whole of its attack surface. It is also a builder chain and a macro invocation, so it exercises the parts of Rust highlighting that a plain function does not.

```rust
pub fn run() {
	tauri::Builder::default()//start building the Tauri application
		.plugin(tauri_plugin_opener::init())//reveal a file in finder or explorer; capabilities grant only reveal, not url opening
		.plugin(tauri_plugin_dialog::init())//the familiar os open and save dialog boxes; capabilities grant only those two, not message boxes
		.manage(desktop::ExitFiles::default())//shared state any command can reach: text handed down to be written on the way out
		.invoke_handler(//the complete list of what javascript may invoke; a name absent here cannot be called at all
			tauri::generate_handler![
				disk::disk_readdir, //functions we've written in disk.rs
				disk::disk_stat,
				disk::disk_read,
				disk::disk_write,
				disk::disk_copy,
				desktop::desktop_exit_hold,//and in desktop.rs
				log::log_start,//and in log.rs
				log::log_append,
				panel::panel_resolution,//and in panel.rs
				thumbnail::thumbnail_render,//and in thumbnail.rs
				thumbnail::thumbnail_probe,
			]
		)
		.build(tauri::generate_context!())//build rather than run, so the closure below gets the event loop
		.expect("error while building tauri application")//panic if startup fails (e.g. bad config)
		.run(|app, event| {//this closure sees every event the application loop produces, for the life of the process
			if let tauri::RunEvent::Exit = event { desktop::desktop_exit_write(app); log::log_write() }//the one event every quit path reaches; desktop.rs says why
		});
}
```

## Fields in columns

The two shapes the thumbnailer moves data in. Aligning the field names and their comments is the same instinct as the aligned braces above, and it is the sort of thing a change to the monospace font, its size, or its letter spacing will either preserve or quietly ruin.

```rust
pub struct Thumbnail {//what a platform hands back, before it is packed into the one buffer
	pub width:  u32,
	pub height: u32,
	pub wide:   bool,//true when the pixels are display p3, which only the mac produces
	pub pixels: Vec<u8>,//straight-alpha rgba, width times four bytes a row, top row first
}

#[derive(Serialize)]
pub struct Probe {//what the probe says about one path, in one shape whatever it found
	pub format:  String,//jpeg png gif bmp webp avif svg heic, from the first bytes; blank when they name nothing fuji knows
	pub width:   u32,//from the header alone, as the picture will show after its orientation; 0 when nothing short of decoding could say
	pub height:  u32,
	pub problem: String,//why this file will not be shown, or blank
}
```

A short Rust function for comparison — the one that writes the log on the way out, which is the last thing Fuji does before the process ends.

```rust
pub fn log_write() {
	let mut log = LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	if log.path.is_empty() || log.text.is_empty() { return }
	if let Some(folder) = std::path::Path::new(&log.path).parent() { let _ = std::fs::create_dir_all(folder); }//the folder under home may not exist yet; if this fails, the write below says so
	if let Err(e) = std::fs::write(&log.path, log.text.as_bytes()) {
		eprintln!("fuji could not write its log on the way out: {e}");//stderr, the only place left, and one nobody is likely watching
	}
	log.text.clear();//so nothing is written twice if this is somehow reached again
}
```

## One command, four bodies

`panel_resolution` answers how many pixels are really on the glass, which every operating system answers differently and none answers directly. So it is one command with four bodies, chosen by `#[cfg(target_os = "...")]` at compile time rather than by a branch at runtime. Three of them are below, in tabs, which is the other way a fence can arrive on a page — each one the whole module, so the attribute that selects it is the first line.

::: code-group

```rust [macOS]
#[cfg(target_os = "macos")]
mod platform {
	use core_graphics::display::{
		CGMainDisplayID,
		CGDisplayCopyAllDisplayModes,
		CGDisplayMode,
	};
	use foreign_types_shared::ForeignType;
	use super::Arrow;
	use std::ffi::c_void;//an address with no type attached, which is how C describes a pointer to anything
	use std::mem::ManuallyDrop;
	use std::panic;
	
	extern "C" {//declared by hand because the crate wrapping Quartz does not wrap CFArray; the linker finds the real ones
		fn CFArrayGetCount(array: *const c_void) -> isize;
		fn CFArrayGetValueAtIndex(array: *const c_void, index: isize) -> *const c_void;
		fn CFRelease(cf: *const c_void);
	}
	
	pub fn panel_resolution() -> Arrow {
		panic::catch_unwind(|| unsafe {
			let id = CGMainDisplayID();//the display with the menu bar on it, not necessarily the one fuji is showing on
			let modes = CGDisplayCopyAllDisplayModes(id, std::ptr::null());//Copy in the name, so this array is ours to release below. The null options are load-bearing: pass kCGDisplayShowDuplicateLowResolutionModes instead and the list gains the scaled modes' backing stores, so the tallest below becomes 3420 by 2224 on a machine whose glass is 2560 by 1664 — the backing store, which is the one answer this function exists not to give. fidelity.md has the measurement
			if modes.is_null() {
				return Arrow { x: 0, y: 0 };
			}
			
			let count = CFArrayGetCount(modes as *const c_void);//array length
			let mut winning_height = 0;//let is immutable by default, so mut is what makes these two assignable in the loop
			let mut winner = Arrow { x: 0, y: 0 };
			
			for i in 0..count {//an exclusive range, so 0 up to but not including count
				let mode_ref = CFArrayGetValueAtIndex(modes as *const c_void, i);//Get in the name, so this one is only lent to us
				if mode_ref.is_null() {
					continue;
				}
				let mode = ManuallyDrop::new(CGDisplayMode::from_ptr(mode_ref as *mut _));//from_ptr would release this mode; ManuallyDrop cancels that
				let height = mode.pixel_height() as u32;//the framebuffer, not the point size: 1920 x 1080 points is backed by 3840 x 2160 pixels
				let width = mode.pixel_width() as u32;
				if height > winning_height {
					winning_height = height;
					winner = Arrow { x: width, y: height };
				}
			}
			
			CFRelease(modes as *const c_void);//necessary to release the array
			winner
		})
		.unwrap_or(Arrow { x: 0, y: 0 })
	}
}
```

```rust [Windows]
#[cfg(target_os = "windows")]//conditional compilation: the bodies that do not match are absent from the binary, not skipped at runtime
mod platform {
	use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};//user32's stable way to ask the desktop's shape
	use super::Arrow;//super is the module above, so this is the Arrow declared at the top of this file
	use std::panic;

	pub fn panel_resolution() -> Arrow {
		panic::catch_unwind(|| unsafe {//unsafe promises the compiler we checked what it cannot; catch_unwind makes a panic a value rather than a dead app
			let width = GetSystemMetrics(SM_CXSCREEN);//the primary display's size, and only the panel's own when the process is dpi aware
			let height = GetSystemMetrics(SM_CYSCREEN);
			if width <= 0 || height <= 0 {//a signed int, so compare before casting: a negative cast to u32 becomes an enormous number
				return Arrow { x: 0, y: 0 };
			}
			Arrow { x: width as u32, y: height as u32 }//every "as" is a conversion someone chose, and a block's last expression is its value
		})
		.unwrap_or(Arrow { x: 0, y: 0 })//and the fallback if that closure panicked
	}
}
```

```rust [Linux]
#[cfg(target_os = "linux")]
mod platform {
	use std::process::Command;//shell out to xrandr rather than bind the RandR C api, which every build machine would then need
	use super::Arrow;
	
	pub fn panel_resolution() -> Arrow {
		//reads output meant for a person, and speaks only to X11: a wayland session has no xrandr to answer
		let output = match Command::new("xrandr").arg("--query").output() {//Err(_) says we want to know there was an error, not which
			Ok(o) => o,
			Err(_) => return Arrow { x: 0, y: 0 },
		};
		let stdout = String::from_utf8_lossy(&output.stdout);//lossy: replace anything that is not valid utf-8 rather than failing over a stray byte
		for line in stdout.lines() {
			//each "if let Some" is one step that found its piece or gives up, so a surprise falls out the bottom as no answer rather than a wrong one
			if line.contains(" connected primary ") {
				if let Some(res_part) = line.split_whitespace().find(|w| w.contains('+')) {//the geometry field, written like "2560x1440+0+0"
					if let Some((resolution, _)) = res_part.split_once('+') {
						if let Some((width_str, height_str)) = resolution.split_once('x') {
							if let (Ok(width), Ok(height)) = (width_str.parse::<u32>(), height_str.parse::<u32>()) {
								return Arrow { x: width, y: height };
							}
						}
					}
				}
			}
		}
		Arrow { x: 0, y: 0 }
	}
}
```

:::

The macOS body is the long one for a reason worth one sentence: Quartz has no call that says "this is the native resolution", so the code asks for every mode the display can be set to and keeps the tallest, on the grounds that a display cannot offer more pixels than it has.

## Reading the first bytes of a file

Hex literals, byte strings, a `matches!` macro and a closure over a slice — a denser line-for-line sample than anything above. This is how Fuji decides what a file is, which is a question it will not take the extension's word for.

```rust
fn sniff(head: &[u8]) -> &'static str {//the format the first bytes announce, or blank; the same signatures chromium chooses its decoder by
	if head.starts_with(&[0xFF, 0xD8, 0xFF]) { return "jpeg" }
	if head.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) { return "png" }
	if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") { return "gif" }
	if head.starts_with(b"BM") { return "bmp" }
	if head.len() >= 12 && head.starts_with(b"RIFF") && &head[8..12] == b"WEBP" { return "webp" }
	if head.len() >= 12 && &head[4..8] == b"ftyp" {//an iso media box, whose brand says which picture format is inside
		let brand = &head[8..12];
		if brand == b"avif" || brand == b"avis" { return "avif" }
		if matches!(brand, b"heic" | b"heix" | b"hevc" | b"hevx" | b"mif1" | b"msf1") { return "heic" }
	}
	let text = head.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(head);//an svg is text, perhaps behind a byte order mark and whitespace, and all that can be said of it here is that it opens a tag; the img it goes into is its sandbox
	if text.iter().find(|b| !b.is_ascii_whitespace()) == Some(&b'<') { return "svg" }
	""
}
```

## Not the two main languages

Four more that turn up on these pages. First the settings file, which is the one sample here that is not copied from a file in the repository, because it is not in the repository: Fuji writes `fuji.toml` into the user's home folder, reads it on the next launch, and regenerates the whole text on the way out, so code produces its comments and its column padding rather than a person typing them. These are two of its sections as that code renders them.

```toml
# fuji.toml — fuji reads this file when it starts and writes it when it closes; edit the values freely, but the comments and the layout are regenerated every time, so notes of your own here will not survive

[view]
# which kind of view fuji was showing when it last closed, so it opens there again: Sheet for the contact sheet, Table for whichever table
showing = "Table"   # factory "Table"
# which table was showing: Diamond sizes an image into an invisible diamond on an infinite plane, Comic runs it full width down a scroll; the tables fuji has are known to the shell rather than here, so a name it does not recognize is reported there and Diamond shown instead
table   = "Diamond" # factory "Diamond"

[flip]
# how many images before the one on screen a table keeps decoded, so flipping back to them is instant instead of a fresh read and decode; one is the smallest that works, because a table always holds the image on either side of the one it is showing, and one here with one forward is the behaviour fuji had before it kept a window
back    = 5 # factory 5
# and how many after it; flipping forward is the common direction, so this is the one to raise first if a folder of large images still makes the user wait
forward = 5 # factory 5
```

Then the sidecar written beside each installer, which is the only thing on the live site that this build does not contain — the home page fetches all three at runtime, which is why publishing an installer changes what the page shows without the site being rebuilt:

```json
{
	"file": "fuji.exe",
	"version": "0.1.0",
	"arch": "x64",
	"bytes": 2042921,
	"sha256": "ac881a265b97be37da8eed2e8e8e42a40cf170de1c6f41ca638f723c5ff0efaf",
	"date": "2026-09-11"
}
```

The commands that build the application, which is the one other place a monospace face has to hold an aligned column:

```bash
pnpm install      # at the repository root; installs every workspace
cd desktop        # every command below runs from the workspace, not the root
pnpm local        # run tauri in dev mode with hot reload
pnpm build-binary # quickest proof the release build compiles and links; no bundles
pnpm release      # stage the installer under its publishing name and hash it
```

And a Vue template, from this site's own home page — the logotype and the disc, which is the whole of Fuji's identity. It sits three wrappers deep in the file and we quote it at that depth, which is the clearest look on this page at what the tab width is doing:

```vue
			<!-- the logotype and the disc, the whole identity -->
			<div class="mark">
				<h1>Fuji</h1>
				<svg viewBox="0 0 360 360" xmlns="http://www.w3.org/2000/svg">
					<circle cx="180" cy="180" r="180" fill="white" />
				</svg>
			</div>
```

## What a fence can do besides hold code

Three blocks carrying four notations of VitePress's own. **We added those markers and they are not in Fuji's source** — the samples are shortened, and this is the one section on the page that is not quoting anything exactly.

Focus dims every line but the ones it names, which suits a file where the surrounding lines are there for context rather than for reading:

```js
export function thumbnailUnpack(buffer) {
	let header = new DataView(buffer, 0, 12) // [!code focus]
	let width = header.getUint32(0, true), height = header.getUint32(4, true)
	let gamut = header.getUint32(8, true) ? 'display-p3' : 'srgb'
	return {width, height, gamut, pixels: new Uint8ClampedArray(buffer, 12, width * height * 4)}
}
```

A diff, which is the real change that turned this site's clean URLs off so that every link would point at a file that exists:

```js
process.env.VITE_EXTRA_EXTENSIONS = 'dmg,exe,msi,deb,rpm,appimage,sig,zip' // [!code ++]

export default defineConfig({
	title: 'Fuji',
	description: 'A multimedia file manager designed with privacy and precision in mind',
	cleanUrls: true, // [!code --]
})
```

And a line marked as an error beside one marked as a warning, which are the two colors a fence can paint a single line:

```js
let count = entry.references.get(holder)
if (!count) throw new Error(`released a path this holder never needed`) // [!code error]
if (count > 1) entry.references.set(holder, count - 1)
else entry.references.delete(holder) // [!code warning]
```

## The one thing this page changed

One declaration, in `.vitepress/theme/style.css`, and the only thing on the site that is not stock VitePress.

**A tab is two columns.** Fuji indents with tabs, and `.editorconfig` at the repository root says `indent_size = 2`, so every editor on every machine here shows a tab two columns wide. VitePress sets `tab-size: 4`, which rendered every excerpt on this site at exactly twice the depth its file reads at — worth most of a screenful of left margin on the macOS body above, which is three levels in before it says anything.

It is not a change to a file, or to what a reader copies out of a block. It is the editor's own setting, said again in CSS for somebody who is reading in a browser.

## Prose with code in it

The last thing a styling change touches is inline code, which has to sit inside a sentence without breaking its line. A sentence with several: `cacheNeed(path, holder)` takes a reference and returns the image, `cacheRelease(path, holder)` gives it back, and when the last reference goes the store frees the blob, revokes the object url, and empties the element. Some of them are not identifiers at all — a path like `desktop/src-tauri/src/thumbnail.rs`, an attribute like `#[cfg(target_os = "windows")]`, a setting written `flip.forward`, or the eight bytes `0x89 0x50 0x4E 0x47 0x0D 0x0A 0x1A 0x0A` that begin every PNG.

And the containers, which share the code block's palette and so move with it:

::: info
The store holds what the views tell it to hold and lets go when they say to let go. It does not decide, schedule, prioritise, or expire.
:::

::: tip
A flip shows first and asks the store for anything new last, because a read or a decode started before the paint blocks the frame it was meant to help.
:::

::: danger
`disk_write` truncates an existing file to nothing before writing, exactly as POSIX does. Code that calls it has to be careful, as native application code has to be.
:::
