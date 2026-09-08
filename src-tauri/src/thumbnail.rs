/*
The operating system's thumbnailer, called from Rust: ImageIO on the Mac and the Windows Imaging Component on Windows, the libraries Finder and Explorer make their own thumbnails with. Given a path and a longest side, each decodes the file scaled — a JPEG at an eighth of its size where the codec allows, skipping most of the inverse transform — resamples the rest of the way with a filter that reads every source pixel, applies the file's orientation, converts its colors into the space asked for, and hands back the small pixels. The full-size raster never exists, nothing here runs on the thread that runs the window, and what crosses to the page is a thumbnail rather than a file. canvas.md weighs this against making the thumbnail in the web layer and says when fuji reaches for it: nothing calls this yet, and CanvasFlow.vue is the thumbnail path on every platform until a measurement says otherwise.

Both libraries take a path or bytes. The path is the right one here: the library reads the file itself, taking what it needs, and the page never holds the file at all — where bytes would mean the page reading the whole file only to hand it back down. The argument is a path the page already holds, which is the same trust disk_read extends and the same contract disk.rs states.

What comes back is one buffer, so the bytes cross as an ArrayBuffer rather than a json array of numbers — performance.md has what that cost once. Eight bytes of header, the width and the height as little-endian unsigned 32-bit integers, then width times height times four bytes of straight-alpha RGBA, top row first, in the requested color space. The page turns that into ImageData and puts it on a canvas in one call; thumbnail.js unpacks it. The size is the size the library produced: the longer side is the maximum asked for, or the picture's own when it was already smaller, because neither path ever enlarges.

The Mac path is short because ImageIO does everything in one call given three options: a maximum pixel size, a flag insisting on a thumbnail rendered from the picture rather than the tiny preview a camera embeds, and a flag applying the file's orientation. Drawing the result into a bitmap context of the wanted color space is where CoreGraphics does the color management, and the only pixel loop of ours undoes the premultiplied alpha that context produces, because ImageData wants straight alpha. The Windows path is longer because WIC is a pipeline of separate objects — decoder, scaled decode, scaler, rotator, color transform, format converter — each initialised over the last, and because WIC does not apply EXIF orientation itself, so the tag is read out of the metadata and turned into a rotation by hand.

Two things the page owns rather than this file. The color space is the page's choice, because only the page knows the screen's gamut: it asks for display-p3 or srgb, and the Windows path answers everything in sRGB, since WIC has no Display P3 context without a profile file. And the fit is the page's: it computes the longest side from the thumbnail box and the pixel ratio, and turns the returned size back into a css size with one rule, longer side to the box, never enlarged.
*/

use tauri::ipc::Response;

pub struct Thumbnail {//what a platform hands back, before it is packed into the one buffer
	pub width:  u32,
	pub height: u32,
	pub pixels: Vec<u8>,//straight-alpha rgba, width times four bytes a row, top row first
}

//the one command; the async in the attribute has tauri run this sync body on its thread pool, which is what keeps a decode off the thread that runs the window
#[tauri::command(async)]
pub fn thumbnail_render(path: String, maximum: u32, gamut: String) -> Result<Response, String> {
	if path.trim().is_empty() { return Err("thumbnail_render: expected a path".into()) }//the two mistakes a caller could make, returned as errors the page sees rather than a panic on a pool thread
	if maximum == 0 { return Err("thumbnail_render: expected a longest side of at least 1".into()) }
	let wide = gamut == "display-p3";//anything else is srgb, which is what a canvas is unless asked
	let t = platform::render(&path, maximum, wide)?;

	let mut bytes = Vec::with_capacity(8 + t.pixels.len());//the header, then the pixels, in one buffer so it crosses raw
	bytes.extend_from_slice(&t.width.to_le_bytes());
	bytes.extend_from_slice(&t.height.to_le_bytes());
	bytes.extend_from_slice(&t.pixels);
	Ok(Response::new(bytes))
}

#[cfg(target_os = "macos")]
mod platform {
	use core_foundation::base::{CFType, TCFType};
	use core_foundation::boolean::CFBoolean;
	use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
	use core_foundation::number::CFNumber;
	use core_foundation::string::{CFString, CFStringRef};
	use core_foundation::url::{CFURL, CFURLRef};
	use core_graphics::base::{kCGBitmapByteOrder32Big, kCGImageAlphaPremultipliedLast};
	use core_graphics::color_space::{kCGColorSpaceDisplayP3, kCGColorSpaceSRGB, CGColorSpace};
	use core_graphics::context::CGContext;
	use core_graphics::geometry::{CGPoint, CGRect, CGSize};
	use core_graphics::image::CGImage;
	use core_graphics::sys::CGImageRef;
	use foreign_types_shared::ForeignType;
	use std::ffi::c_void;
	use std::panic;
	use super::Thumbnail;

	type CGImageSourceRef = *const c_void;//ImageIO's handle to a file it has opened, opaque to us

	#[link(name = "ImageIO", kind = "framework")]
	extern "C" {//declared by hand because no crate fuji has wraps ImageIO; the linker finds the real ones in the framework
		fn CGImageSourceCreateWithURL(url: CFURLRef, options: CFDictionaryRef) -> CGImageSourceRef;
		fn CGImageSourceCreateThumbnailAtIndex(source: CGImageSourceRef, index: usize, options: CFDictionaryRef) -> CGImageRef;
		static kCGImageSourceThumbnailMaxPixelSize: CFStringRef;
		static kCGImageSourceCreateThumbnailFromImageAlways: CFStringRef;
		static kCGImageSourceCreateThumbnailWithTransform: CFStringRef;
	}
	extern "C" {
		fn CFRelease(cf: *const c_void);
	}

	pub fn render(path: &str, maximum: u32, wide: bool) -> Result<Thumbnail, String> {
		panic::catch_unwind(|| render_or_panic(path, maximum, wide))//the crate wrapping CoreGraphics asserts rather than returns on a context it could not make, and a panic on a pool thread has to come back as an error the page can see
			.unwrap_or_else(|_| Err(format!("thumbnail: CoreGraphics could not draw {path}")))
	}

	fn render_or_panic(path: &str, maximum: u32, wide: bool) -> Result<Thumbnail, String> {
		let url = CFURL::from_path(path, false).ok_or_else(|| format!("thumbnail: not a path: {path}"))?;
		let source = unsafe { CGImageSourceCreateWithURL(url.as_concrete_TypeRef(), std::ptr::null()) };//Create in the name, so ours to release below
		if source.is_null() { return Err(format!("thumbnail: ImageIO could not open {path}")) }

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
		Ok(Thumbnail { width: width as u32, height: height as u32, pixels })
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
	use windows::Win32::System::Variant::VT_UI2;
	use super::Thumbnail;

	pub fn render(path: &str, maximum: u32, _wide: bool) -> Result<Thumbnail, String> {//wide is ignored: everything comes back srgb, because wic has no display p3 context without a profile file
		unsafe {
			let _ = CoInitializeEx(None, COINIT_MULTITHREADED);//once per thread and harmless again; a pool thread keeps it for its life, and a thread already in the other mode says so and works anyway
			render_com(path, maximum).map_err(|e| format!("thumbnail: {path}: {e}"))
		}
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
		Ok(Thumbnail { width: target_width, height: target_height, pixels })
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
		Err("thumbnail: the operating system's thumbnailer is not used on this platform".into())//linux stays in the web renderer, which canvas.md decides and the page carries out
	}
}
