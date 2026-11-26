//the operating system's icon for a kind of file, a folder, a medium or an extension, drawn at the size a row shows it and handed back as a png, and on windows where the system keeps the icon for a kind; brass's icon.rs holds the commands over this, and iconCache.js on the page says what fuji does with it

/*
The icon is asked for by kind and never by path, so the system opens no file to answer and hands back no thumbnail of what one holds. On the Mac the kind is a UTType, and NSWorkspace gives an NSImage holding every size the icon comes in, each at every scale a Mac draws. A medium is the branch macOS files every kind of it under, public.image, public.movie or public.audio, whose icon is Apple's own whatever app is installed to open those kinds; on the Mac mini every picture kind's own icon was already the branch's, and every movie and sound kind's but the ones an installed app brought an icon for, like VLC's for mkv. An extension is its own UTType, the way launch.rs gets one, and its icon is the one the app that opens it brought, or the system's. The icon is drawn into a bitmap of the row's size in backing pixels, with the drawing scaled by backing_per_css, so the image chosen is the one made for that many points at that scale: on a Retina Mac a 16 point icon at 2x, as Finder shows, which is a different picture from the 32 point icon at 1x, as many as a quarter of their pixels apart, measured on the Mac mini on macOS 27.2.

The bitmap is Display P3, and the png carries that profile, so the web engine shows the icon's colors on whatever screen the window is on, the way it shows any picture with a profile; iconCache.js says why a png, rather than the raw pixels thumbnails come back as.

NSWorkspace is not documented for one thread or another, and this runs on the blocking pool like everything else that waits on the system: Chromium asks the same call on its thread pool, with the comment "NSWorkspace is thread-safe", and Apple documents its sibling iconForFiles as safe from any thread.

On Windows the folder, the three media and a file of no kind are the system's stock icons, which SHGetStockIconInfo names by kind as a file and an icon inside it, imageres.dll's on Windows 10, so each version of Windows gives its own drawing of them, and they are the same icons Explorer draws for the folder and for documents of those kinds. Windows registers no medium the way the Mac does, but it keeps an icon for each one. The icons fuji registers for Explorer on the types it offers are these same locations, through location below, so a type fuji holds wears the same icon outside fuji as in it, and fuji carries no icon of its own for one.

Every other extension shows the icon its registered app gave it, which AssocQueryString finds by the extension alone: the location written in the extension's DefaultIcon, a file and the icon inside it, so no file of that kind is ever opened. Where that answers nothing to draw, the system image list answers by the extension instead. That is a kind whose every file draws its own, which DefaultIcon writes as %1, like an .exe or an .ico, where the list gives the kind's generic icon, since fuji asks by kind and never by path; a shortcut, which has no DefaultIcon at all; and a kind a Store app holds, whose icon is a resource in its package rather than a location, where the list gives the app's. Asked for fifteen extensions on the Windows box on 2026-11-09, these calls loaded no DLL into the process but Windows' own; a location is a string until something draws it, and drawing one reads the icon from that file as data, as Explorer does.

A location is drawn by SHDefExtractIcon at exactly the backing pixels asked, which takes the image the file carries for that size, 20 at 125 percent from imageres, or shrinks the next larger, as Explorer's own icon cache does at 28 for 175 percent. The image list comes only in its own few sizes, so WIC shrinks its icon from the smallest that is large enough. Either way WIC turns the icon into a png, sRGB as every Windows icon is, which a png with no profile already means.

The shell's calls want COM, set up here around each call in a single-threaded apartment, which is what SHGetFileInfo's documentation asks for in naming CoInitialize, and how Chromium's icon loader has asked it for a file type's icon, on a thread pool thread set up the same way. The thumbnail sets up a multithreaded apartment around each render for WIC alone, which works in either; the comment above Com in thumbnail.rs says why per call.
*/

/// The system's icon for a kind, css pixels square at backing_per_css backing pixels each, as png bytes. The kind is folder; image, video or audio, the medium's own icon whatever app is installed to open it; or file, the icon for a file of this extension, lowercased and without its dot. An error for a kind it does not know, and where the system has no icon to give, which on linux today is everywhere
pub fn render(kind: &str, extension: &str, css: u32, backing_per_css: f64) -> Result<Vec<u8>, String> { platform::render(kind, extension, css, backing_per_css) }

/// Where Windows keeps its own icon for a kind, written the way a DefaultIcon in the registry takes one, the file and then the icon inside it after a comma: folder; image, video or audio; or file, the blank page of a document of no kind. An error for a kind it does not know, and off windows, where nothing registers an icon by its location
pub fn location(kind: &str) -> Result<String, String> {
	#[cfg(target_os = "windows")]
	{ platform::location(kind) }
	#[cfg(not(target_os = "windows"))]
	{ Err(format!("icon: no location for {kind}, since only windows registers an icon by its location")) }
}

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

#[cfg(target_os = "windows")]
mod platform {
	use windows::core::{Interface, HSTRING, PCWSTR, PWSTR};
	use windows::Win32::Foundation::{E_FAIL, S_OK};
	use windows::Win32::Graphics::Imaging::*;
	use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;
	use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, STREAM_SEEK_END, STREAM_SEEK_SET};
	use windows::Win32::System::Environment::ExpandEnvironmentStringsW;
	use windows::Win32::UI::Controls::{IImageList, ILD_TRANSPARENT};
	use windows::Win32::UI::Shell::*;
	use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, HICON};

	pub fn render(kind: &str, extension: &str, css: u32, backing_per_css: f64) -> Result<Vec<u8>, String> {
		let side = (css as f64 * backing_per_css).round().max(1.0) as u32;
		unsafe {
			let _com = start_com();//first, so it drops last, after every object below
			let icon = match stock(kind, extension)? {
				Some(id) => { let (path, index) = stock_location(id)?; extract(&path, index, side) }
				None => {
					if !nameable(extension) { return Err(format!("icon: not an extension a file on windows could have: {extension}")) }
					registered(extension).and_then(|(path, index)| extract(&path, index, side)).or_else(|| listed(extension, side))//the icon its app registered, and the system image list where that gives nothing to draw
				}
			};
			let icon = icon.ok_or_else(|| format!("icon: windows gave no icon for {}", if kind == "file" { format!(".{extension}") } else { kind.to_string() }))?;
			png(&icon, side).map_err(|e| format!("icon: {e}"))
		}
	}

	pub fn location(kind: &str) -> Result<String, String> {
		let id = stock(kind, "")?.ok_or_else(|| format!("icon: no stock icon for {kind}"))?;//every kind has one, file's being the blank page
		unsafe {
			let _com = start_com();
			let (path, index) = stock_location(id)?;
			Ok(format!("{path},{index}"))//unquoted, as windows writes its own, since a location is split at its last comma
		}
	}

	fn stock(kind: &str, extension: &str) -> Result<Option<SHSTOCKICONID>, String> {//the stock icon windows keeps for a kind, or none for a file of an extension, which shows its registered app's
		Ok(Some(match kind {
			"folder" => SIID_FOLDER,
			"image" => SIID_IMAGEFILES,
			"video" => SIID_VIDEOFILES,
			"audio" => SIID_AUDIOFILES,
			"file" if extension.is_empty() => SIID_DOCNOASSOC,//a document of no kind, the blank page, as the mac gives its generic document
			"file" => return Ok(None),
			_ => return Err(format!("icon: no kind called {kind}")),
		}))
	}

	unsafe fn stock_location(id: SHSTOCKICONID) -> Result<(String, i32), String> {//the file windows keeps a stock icon in and which icon it is there, a negative number being a resource id, which stays put where a position could shift
		let mut info = SHSTOCKICONINFO { cbSize: std::mem::size_of::<SHSTOCKICONINFO>() as u32, ..Default::default() };
		SHGetStockIconInfo(id, SHGSI_ICONLOCATION, &mut info).map_err(|e| format!("icon: windows gave no location for its stock icon {}: {e}", id.0))?;
		Ok((wide(&info.szPath), info.iIcon))
	}

	unsafe fn registered(extension: &str) -> Option<(String, i32)> {//the icon location the app that opens this extension registered for it, found by the extension alone, or none where there is nothing to draw from
		let dotted = HSTRING::from(format!(".{extension}"));
		let mut length = 0u32;
		let _ = AssocQueryStringW(ASSOCF_NONE, ASSOCSTR_DEFAULTICON, &dotted, PCWSTR::null(), None, &mut length);//first only the length, counting the terminating zero, which stays 0 for an extension with no answer at all
		if length == 0 { return None }
		let mut text = vec![0u16; length as usize];
		if AssocQueryStringW(ASSOCF_NONE, ASSOCSTR_DEFAULTICON, &dotted, PCWSTR::null(), Some(PWSTR(text.as_mut_ptr())), &mut length) != S_OK { return None }
		let (path, index) = parse(&wide(&text))?;
		Some((expand(&path), index))
	}

	pub fn parse(location: &str) -> Option<(String, i32)> {//a location as the registry holds one, a file and then after the last comma which icon in it, or none where it names nothing to draw: %1, every file its own, and @, a resource in a Store app's package
		let location = location.trim();
		if location.is_empty() || location.starts_with('@') || location.contains("%1") { return None }
		let (path, index) = match location.rsplit_once(',').map(|(path, index)| (path, index.trim().parse::<i32>())) {
			Some((path, Ok(index))) => (path, index),
			_ => (location, 0),//no number after a comma, as zipfldr.dll is registered, which is the file's first icon
		};
		let path = path.trim().trim_matches('"').trim();//quoted, as some apps write a path with spaces
		if path.is_empty() { None } else { Some((path.to_string(), index)) }
	}

	unsafe fn expand(path: &str) -> String {//a location may name a folder by a variable, like %SystemRoot%, which a registry value of the expandable kind leaves for its reader to fill in
		let source = HSTRING::from(path);
		let length = ExpandEnvironmentStringsW(&source, None);
		if length == 0 { return path.to_string() }
		let mut text = vec![0u16; length as usize];
		if ExpandEnvironmentStringsW(&source, Some(&mut text)) == 0 { return path.to_string() }
		wide(&text)
	}

	fn nameable(extension: &str) -> bool {//an extension a file on windows could have, so the text the page sends names one class in the registry and reaches nothing past it
		!extension.is_empty() && extension.len() <= 255 && !extension.chars().any(|c| c.is_control() || "\\/:*?\"<>|.".contains(c))
	}

	fn wide(text: &[u16]) -> String { String::from_utf16_lossy(&text[..text.iter().position(|&c| c == 0).unwrap_or(text.len())]) }//up to the terminating zero

	struct Icon(HICON);//an icon handle the caller owns, destroyed when this drops
	impl Drop for Icon {
		fn drop(&mut self) { unsafe { let _ = DestroyIcon(self.0); } }
	}

	unsafe fn extract(path: &str, index: i32, side: u32) -> Option<Icon> {//the icon at this location, side backing pixels square: the image the file carries at that size, or the next larger shrunk
		let mut icon = HICON::default();
		let result = SHDefExtractIconW(&HSTRING::from(path), index, 0, Some(&mut icon), None, side);//the size in the low word, for the large icon, the only one asked for
		if result != S_OK || icon.is_invalid() { return None }//S_FALSE where the file holds no such icon
		Some(Icon(icon))
	}

	unsafe fn listed(extension: &str, side: u32) -> Option<Icon> {//the icon the system image list holds for this extension, from the smallest of its lists at least side across, or the largest
		let mut info = SHFILEINFOW::default();
		let name = HSTRING::from(format!("file.{extension}"));//a name with no file behind it, which the file attributes flag says to take as it stands
		if SHGetFileInfoW(&name, FILE_ATTRIBUTE_NORMAL, Some(&mut info), std::mem::size_of::<SHFILEINFOW>() as u32, SHGFI_SYSICONINDEX | SHGFI_USEFILEATTRIBUTES) == 0 { return None }
		let mut best: Option<(IImageList, i32)> = None;
		for list in [SHIL_SMALL, SHIL_LARGE, SHIL_EXTRALARGE, SHIL_JUMBO] {//16, 32, 48 and 256 at 100 percent; small and large grow with windows' own scale
			let Ok(images) = SHGetImageList::<IImageList>(list as i32) else { continue };
			let (mut width, mut height) = (0, 0);
			if images.GetIconSize(&mut width, &mut height).is_err() { continue }
			let better = match &best { None => true, Some((_, size)) => (*size < side as i32 && width > *size) || (width >= side as i32 && width < *size) };
			if better { best = Some((images, width)) }
		}
		let (images, _) = best?;
		images.GetIcon(info.iIcon, ILD_TRANSPARENT.0).ok().map(Icon)
	}

	unsafe fn png(icon: &Icon, side: u32) -> windows::core::Result<Vec<u8>> {//the icon as a png side pixels square, shrunk by wic's fant scaler where it came back larger, as thumbnail.rs shrinks
		let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
		let mut source: IWICBitmapSource = factory.CreateBitmapFromHICON(icon.0)?.cast()?;//its pixels with straight alpha, an old icon's mask turned into alpha
		let (mut width, mut height) = (0u32, 0u32);
		source.GetSize(&mut width, &mut height)?;
		if width != side || height != side {
			let scaler = factory.CreateBitmapScaler()?;
			scaler.Initialize(&source, side, side, WICBitmapInterpolationModeFant)?;
			source = scaler.cast()?;
		}
		let converter = factory.CreateFormatConverter()?;
		converter.Initialize(&source, &GUID_WICPixelFormat32bppBGRA, WICBitmapDitherTypeNone, None, 0.0, WICBitmapPaletteTypeCustom)?;

		let stream = SHCreateMemStream(None).ok_or_else(|| windows::core::Error::from(E_FAIL))?;//a stream in memory for the encoder to write into
		let encoder = factory.CreateEncoder(&GUID_ContainerFormatPng, std::ptr::null())?;
		encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;
		let mut frame = None;
		encoder.CreateNewFrame(&mut frame, std::ptr::null_mut())?;
		let frame = frame.ok_or_else(|| windows::core::Error::from(E_FAIL))?;
		frame.Initialize(None)?;
		frame.SetSize(side, side)?;
		let mut format = GUID_WICPixelFormat32bppBGRA;
		frame.SetPixelFormat(&mut format)?;
		frame.WriteSource(&converter, std::ptr::null())?;
		frame.Commit()?;
		encoder.Commit()?;

		let mut length = 0u64;
		stream.Seek(0, STREAM_SEEK_END, Some(&mut length))?;
		stream.Seek(0, STREAM_SEEK_SET, None)?;
		let mut bytes = vec![0u8; length as usize];
		let mut read = 0u32;
		stream.Read(bytes.as_mut_ptr().cast(), length as u32, Some(&mut read)).ok()?;
		bytes.truncate(read as usize);
		Ok(bytes)
	}

	struct Com;//com initialized on this thread in a single-threaded apartment, for as long as this value lives
	impl Drop for Com {
		fn drop(&mut self) { unsafe { CoUninitialize() } }
	}
	unsafe fn start_com() -> Option<Com> {//the apartment the shell's documentation assumes, set up around each call as the thumbnail sets up its own, for the reasons above Com in thumbnail.rs; with ole 1's dde turned off, the pairing ShellExecute's documentation gives for calling the shell
		let result = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
		if result.is_ok() { Some(Com) } else { None }//S_OK and S_FALSE each owe one uninitialize; RPC_E_CHANGED_MODE owes none, and the calls go on in the apartment the thread already has
	}
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
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

#[cfg(all(test, target_os = "windows"))]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;

	fn side(png: &[u8]) -> (u32, u32) {//the header chunk's width and height, right after its length and its name
		assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
		(u32::from_be_bytes(png[16..20].try_into().unwrap()), u32::from_be_bytes(png[20..24].try_into().unwrap()))
	}

	#[test]
	fn a_folder_at_16_css_pixels_and_1_25_backing_per_css_is_a_20_pixel_png() {
		assert_eq!(side(&render("folder", "", 16, 1.25).unwrap()), (20, 20));
	}

	#[test]
	fn every_kind_the_page_names_comes_back_as_a_png_of_the_size_asked() {//at 100, 175 and 200 percent, 175 being the one imageres carries no image for; txt has a location registered on every windows, exe and ico are drawn by each file, lnk has no location at all, and xyzzy is a kind nothing claims, the last three from the system image list
		let mut wrong = vec![];//every kind that disagrees, so one run shows them all
		for backing_per_css in [1.0, 1.75, 2.0] {
			let want = (16.0 * backing_per_css) as u32;
			for (kind, extension) in [("folder", ""), ("image", ""), ("video", ""), ("audio", ""), ("file", ""), ("file", "txt"), ("file", "exe"), ("file", "ico"), ("file", "lnk"), ("file", "xyzzy")] {
				match render(kind, extension, 16, backing_per_css) {
					Ok(png) if side(&png) == (want, want) => {}
					Ok(png) => wrong.push(format!("{kind} {extension} at {backing_per_css}: {:?}, not {want}", side(&png))),
					Err(e) => wrong.push(format!("{kind} {extension} at {backing_per_css}: {e}")),
				}
			}
		}
		assert!(wrong.is_empty(), "{}", wrong.join("\n"));
	}

	#[test]
	fn a_kind_it_does_not_know_and_an_extension_no_file_could_have_are_refused() {
		assert!(render("spreadsheet", "", 16, 1.0).unwrap_err().contains("no kind called spreadsheet"));
		for extension in [r"jpg\shell", "tar.gz", "a:b", "x\u{1}"] { assert!(render("file", extension, 16, 1.0).is_err(), "{extension:?}") }
		assert!(location("spreadsheet").is_err());
	}

	#[test]
	fn each_kind_has_a_location_in_windows_own_files_that_draws() {//the same locations associate.js registers for Explorer
		for kind in ["folder", "image", "video", "audio", "file"] {
			let said = location(kind).unwrap();
			let (path, index) = platform::parse(&said).unwrap();
			assert!(std::path::Path::new(&path).is_file() && index < 0, "{kind}: {said}");//a file that is there, and a resource id rather than a position
		}
	}

	#[test]
	fn a_registered_location_is_read_the_way_windows_writes_one() {
		let parse = |location: &str| platform::parse(location);
		assert_eq!(parse(r"C:\WINDOWS\system32\imageres.dll,-102"), Some((r"C:\WINDOWS\system32\imageres.dll".into(), -102)));
		assert_eq!(parse(r#""C:\Program Files\VideoLAN\VLC\vlc.exe",0"#), Some((r"C:\Program Files\VideoLAN\VLC\vlc.exe".into(), 0)));
		assert_eq!(parse(r"C:\WINDOWS\system32\zipfldr.dll"), Some((r"C:\WINDOWS\system32\zipfldr.dll".into(), 0)));//no number, the first icon
		assert_eq!(parse(r"C:\a,b\icons.dll"), Some((r"C:\a,b\icons.dll".into(), 0)));//a comma in the path with no number after the last
		assert_eq!(parse("%1"), None);//every file draws its own
		assert_eq!(parse("@{Microsoft.Windows.Photos_8wekyb3d8bbwe?ms-resource://Microsoft.Windows.Photos/Files/Assets/Logo.png}"), None);//a store app's resource
		assert_eq!(parse(""), None);
		assert_eq!(parse(",3"), None);
	}
}
