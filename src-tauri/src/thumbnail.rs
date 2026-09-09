/*
The operating system's thumbnailer, called from Rust: ImageIO on the Mac and the Windows Imaging Component on Windows, the libraries Finder and Explorer use. Given a path and a longest side, each decodes the file scaled where the codec allows, a JPEG at an eighth of its size, resamples the rest of the way with a filter that reads every source pixel, applies the file's orientation, converts its colors into the space asked for, and hands back the small pixels. The full-size raster never exists, nothing here runs on the thread that runs the window, and a thumbnail crosses to the page rather than a file. thumbnail-plan.md says which files come here and which the page makes for itself; canvas.md has the measurements that decided it.

Both libraries take a path or bytes, and the path is right: the library reads the file itself and the page never holds it. That is the same trust disk_read extends, under the contract disk.rs states.

Two commands. thumbnail_probe takes a card's paths and says, for each, what its first bytes are and what its header claims its size is, without decoding, so the page can route every file and lay out every box before any thumbnail is made. thumbnail_render makes one thumbnail. Both hold the two walls from security.md: bytes that are not a format fuji handles are refused whatever the extension, and a header claiming a raster that would not fit in half this machine's memory is refused before any decoder allocates. That second is the decompression bomb, which needs no bug in anything, and it is a share of the machine's memory rather than a number, so it never limits capable hardware.

The render returns one buffer, so the bytes cross as an ArrayBuffer rather than a json array of numbers, which performance.md has the cost of. Twelve bytes of header — width, height, and whether the pixels are Display P3 — as little-endian unsigned 32-bit integers, then straight-alpha RGBA, top row first; thumbnail.js unpacks it for ImageData. The longer side is the maximum asked for, or the picture's own when it was smaller, because neither library enlarges.

The Mac body is short because ImageIO does the whole job in one call given three options, and drawing the result into a bitmap context of the wanted color space is where CoreGraphics does the color management. The Windows body is long because WIC is a pipeline of separate objects, each initialised over the last, and because WIC leaves EXIF orientation to the caller. The page owns two things: the color space, since only it knows the screen's gamut, and Windows answers sRGB whatever is asked, which the header says; and the fit, since it turns the returned size back into a css size by one rule, longer side to the box, never enlarged.
*/

use serde::Serialize;
use std::io::Read;
use std::sync::OnceLock;
use tauri::ipc::Response;

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

//a card's worth of paths at once, so the page asks once per card rather than once per file; on tauri's thread pool, like the render
#[tauri::command(async)]
pub fn thumbnail_probe(paths: Vec<String>) -> Vec<Probe> {
	paths.iter().map(|path| probe(path)).collect()
}

//the one thumbnail; the async in the attribute has tauri run this sync body on its thread pool, which is what keeps a decode off the thread that runs the window
#[tauri::command(async)]
pub fn thumbnail_render(path: String, format: String, maximum: u32, gamut: String) -> Result<Response, String> {
	if path.trim().is_empty() { return Err("thumbnail_render: expected a path".into()) }//the mistakes a caller could make, returned as errors the page sees rather than a panic on a pool thread
	if maximum == 0 { return Err("thumbnail_render: expected a longest side of at least 1".into()) }
	let head = head(&path)?;
	let found = sniff(&head);
	if found != format { return Err(format!("thumbnail: the bytes say {} and the caller expected {format}: {path}", if found.is_empty() { "nothing fuji knows" } else { found })) }//the first wall, held here as well as in the probe, so a caller that skipped the probe cannot hand this a mystery
	let wide = gamut == "display-p3";//anything else is srgb, which is what a canvas is unless asked
	let t = platform::render(&path, maximum, wide)?;//which holds the second wall, the ceiling, because it has the header in hand before it decodes

	let mut bytes = Vec::with_capacity(12 + t.pixels.len());//the header, then the pixels, in one buffer so it crosses raw
	bytes.extend_from_slice(&t.width.to_le_bytes());
	bytes.extend_from_slice(&t.height.to_le_bytes());
	bytes.extend_from_slice(&(t.wide as u32).to_le_bytes());
	bytes.extend_from_slice(&t.pixels);
	Ok(Response::new(bytes))
}

fn probe(path: &str) -> Probe {//one file: its format from its bytes, its size from its header, and whether either is a reason to refuse it
	let mut p = Probe { format: String::new(), width: 0, height: 0, problem: String::new() };
	let head = match head(path) { Ok(head) => head, Err(problem) => { p.problem = problem; return p } };
	p.format = sniff(&head).to_string();
	if p.format.is_empty() { p.problem = "the first bytes are not an image fuji knows".into(); return p }

	let (mut width, mut height) = head_size(&p.format, &head);//png, gif and bmp say their size in the bytes already read
	if width == 0 && matches!(p.format.as_str(), "jpeg" | "webp" | "avif" | "heic") {//the rest need the library to read further into the header, which the mac and windows can do without decoding and linux cannot
		if let Ok(size) = platform::size(path) { width = size.0; height = size.1 }
	}
	p.width = width; p.height = height;
	if let Err(problem) = check_ceiling(width, height) { p.problem = problem }
	p
}

fn head(path: &str) -> Result<Vec<u8>, String> {//the first bytes of a file, up to 32, which is enough to name every format fuji handles and to size three of them
	let mut file = std::fs::File::open(path).map_err(|e| format!("thumbnail: {path}: {e}"))?;
	let mut buffer = vec![0u8; 32];
	let mut got = 0;
	while got < buffer.len() {
		let n = file.read(&mut buffer[got..]).map_err(|e| format!("thumbnail: {path}: {e}"))?;
		if n == 0 { break }
		got += n;
	}
	buffer.truncate(got);
	Ok(buffer)
}

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

fn head_size(format: &str, head: &[u8]) -> (u32, u32) {//the size written in the first bytes, for the formats that write it there; 0 for the others
	let le16 = |i: usize| u16::from_le_bytes([head[i], head[i + 1]]) as u32;
	let be32 = |i: usize| u32::from_be_bytes([head[i], head[i + 1], head[i + 2], head[i + 3]]);
	let le32 = |i: usize| i32::from_le_bytes([head[i], head[i + 1], head[i + 2], head[i + 3]]).unsigned_abs();//a bmp stores a negative height to mean top-down rows
	match format {
		"png" if head.len() >= 24 => (be32(16), be32(20)),
		"gif" if head.len() >= 10 => (le16(6), le16(8)),
		"bmp" if head.len() >= 26 && le32(14) == 12 => (le16(18), le16(20)),//the 1990 os/2 header, twelve bytes long with 16-bit sides
		"bmp" if head.len() >= 26 => (le32(18), le32(22)),
		_ => (0, 0),
	}
}

fn check_ceiling(width: u32, height: u32) -> Result<(), String> {//the second wall: a header may claim any size at all, and a decoder believes it before it finds out otherwise
	if width == 0 || height == 0 { return Ok(()) }//unknown, so nothing to check against
	let raster = width as u64 * height as u64 * 4;
	let ceiling = memory() / 2;
	if raster > ceiling { return Err(format!("thumbnail: {width} by {height} pixels would need {} MB to decode, more than half of this machine's {} MB", raster >> 20, memory() >> 20)) }
	Ok(())
}

fn memory() -> u64 {//the machine's physical memory in bytes, asked once; 8 GB when the platform will not say, which is a floor rather than a guess about any real machine
	static MEMORY: OnceLock<u64> = OnceLock::new();
	*MEMORY.get_or_init(|| { let m = platform::memory(); if m > 0 { m } else { 8 << 30 } })
}

#[cfg(target_os = "macos")]
mod platform {
	use core_foundation::base::{CFType, TCFType};
	use core_foundation::boolean::CFBoolean;
	use core_foundation::dictionary::{CFDictionary, CFDictionaryGetValue, CFDictionaryRef};
	use core_foundation::number::{CFNumber, CFNumberRef};
	use core_foundation::string::{CFString, CFStringRef};
	use core_foundation::url::{CFURL, CFURLRef};
	use core_graphics::base::{kCGBitmapByteOrder32Big, kCGImageAlphaPremultipliedLast};
	use core_graphics::color_space::{kCGColorSpaceDisplayP3, kCGColorSpaceSRGB, CGColorSpace};
	use core_graphics::context::CGContext;
	use core_graphics::geometry::{CGPoint, CGRect, CGSize};
	use core_graphics::image::CGImage;
	use core_graphics::sys::CGImageRef;
	use foreign_types_shared::ForeignType;
	use std::ffi::{c_char, c_void};
	use std::panic;
	use super::Thumbnail;

	type CGImageSourceRef = *const c_void;//ImageIO's handle to a file it has opened, opaque to us

	#[link(name = "ImageIO", kind = "framework")]
	extern "C" {//declared by hand because no crate fuji has wraps ImageIO; the linker finds the real ones in the framework
		fn CGImageSourceCreateWithURL(url: CFURLRef, options: CFDictionaryRef) -> CGImageSourceRef;
		fn CGImageSourceCopyPropertiesAtIndex(source: CGImageSourceRef, index: usize, options: CFDictionaryRef) -> CFDictionaryRef;
		fn CGImageSourceCreateThumbnailAtIndex(source: CGImageSourceRef, index: usize, options: CFDictionaryRef) -> CGImageRef;
		static kCGImageSourceThumbnailMaxPixelSize: CFStringRef;
		static kCGImageSourceCreateThumbnailFromImageAlways: CFStringRef;
		static kCGImageSourceCreateThumbnailWithTransform: CFStringRef;
		static kCGImagePropertyPixelWidth: CFStringRef;
		static kCGImagePropertyPixelHeight: CFStringRef;
		static kCGImagePropertyOrientation: CFStringRef;
	}
	extern "C" {
		fn CFRelease(cf: *const c_void);
		fn sysctlbyname(name: *const c_char, old: *mut c_void, old_length: *mut usize, new: *mut c_void, new_length: usize) -> i32;//how the mac says how much memory it has
	}

	pub fn render(path: &str, maximum: u32, wide: bool) -> Result<Thumbnail, String> {
		panic::catch_unwind(|| render_or_panic(path, maximum, wide))//the crate wrapping CoreGraphics asserts rather than returns on a context it could not make, and a panic on a pool thread has to come back as an error the page can see
			.unwrap_or_else(|_| Err(format!("thumbnail: CoreGraphics could not draw {path}")))
	}

	fn render_or_panic(path: &str, maximum: u32, wide: bool) -> Result<Thumbnail, String> {
		let source = open(path)?;
		let (width, height) = properties_size(source);//the header, read before anything decodes
		if let Err(problem) = super::check_ceiling(width, height) { unsafe { CFRelease(source) }; return Err(problem) }

		let options: CFDictionary<CFString, CFType> = CFDictionary::from_CFType_pairs(&[
			(key(unsafe { kCGImageSourceThumbnailMaxPixelSize }),          CFNumber::from(maximum as i64).as_CFType()),//the longer side, never enlarging
			(key(unsafe { kCGImageSourceCreateThumbnailFromImageAlways }), CFBoolean::true_value().as_CFType()),//render one from the picture; without this a camera's embedded 160 by 120 preview comes back instead
			(key(unsafe { kCGImageSourceCreateThumbnailWithTransform }),   CFBoolean::true_value().as_CFType()),//apply the exif orientation, so a portrait from a phone is upright
		]);
		let image = unsafe { CGImageSourceCreateThumbnailAtIndex(source, 0, options.as_concrete_TypeRef()) };//scaled decode, resample, orientation: ImageIO does all three in here
		unsafe { CFRelease(source) };//the thumbnail stands on its own, so the source goes now, on every path below
		if image.is_null() { return Err(format!("thumbnail: ImageIO could not decode {path}")) }
		let image = unsafe { CGImage::from_ptr(image as *mut _) };//Create rule again: the wrapper releases it once, when it drops

		let (width, height) = (image.width(), image.height());
		let name = unsafe { if wide { kCGColorSpaceDisplayP3 } else { kCGColorSpaceSRGB } };
		let space = CGColorSpace::create_with_name(name).ok_or("thumbnail: no such color space")?;
		let mut context = CGContext::create_bitmap_context(None, width, height, 8, width * 4, &space, kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);//rgba bytes in that order; premultiplied because that is the only alpha a drawing context accepts, undone below
		context.draw_image(CGRect::new(&CGPoint::new(0.0, 0.0), &CGSize::new(width as f64, height as f64)), &image);//one to one, so this is a color conversion and not a resample
		let mut pixels = context.data().to_vec();
		unpremultiply(&mut pixels);
		Ok(Thumbnail { width: width as u32, height: height as u32, wide, pixels })
	}

	pub fn size(path: &str) -> Result<(u32, u32), String> {//the size the picture will show at, from the header alone
		let source = open(path)?;
		let size = properties_size(source);
		unsafe { CFRelease(source) };
		Ok(size)
	}

	fn open(path: &str) -> Result<CGImageSourceRef, String> {//an image source over the file, which reads nothing until asked; Create in the name, so the caller releases it
		let url = CFURL::from_path(path, false).ok_or_else(|| format!("thumbnail: not a path: {path}"))?;
		let source = unsafe { CGImageSourceCreateWithURL(url.as_concrete_TypeRef(), std::ptr::null()) };
		if source.is_null() { return Err(format!("thumbnail: ImageIO could not open {path}")) }
		Ok(source)
	}

	fn properties_size(source: CGImageSourceRef) -> (u32, u32) {//pixel width and height from the header, swapped when the exif orientation turns the picture on its side; 0 when the header does not say
		let properties = unsafe { CGImageSourceCopyPropertiesAtIndex(source, 0, std::ptr::null()) };
		if properties.is_null() { return (0, 0) }
		let properties: CFDictionary<CFString, CFType> = unsafe { CFDictionary::wrap_under_create_rule(properties) };//Copy in the name, so the wrapper's drop releases it
		let number = |name: CFStringRef| -> i64 {
			let value = unsafe { CFDictionaryGetValue(properties.as_concrete_TypeRef(), name as *const c_void) };//Get in the name: borrowed, so the wrapper below retains and releases, net nothing
			if value.is_null() { return 0 }
			unsafe { CFNumber::wrap_under_get_rule(value as CFNumberRef) }.to_i64().unwrap_or(0)
		};
		let width = number(unsafe { kCGImagePropertyPixelWidth }).max(0) as u32;
		let height = number(unsafe { kCGImagePropertyPixelHeight }).max(0) as u32;
		let sideways = number(unsafe { kCGImagePropertyOrientation }) >= 5;//5 through 8 are the four that turn width into height
		if sideways { (height, width) } else { (width, height) }
	}

	pub fn memory() -> u64 {
		let mut bytes: u64 = 0;
		let mut length = std::mem::size_of::<u64>();
		let status = unsafe { sysctlbyname(c"hw.memsize".as_ptr(), &mut bytes as *mut u64 as *mut c_void, &mut length, std::ptr::null_mut(), 0) };
		if status == 0 { bytes } else { 0 }
	}

	fn key(name: CFStringRef) -> CFString {//a constant the framework owns: Get rule, so the wrapper retains it on the way in and releases it on the way out, net nothing
		unsafe { CFString::wrap_under_get_rule(name) }
	}

	fn unpremultiply(pixels: &mut [u8]) {//straight alpha from premultiplied, which is the identity for every opaque pixel, so a jpeg costs one compare a pixel
		for p in pixels.chunks_exact_mut(4) {
			let a = p[3] as u32;
			if a == 0 || a == 255 { continue }
			for c in &mut p[..3] { *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8 }
		}
	}
}

#[cfg(target_os = "windows")]
mod platform {
	use windows::core::{w, Interface, HSTRING};
	use windows::Win32::Foundation::GENERIC_READ;
	use windows::Win32::Graphics::Imaging::*;
	use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
	use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PROPVARIANT};
	use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
	use windows::Win32::System::Variant::VT_UI2;
	use super::Thumbnail;

	pub fn render(path: &str, maximum: u32, _wide: bool) -> Result<Thumbnail, String> {//wide is ignored: everything comes back srgb, because wic has no display p3 context without a profile file, and the header says so
		let (width, height) = size(path)?;//the header, read before anything decodes
		super::check_ceiling(width, height)?;
		unsafe { render_com(path, maximum).map_err(|e| format!("thumbnail: {path}: {e}")) }
	}

	pub fn size(path: &str) -> Result<(u32, u32), String> {//the size the picture will show at, from the header alone
		unsafe {
			let _ = CoInitializeEx(None, COINIT_MULTITHREADED);//once per thread and harmless again; a pool thread keeps it for its life, and a thread already in the other mode says so and works anyway
			size_com(path).map_err(|e| format!("thumbnail: {path}: {e}"))
		}
	}

	unsafe fn size_com(path: &str) -> windows::core::Result<(u32, u32)> {
		let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
		let decoder = factory.CreateDecoderFromFilename(&HSTRING::from(path), None, GENERIC_READ, WICDecodeMetadataCacheOnDemand)?;//opens and reads the header; the pixels wait until something asks for them
		let frame = decoder.GetFrame(0)?;
		let (mut width, mut height) = (0u32, 0u32);
		frame.GetSize(&mut width, &mut height)?;
		if exif_orientation(&frame) >= 5 { Ok((height, width)) } else { Ok((width, height)) }
	}

	unsafe fn render_com(path: &str, maximum: u32) -> windows::core::Result<Thumbnail> {
		let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
		let decoder = factory.CreateDecoderFromFilename(&HSTRING::from(path), None, GENERIC_READ, WICDecodeMetadataCacheOnDemand)?;//wic opens and reads the file itself
		let frame = decoder.GetFrame(0)?;//the first frame, which is the whole picture for anything but an animation
		let (mut width, mut height) = (0u32, 0u32);
		frame.GetSize(&mut width, &mut height)?;

		let orientation = exif_orientation(&frame);//1 through 8, and 1 when the file says nothing
		let sideways = orientation >= 5;//the four that turn width into height
		let (shown_width, shown_height) = if sideways { (height, width) } else { (width, height) };
		let (target_width, target_height) = fit(shown_width, shown_height, maximum);
		let (want_width, want_height) = if sideways { (target_height, target_width) } else { (target_width, target_height) };//the size to decode at, in the file's own orientation, before the rotation below

		//a scaled decode, where the codec can do one: jpeg decodes at a half, a quarter or an eighth by skipping most of the inverse transform
		let mut source: IWICBitmapSource = frame.cast()?;
		if let Ok(transform) = frame.cast::<IWICBitmapSourceTransform>() {
			let (mut closest_width, mut closest_height) = (want_width, want_height);
			transform.GetClosestSize(&mut closest_width, &mut closest_height)?;//the nearest size the codec can produce natively, at or above the one asked for
			if closest_width < width || closest_height < height {
				let mut format = GUID_WICPixelFormat32bppBGRA;
				transform.GetClosestPixelFormat(&mut format)?;//and the nearest format it can produce, which may not be the one asked for
				let bits = factory.CreateComponentInfo(&format)?.cast::<IWICPixelFormatInfo>()?.GetBitsPerPixel()?;
				let stride = (closest_width * bits + 7) / 8;
				let mut buffer = vec![0u8; (stride * closest_height) as usize];
				transform.CopyPixels(std::ptr::null(), closest_width, closest_height, &format, WICBitmapTransformRotate0, stride, &mut buffer)?;
				source = factory.CreateBitmapFromMemory(closest_width, closest_height, &format, stride, &buffer)?.cast()?;//a bitmap over that buffer, so the rest of the pipeline can read it
				width = closest_width; height = closest_height;
			}
		}

		//the rest of the way with the fant scaler, which reads every source pixel
		if width != want_width || height != want_height {
			let scaler = factory.CreateBitmapScaler()?;
			scaler.Initialize(&source, want_width, want_height, WICBitmapInterpolationModeFant)?;
			source = scaler.cast()?;
		}

		//the orientation, which wic leaves to the caller
		if orientation != 1 {
			let rotator = factory.CreateBitmapFlipRotator()?;
			rotator.Initialize(&source, transform_for(orientation))?;
			source = rotator.cast()?;
		}

		//the file's color profile to srgb, when it carries one; a file without one is taken as srgb, which is what the engine assumes too
		if let Some(profile) = first_color_context(&factory, &frame) {
			let srgb = factory.CreateColorContext()?;
			srgb.InitializeFromExifColorSpace(1)?;//1 is srgb in exif's numbering
			let color = factory.CreateColorTransformer()?;
			if color.Initialize(&source, &profile, &srgb, &GUID_WICPixelFormat32bppBGRA).is_ok() { source = color.cast()? }//a profile wic cannot use leaves the colors as they are rather than failing the thumbnail
		}

		//straight-alpha rgba, top row first, which is what ImageData wants
		let converter = factory.CreateFormatConverter()?;
		converter.Initialize(&source, &GUID_WICPixelFormat32bppRGBA, WICBitmapDitherTypeNone, None, 0.0, WICBitmapPaletteTypeCustom)?;
		let stride = target_width * 4;
		let mut pixels = vec![0u8; (stride * target_height) as usize];
		converter.CopyPixels(std::ptr::null(), stride, &mut pixels)?;//the call that runs the whole pipeline above
		Ok(Thumbnail { width: target_width, height: target_height, wide: false, pixels })
	}

	pub fn memory() -> u64 {
		let mut status = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };//the length field is how windows knows which version of the struct it was handed
		unsafe { if GlobalMemoryStatusEx(&mut status).is_ok() { status.ullTotalPhys } else { 0 } }
	}

	fn fit(width: u32, height: u32, maximum: u32) -> (u32, u32) {//the longer side to maximum, never enlarging, which is what ImageIO does for itself on the mac
		let longer = width.max(height);
		if longer <= maximum { return (width, height) }
		let scale = maximum as f64 / longer as f64;
		(((width as f64 * scale).round() as u32).max(1), ((height as f64 * scale).round() as u32).max(1))
	}

	unsafe fn exif_orientation(frame: &IWICBitmapFrameDecode) -> u16 {
		let Ok(reader) = frame.GetMetadataQueryReader() else { return 1 };//png and bmp have no reader at all
		let mut value = PROPVARIANT::default();
		if reader.GetMetadataByName(w!("/app1/ifd/{ushort=274}"), &mut value).is_err() { return 1 }//exif's orientation tag, by its number, under the jpeg app1 segment
		let mut orientation = 1;
		if value.Anonymous.Anonymous.vt == VT_UI2 { orientation = value.Anonymous.Anonymous.Anonymous.uiVal }
		let _ = PropVariantClear(&mut value);
		if (1..=8).contains(&orientation) { orientation } else { 1 }
	}

	fn transform_for(orientation: u16) -> WICBitmapTransformOptions {//exif's eight cases as wic's flags, which wic applies as the rotation and then the flip; 5 and 7 are the mirrored diagonals no camera writes, expressed on that order and untested
		let (rotate, flip) = match orientation {
			2 => (WICBitmapTransformRotate0,   WICBitmapTransformFlipHorizontal),
			3 => (WICBitmapTransformRotate180, WICBitmapTransformRotate0),
			4 => (WICBitmapTransformRotate0,   WICBitmapTransformFlipVertical),
			5 => (WICBitmapTransformRotate90,  WICBitmapTransformFlipHorizontal),
			6 => (WICBitmapTransformRotate90,  WICBitmapTransformRotate0),
			7 => (WICBitmapTransformRotate90,  WICBitmapTransformFlipVertical),
			8 => (WICBitmapTransformRotate270, WICBitmapTransformRotate0),
			_ => (WICBitmapTransformRotate0,   WICBitmapTransformRotate0),
		};
		WICBitmapTransformOptions(rotate.0 | flip.0)
	}

	unsafe fn first_color_context(factory: &IWICImagingFactory, frame: &IWICBitmapFrameDecode) -> Option<IWICColorContext> {//the profile a file carries, or none; wic fills contexts the caller made rather than making them, hence the two calls
		let mut count = 0u32;
		let mut none: [Option<IWICColorContext>; 0] = [];
		frame.GetColorContexts(&mut none, &mut count).ok()?;//how many the file carries
		if count == 0 { return None }
		let context = factory.CreateColorContext().ok()?;
		let mut one = [Some(context.clone())];
		frame.GetColorContexts(&mut one, &mut count).ok()?;//fill the first; a second profile in one file is not a case worth carrying
		Some(context)
	}
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
	use super::Thumbnail;

	pub fn render(_path: &str, _maximum: u32, _wide: bool) -> Result<Thumbnail, String> {
		Err("thumbnail: the operating system's thumbnailer is not used on this platform".into())//linux stays in the web renderer, which thumbnail-plan.md decides and the page carries out
	}

	pub fn size(_path: &str) -> Result<(u32, u32), String> {
		Err("thumbnail: no library to read a header with on this platform".into())//so the probe answers 0, and the page lays the box out when the picture arrives
	}

	pub fn memory() -> u64 {//MemTotal from the kernel's own listing, in kilobytes there
		let text = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
		let line = text.lines().find(|line| line.starts_with("MemTotal:")).unwrap_or("");
		line.split_whitespace().nth(1).and_then(|kb| kb.parse::<u64>().ok()).map(|kb| kb * 1024).unwrap_or(0)
	}
}
