//the operating system's icon for a kind of file, a folder, a medium or an extension, drawn at the size a row shows it and handed back as a png; brass's icon.rs holds the command over this, and iconCache.js on the page says what fuji does with it

/*
The icon is asked for by kind and never by path, so the system opens no file to answer and hands back no thumbnail of what one holds. On the Mac the kind is a UTType, and NSWorkspace gives an NSImage holding every size the icon comes in, each at every scale a Mac draws. A medium is the branch macOS files every kind of it under, public.image, public.movie or public.audio, whose icon is Apple's own whatever app is installed to open those kinds; on the Mac mini every picture kind's own icon was already the branch's, and every movie and sound kind's but the ones an installed app brought an icon for, like VLC's for mkv. An extension is its own UTType, the way launch.rs gets one, and its icon is the one the app that opens it brought, or the system's. The icon is drawn into a bitmap of the row's size in backing pixels, with the drawing scaled by backing_per_css, so the image chosen is the one made for that many points at that scale: on a Retina Mac a 16 point icon at 2x, as Finder shows, which is a different picture from the 32 point icon at 1x, as many as a quarter of their pixels apart, measured on the Mac mini on macOS 27.2.

The bitmap is Display P3, and the png carries that profile, so the web engine shows the icon's colors on whatever screen the window is on, the way it shows any picture with a profile; iconCache.js says why a png, rather than the raw pixels thumbnails come back as.

NSWorkspace is not documented for one thread or another, and this runs on the blocking pool like everything else that waits on the system: Chromium asks the same call on its thread pool, with the comment "NSWorkspace is thread-safe", and Apple documents its sibling iconForFiles as safe from any thread.
*/

/// The system's icon for a kind, css pixels square at backing_per_css backing pixels each, as png bytes. The kind is folder; image, video or audio, the medium's own icon whatever app is installed to open it; or file, the icon for a file of this extension, lowercased and without its dot. An error for a kind it does not know, and where the system has no icon to give, which off the Mac today is everywhere
pub fn render(kind: &str, extension: &str, css: u32, backing_per_css: f64) -> Result<Vec<u8>, String> { platform::render(kind, extension, css, backing_per_css) }

#[cfg(target_os = "macos")]
mod platform {
	use std::ffi::c_void;
	use objc2::Message;//retain, so a kind the system holds as a static is kept the way the extension's own kind is
	use objc2_app_kit::{NSGraphicsContext, NSWorkspace};
	use objc2_core_foundation::{CFDictionary, CFMutableData, CFString};
	use objc2_core_graphics::{kCGColorSpaceDisplayP3, CGBitmapContextCreate, CGBitmapContextCreateImage, CGColorSpace, CGContext, CGImage, CGImageAlphaInfo, CGImageByteOrderInfo};
	use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
	use objc2_uniform_type_identifiers::{UTType, UTTypeAudio, UTTypeData, UTTypeFolder, UTTypeImage, UTTypeMovie};

	#[link(name = "ImageIO", kind = "framework")]
	extern "C" {//declared by hand, as thumbnail.rs declares its own, since no crate fuji has wraps ImageIO
		fn CGImageDestinationCreateWithData(data: &CFMutableData, kind: &CFString, count: usize, options: Option<&CFDictionary>) -> *mut c_void;
		fn CGImageDestinationAddImage(destination: *mut c_void, image: &CGImage, properties: Option<&CFDictionary>);
		fn CGImageDestinationFinalize(destination: *mut c_void) -> bool;
	}
	extern "C" {
		fn CFRelease(cf: *const c_void);
	}

	pub fn render(kind: &str, extension: &str, css: u32, backing_per_css: f64) -> Result<Vec<u8>, String> {
		let asked = match kind {
			"folder" => unsafe { UTTypeFolder }.retain(),
			"image" => unsafe { UTTypeImage }.retain(),
			"video" => unsafe { UTTypeMovie }.retain(),//public.movie, which every video kind sits under; public.video beneath it draws the same icon, and public.audiovisual-content above it is for sound and movies together
			"audio" => unsafe { UTTypeAudio }.retain(),
			"file" if extension.is_empty() => unsafe { UTTypeData }.retain(),//a file with no extension, which gets the system's generic document
			"file" => UTType::typeWithFilenameExtension(&NSString::from_str(extension)).ok_or_else(|| format!("icon: no kind of file for {extension}"))?,//an extension nothing claims still gets a kind of its own, and the generic document with it
			_ => return Err(format!("icon: no kind called {kind}")),
		};
		let icon = NSWorkspace::sharedWorkspace().iconForContentType(&asked);

		let side = (css as f64 * backing_per_css).round().max(1.0) as usize;
		let space = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceDisplayP3 })).ok_or("icon: no Display P3 color space")?;
		let context = unsafe { CGBitmapContextCreate(std::ptr::null_mut(), side, side, 8, side * 4, Some(&space), CGImageAlphaInfo::PremultipliedLast.0 | CGImageByteOrderInfo::Order32Big.0) }.ok_or("icon: no bitmap to draw into")?;//null for the data, so coregraphics keeps the pixels itself
		CGContext::scale_ctm(Some(&context), backing_per_css, backing_per_css);//points to backing pixels, which is what makes the icon choose the image made for this size at this scale
		NSGraphicsContext::saveGraphicsState_class();
		NSGraphicsContext::setCurrentContext(Some(&NSGraphicsContext::graphicsContextWithCGContext_flipped(&context, false)));//this thread's own, so nothing another thread draws comes near it
		icon.drawInRect(NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(css as f64, css as f64)));
		NSGraphicsContext::restoreGraphicsState_class();

		let image = CGBitmapContextCreateImage(Some(&context)).ok_or("icon: no image from the bitmap")?;
		png(&image)
	}

	fn png(image: &CGImage) -> Result<Vec<u8>, String> {//the image encoded by imageio, which writes the image's color space into the file as its profile
		let data = CFMutableData::new(None, 0).ok_or("icon: no buffer for the png")?;
		let destination = unsafe { CGImageDestinationCreateWithData(&data, &CFString::from_static_str("public.png"), 1, None) };
		if destination.is_null() { return Err("icon: ImageIO would not make a png".into()) }
		unsafe { CGImageDestinationAddImage(destination, image, None) };
		let finished = unsafe { CGImageDestinationFinalize(destination) };
		unsafe { CFRelease(destination) };//Create in the name, so released here, on both paths below
		if !finished { return Err("icon: ImageIO could not finish the png".into()) }
		Ok(data.to_vec())
	}
}

#[cfg(not(target_os = "macos"))]
mod platform {
	pub fn render(_kind: &str, _extension: &str, _css: u32, _backing_per_css: f64) -> Result<Vec<u8>, String> {
		Err("icon: this platform's icons are not asked for yet".into())
	}
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use std::ffi::c_void;
	use objc2_core_foundation::{CFData, CFDictionary, CFString};

	#[link(name = "ImageIO", kind = "framework")]
	#[allow(clashing_extern_declarations)]//thumbnail.rs declares CGImageSourceCopyPropertiesAtIndex in the core-foundation crate's types and this in objc2's, the same pointers either way
	extern "C" {
		fn CGImageSourceCreateWithData(data: &CFData, options: Option<&CFDictionary>) -> *mut c_void;
		fn CGImageSourceCopyPropertiesAtIndex(source: *mut c_void, index: usize, options: Option<&CFDictionary>) -> *const c_void;
		static kCGImagePropertyProfileName: &'static CFString;
	}
	#[link(name = "CoreFoundation", kind = "framework")]
	extern "C" {
		fn CFDictionaryGetValue(dictionary: *const c_void, key: *const c_void) -> *const c_void;
		fn CFRelease(cf: *const c_void);
	}

	fn profile_name(png: &[u8]) -> String {//the name of the color profile a png carries, as imageio reads it back, which is the name macOS's sips reports for the file
		let data = CFData::from_bytes(png);
		let source = unsafe { CGImageSourceCreateWithData(&data, None) };
		assert!(!source.is_null(), "imageio could not read the png back");
		let properties = unsafe { CGImageSourceCopyPropertiesAtIndex(source, 0, None) };
		assert!(!properties.is_null(), "imageio read the png back with no properties");
		let name = unsafe { CFDictionaryGetValue(properties, kCGImagePropertyProfileName as *const CFString as *const c_void) };
		let said = if name.is_null() { String::new() } else { unsafe { &*(name as *const CFString) }.to_string() };
		unsafe { CFRelease(properties); CFRelease(source) };
		said
	}

	#[test]
	fn a_folder_at_16_css_pixels_and_2_backing_per_css_is_a_32_pixel_png_in_display_p3() {
		let png = render("folder", "", 16, 2.0).unwrap();
		assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
		let width = u32::from_be_bytes(png[16..20].try_into().unwrap());//the header chunk's width and height, right after its length and its name
		let height = u32::from_be_bytes(png[20..24].try_into().unwrap());
		assert_eq!((width, height, profile_name(&png).as_str()), (32, 32, "Display P3"));
	}

	#[test]
	fn an_extension_nothing_claims_still_gets_an_icon() {
		let png = render("file", "xyzzy", 16, 1.0).unwrap();
		assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 16);
	}

	#[test]
	fn every_kind_the_page_names_comes_back_as_a_png_of_the_size_asked() {
		let mut wrong = vec![];//every kind that disagrees, so one run shows them all
		for (kind, extension) in [("folder", ""), ("image", ""), ("video", ""), ("audio", ""), ("file", "pdf"), ("file", "")] {
			match render(kind, extension, 16, 2.0) {
				Ok(png) if &png[..8] == b"\x89PNG\r\n\x1a\n" && u32::from_be_bytes(png[16..20].try_into().unwrap()) == 32 => {}
				Ok(_) => wrong.push(format!("{kind} {extension}: not a 32 pixel png")),
				Err(e) => wrong.push(format!("{kind} {extension}: {e}")),
			}
		}
		assert!(wrong.is_empty(), "{}", wrong.join("\n"));
	}

	#[test]
	fn a_kind_it_does_not_know_is_refused() {
		assert!(render("spreadsheet", "", 16, 1.0).unwrap_err().contains("no kind called spreadsheet"));
	}
}
