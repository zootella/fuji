/*
The operating system's thumbnailer, called from Rust: ImageIO on the Mac and the Windows Imaging Component on Windows, the libraries Finder and Explorer use. Given a path and the size a fit chose, each decodes the file scaled where the codec allows, a JPEG at an eighth of its size, resamples the rest of the way with a filter that reads every source pixel, applies the file's orientation, converts its colors into the space asked for, and hands back the small pixels. The full-size raster never exists, nothing here runs on the thread that runs the window, and a thumbnail crosses to the page rather than a file. The thumbnail pipeline document on fuji's site says which files come here and which the page makes for itself, and carries the measurements that decided it.

Both libraries take a path or bytes, and the path is right: the library opens the file and reads it itself, a buffer at a time as its decoder needs it, so a thumbnail is one open of its file, and the page never holds it. That is the same trust disk_read extends, under the contract disk.rs states.

There is one command, thumbnail_render, and it holds one wall before any decoder allocates: it refuses a header claiming a raster that would not fit in half this machine's memory. That is the decompression bomb, which needs no bug in anything, and the wall is a share of the machine's memory rather than a number, so it never limits capable hardware. It also refuses a picture whose size the library will not say, since no fit can be worked out without one; check_size says more. It stands here rather than in the page because this is where the decoder runs unsandboxed, in fuji's own process. Which files come here at all is the page's decision, made from the extension alone, and the library chooses its decoder from the file's own bytes.

The size comes from a fit. The page says which fit, the beam, the screen's size and how many backing pixels make a CSS pixel; Rust learns the picture's size from the library partway through the call, just before the thumbnail is made, and fit.rs turns that into the size to render at. The essay at the top of fit.js says why the fits are written in both languages and must agree to the integer, and fit.rs what is Rust's own. The render sends back the picture's own size beside the thumbnail, never the fit's answer, so the page works out the tile's size from that raw fact as it does for every other route.

The render returns one buffer, so the bytes cross as an ArrayBuffer rather than a json array of numbers — that serialization was measured at about 150 milliseconds a megabyte, flat and linear in file size, which no disk is. Twenty bytes of header — the thumbnail's width and height, whether its pixels are Display P3, and the picture's own width and height as it shows, after its orientation — as little-endian unsigned 32-bit integers, then straight-alpha RGBA, top row first; thumbnail.js unpacks it for ImageData. The thumbnail is the size the fit chose, or the picture's own when that is smaller, because neither library enlarges.

The Mac body is short because ImageIO does the whole job in one call given three options, and drawing the result into a bitmap context of the wanted color space is where CoreGraphics does the color management. The Windows body is long because WIC is a pipeline of separate objects, each initialised over the last, and because WIC leaves EXIF orientation to the caller. WIC is a COM library, which is how Windows hands an application an object out of a system DLL: a thread calls CoInitializeEx once before it asks for anything, and then every piece of the pipeline arrives through CoCreateInstance. There is no plain function to call instead, so the initialisation is the price of using the library at all, and it is not the heavier embedding layer of the same name that puts a spreadsheet inside a document. The page owns the color space, since only it knows the screen's gamut, and Windows answers sRGB whatever is asked, which the header says.
*/

use std::sync::OnceLock;
use tauri::ipc::Response;
use crate::run_blocking;

pub struct Thumbnail {//what a platform hands back, before it is packed into the one buffer
	pub width:  u32,
	pub height: u32,
	pub wide:   bool,//true when the pixels are display p3, which only the mac produces
	pub pixels: Vec<u8>,//straight-alpha rgba, width times four bytes a row, top row first
	pub natural_width:  u32,//the picture's own size as it shows, after its orientation, in image pixels, which the page sizes the tile from
	pub natural_height: u32,
}

pub struct Ask {//what the page asks for, which every platform turns into the size to render at through target below
	pub fit: String,//one of fitNames in fit.js
	pub beam: u32,//css pixels
	pub screen_width: u32,//css pixels, read only by ScaleFit and LogFit
	pub screen_height: u32,
	pub backing_per_css: f64,//devicePixelRatio, from the page
}

//the one thumbnail, its body on the blocking pool through run_blocking, which keeps a decode off the thread that runs the window and turns a panic anywhere below into an error
#[tauri::command]
pub async fn thumbnail_render(path: String, fit: String, beam: u32, screen_width: u32, screen_height: u32, backing_per_css: f64, gamut: String) -> Result<Response, String> {
	run_blocking(move || {
		if path.trim().is_empty() { return Err("thumbnail_render: expected a path".into()) }//the mistakes a caller could make, returned as errors the page can read; fit.rs checks the fit's own
		if !(backing_per_css.is_finite() && backing_per_css > 0.0) { return Err(format!("thumbnail_render: expected backing pixels per css pixel above zero: {backing_per_css}")) }
		let ask = Ask { fit, beam, screen_width, screen_height, backing_per_css };
		let wide = gamut == "display-p3";//anything else is srgb, which is what a canvas is unless asked
		let t = platform::render(&path, &ask, wide)?;//which holds the wall, because it has the picture's size in hand before it decodes

		let mut bytes = Vec::with_capacity(20 + t.pixels.len());//the header, then the pixels, in one buffer so it crosses raw
		bytes.extend_from_slice(&t.width.to_le_bytes());
		bytes.extend_from_slice(&t.height.to_le_bytes());
		bytes.extend_from_slice(&(t.wide as u32).to_le_bytes());
		bytes.extend_from_slice(&t.natural_width.to_le_bytes());
		bytes.extend_from_slice(&t.natural_height.to_le_bytes());
		bytes.extend_from_slice(&t.pixels);
		Ok(Response::new(bytes))
	}).await
}

/*
check_size: the picture's size as the library reported it, before anything decodes, refused if it would not fit in memory or if there is no size at all.

The ceiling is the wall against a decompression bomb: a header may claim any size at all, and a decoder believes it before it finds out otherwise. The size is the stored one, since turning a picture does not change its area.

No size is a different case. On the Mac, properties answers 0 when ImageIO's properties for a file lack a width or a height. Nobody has seen that happen: for every format fuji sends here, the size is a basic part of the file, and ImageIO reports it for any file it can decode. It is refused anyway, as the error it would be, because every fit is worked out from the picture's size and none can run without one. Windows cannot reach this case the same way, since GetSize either answers or fails, and a failure is already an error.
*/
fn check_size(width: u32, height: u32) -> Result<(), String> {
	if width == 0 || height == 0 { return Err(format!("thumbnail: the library gave no size for this picture, {width} by {height}")) }
	let raster = (width as u64).saturating_mul(height as u64).saturating_mul(4);//saturating, because two u32 sizes times four can pass u64, and a release build would wrap that to a small number the ceiling lets through: a header claiming 2^31 by 2^31 is the one file this wall is for, and it must not walk past it on arithmetic
	let ceiling = memory() / 2;
	if raster > ceiling { return Err(format!("thumbnail: {width} by {height} pixels would need {} MB to decode, more than half of this machine's {} MB", raster >> 20, memory() >> 20)) }
	Ok(())
}

//the size to render at, in backing pixels, for a picture that shows at width by height image pixels: the fit's css size times backing_per_css, rounded the way flowSnap in TestFlow.vue rounds it, so the canvas the page makes is exactly this size; and never more than the picture's own pixels, since a thumbnail is never enlarged and the windows scaler, unlike imageio, would enlarge it if asked
fn target(ask: &Ask, width: u32, height: u32) -> Result<(u32, u32), String> {
	let (css_width, css_height, _) = crate::fit::fit_size(&ask.fit, width, height, ask.beam, ask.screen_width, ask.screen_height)?;
	let want_width  = (css_width  as f64 * ask.backing_per_css).round().max(1.0);//the same multiplication and rounding as Math.round(side * backingPerCss) in flowSnap
	let want_height = (css_height as f64 * ask.backing_per_css).round().max(1.0);
	if want_width.max(want_height) >= width.max(height) as f64 { return Ok((width, height)) }//the picture's own pixels, all of them, when the fit's longer side asks for as many or more; decided by the longer side, because a sliver can reach its own short side by rounding while its long side is still being shrunk
	Ok(((want_width as u32).min(width), (want_height as u32).min(height)))//and the short side never past the picture's own either
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
	use super::{Ask, Thumbnail};

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

	pub fn render(path: &str, ask: &Ask, wide: bool) -> Result<Thumbnail, String> {
		let source = open(path)?;
		let (natural_width, natural_height, maximum) = match natural_and_limit(source, ask) {//the picture's size from imageio, before anything decodes, and the longer side the fit wants
			Ok(sizes) => sizes,
			Err(problem) => { unsafe { CFRelease(source) }; return Err(problem) }
		};

		let options: CFDictionary<CFString, CFType> = CFDictionary::from_CFType_pairs(&[
			(key(unsafe { kCGImageSourceThumbnailMaxPixelSize }),          CFNumber::from(maximum as i64).as_CFType()),//the longer side, which is the only size imageio takes: it keeps the aspect, rounds the shorter side itself, and never enlarges
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
		let mut context = CGContext::create_bitmap_context(None, width, height, 8, width * 4, &space, kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);//the crate asserts rather than returns when it cannot make a context, and run_blocking hands that panic to the page as an error. Rgba bytes in that order; premultiplied because that is the only alpha a drawing context accepts, undone below
		context.draw_image(CGRect::new(&CGPoint::new(0.0, 0.0), &CGSize::new(width as f64, height as f64)), &image);//one to one, so this is a color conversion and not a resample
		let mut pixels = context.data().to_vec();
		unpremultiply(&mut pixels);
		Ok(Thumbnail { width: width as u32, height: height as u32, wide, pixels, natural_width, natural_height })
	}

	fn natural_and_limit(source: CGImageSourceRef, ask: &Ask) -> Result<(u32, u32, u32), String> {//the picture's size as it shows, and the longer side in backing pixels to ask imageio for, which is how any fit reaches a library that knows only the square one
		let (width, height, orientation) = properties(source);
		super::check_size(width, height)?;//the stored size, since turning a picture does not change its area
		let (shown_width, shown_height) = if orientation >= 5 { (height, width) } else { (width, height) };//exif's 5 through 8 turn width into height, and the thumbnail is made turned
		let (target_width, target_height) = super::target(ask, shown_width, shown_height)?;
		Ok((shown_width, shown_height, target_width.max(target_height)))
	}

	fn open(path: &str) -> Result<CGImageSourceRef, String> {//an image source over the file, which reads nothing until asked; Create in the name, so the caller releases it
		let url = CFURL::from_path(path, false).ok_or_else(|| format!("thumbnail: not a path: {path}"))?;
		let source = unsafe { CGImageSourceCreateWithURL(url.as_concrete_TypeRef(), std::ptr::null()) };
		if source.is_null() { return Err(format!("thumbnail: ImageIO could not open {path}")) }
		Ok(source)
	}

	fn properties(source: CGImageSourceRef) -> (u32, u32, u32) {//pixel width and height as stored, and the exif orientation, from imageio's properties for the picture; 0 for a size imageio does not give, and 1, upright, for an orientation it does not
		let properties = unsafe { CGImageSourceCopyPropertiesAtIndex(source, 0, std::ptr::null()) };
		if properties.is_null() { return (0, 0, 1) }
		let properties: CFDictionary<CFString, CFType> = unsafe { CFDictionary::wrap_under_create_rule(properties) };//Copy in the name, so the wrapper's drop releases it
		let number = |name: CFStringRef| -> i64 {
			let value = unsafe { CFDictionaryGetValue(properties.as_concrete_TypeRef(), name as *const c_void) };//Get in the name: borrowed, so the wrapper below retains and releases, net nothing
			if value.is_null() { return 0 }
			unsafe { CFNumber::wrap_under_get_rule(value as CFNumberRef) }.to_i64().unwrap_or(0)
		};
		let width = number(unsafe { kCGImagePropertyPixelWidth }).max(0) as u32;
		let height = number(unsafe { kCGImagePropertyPixelHeight }).max(0) as u32;
		let orientation = number(unsafe { kCGImagePropertyOrientation });
		(width, height, if (1..=8).contains(&orientation) { orientation as u32 } else { 1 })
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
	use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
	use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PROPVARIANT};
	use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
	use windows::Win32::System::Variant::VT_UI2;
	use super::{Ask, Thumbnail};

	struct Opened {//one open of one file, and everything the render needs before any pixel is decoded
		_decoder: IWICBitmapDecoder,//nothing reads it: the frame below pulls its pixels through the decoder's stream, so the decoder is held for as long as the frame is
		factory: IWICImagingFactory,
		frame: IWICBitmapFrameDecode,
		width: u32, height: u32,//the size as the pixels are stored, before the orientation turns them
		orientation: u16,//1 through 8, and 1 when the file says nothing
	}

	pub fn render(path: &str, ask: &Ask, _wide: bool) -> Result<Thumbnail, String> {//wide is ignored: everything comes back srgb, because wic has no display p3 context without a profile file, and the header says so
		unsafe {
			let _com = start_com();//first, so it drops last: every com object below is released before com is uninitialized, on every path out, the ? ones included
			let o = open(path).map_err(|e| format!("thumbnail: {path}: {e}"))?;//one open for the whole thumbnail: the size here, the pixels below
			super::check_size(o.width, o.height)?;//the stored size, since turning a picture does not change its area
			let natural = shown(&o);//the picture's size as it shows, once: the fit is worked out from it, and it is sent back for the page
			let target = super::target(ask, natural.0, natural.1)?;//the fit's size in backing pixels, which wic takes exactly, width and height both
			render_com(&o, natural, target).map_err(|e| format!("thumbnail: {path}: {e}"))
		}
	}

	struct Com;//com initialized on this thread, for as long as this value lives. Per render on purpose, settled 2026-10-06 against Microsoft's documentation: a pair around each unit of work is the balance CoInitializeEx asks for and the common shape for work on threads the program did not make, and the cost below, paid once per burst, is about a millisecond against a decode and accepted. Once per thread was weighed and declined both ways it could be had: a thread-local's destructor may run under the loader lock, where CoUninitialize can deadlock, and a runtime of fuji's own with thread hooks would make an ordinary call unusual for one platform
	impl Drop for Com {
		fn drop(&mut self) { unsafe { CoUninitialize() } }//the other half of the initialize below, which windows asks for before a thread exits; the blocking pool retires a thread after ten idle seconds, so a thread left initialized would add up over a session. The cost: the last uninitialize is documented to unload the dlls com loaded, so wic may load again at the start of each burst; renders in flight together keep it loaded for each other
	}

	unsafe fn start_com() -> Option<Com> {//wic is a com library, so every object below arrives through CoCreateInstance, and that answers nothing on a thread that has not said this first. Initialized and uninitialized around each render rather than once per thread, because the blocking pool's threads come and go
		let result = CoInitializeEx(None, COINIT_MULTITHREADED);
		if result.is_ok() { Some(Com) } else { None }//S_OK and S_FALSE, already initialized, each owe one uninitialize; RPC_E_CHANGED_MODE owes none, and com works anyway in the mode the thread already had
	}

	unsafe fn open(path: &str) -> windows::core::Result<Opened> {//the file, opened once: the header is read now and the pixels wait until something asks for them
		let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
		let decoder = factory.CreateDecoderFromFilename(&HSTRING::from(path), None, GENERIC_READ, WICDecodeMetadataCacheOnDemand)?;//wic opens and reads the file itself
		let frame = decoder.GetFrame(0)?;//the first frame, which is the whole picture for anything but an animation
		let (mut width, mut height) = (0u32, 0u32);
		frame.GetSize(&mut width, &mut height)?;
		let orientation = exif_orientation(&frame);
		Ok(Opened { _decoder: decoder, factory, frame, width, height, orientation })
	}

	fn sideways(o: &Opened) -> bool { o.orientation >= 5 }//exif's 5 through 8 are the four orientations that turn width into height, and this is the only line that knows it

	fn shown(o: &Opened) -> (u32, u32) {//the size the picture shows at, which is the stored size turned over when the orientation turns it
		if sideways(o) { (o.height, o.width) } else { (o.width, o.height) }
	}

	unsafe fn render_com(o: &Opened, natural: (u32, u32), target: (u32, u32)) -> windows::core::Result<Thumbnail> {//natural is the picture's size as it shows, sent back for the page; target is the size to make, the same way round, never more than its own pixels
		let (factory, frame) = (&o.factory, &o.frame);
		let (mut width, mut height) = (o.width, o.height);//what the source holds right now, which the scaled decode below can change
		let orientation = o.orientation;

		let (natural_width, natural_height) = natural;
		let (target_width, target_height) = target;
		let (want_width, want_height) = if sideways(o) { (target_height, target_width) } else { (target_width, target_height) };//the size to decode at, in the file's own orientation, before the rotation below

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
		if let Some(profile) = first_color_context(factory, frame) {
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
		Ok(Thumbnail { width: target_width, height: target_height, wide: false, pixels, natural_width, natural_height })
	}

	pub fn memory() -> u64 {
		let mut status = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };//the length field is how windows knows which version of the struct it was handed
		unsafe { if GlobalMemoryStatusEx(&mut status).is_ok() { status.ullTotalPhys } else { 0 } }
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

	fn transform_for(orientation: u16) -> WICBitmapTransformOptions {//exif's eight cases as wic's flags, which wic applies as the flip and then the rotation
		/*
		The order matters for exactly two of the eight, and it is the opposite of what this function first assumed. Five of the cases carry a rotation or a flip but not both, and one is the identity, so six of them read the same whichever way round wic composes them. Only 5 and 7, exif's two mirrored diagonals, carry a rotation and a flip together, and only they can tell the orders apart — a flip then a quarter turn and a quarter turn then that same flip differ by a half turn, which is the whole of the bug this once had.

		Measured on the windows 10 box, 2026-09-13, with eight jpegs authored one per case, each storing the raster that its own orientation turns upright, so a correct thumbnailer shows all eight the same way up. Six were right and 5 and 7 came back rotated 180 degrees, which says wic flips first. Rotating the other way round the circle for those two is the correction, and the eight were re-run against it.
		*/
		let (rotate, flip) = match orientation {
			2 => (WICBitmapTransformRotate0,   WICBitmapTransformFlipHorizontal),
			3 => (WICBitmapTransformRotate180, WICBitmapTransformRotate0),
			4 => (WICBitmapTransformRotate0,   WICBitmapTransformFlipVertical),
			5 => (WICBitmapTransformRotate270, WICBitmapTransformFlipHorizontal),//flipped first, so the quarter turn goes the other way round than a rotation-first reading would put it
			6 => (WICBitmapTransformRotate90,  WICBitmapTransformRotate0),
			7 => (WICBitmapTransformRotate270, WICBitmapTransformFlipVertical),//and the same for this one
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
	use super::{Ask, Thumbnail};

	pub fn render(_path: &str, _ask: &Ask, _wide: bool) -> Result<Thumbnail, String> {
		Err("thumbnail: the operating system's thumbnailer is not used on this platform".into())//linux makes every thumbnail in the web renderer, so the page routes nothing here and this answers only a caller that got it wrong
	}

	pub fn memory() -> u64 {//MemTotal from the kernel's own listing, in kilobytes there
		let text = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
		let line = text.lines().find(|line| line.starts_with("MemTotal:")).unwrap_or("");
		line.split_whitespace().nth(1).and_then(|kb| kb.parse::<u64>().ok()).map(|kb| kb * 1024).unwrap_or(0)
	}
}
