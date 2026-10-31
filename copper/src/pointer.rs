//the operating system's own pointers, read as pixels for the page to show as a css cursor; brass's pointer.rs holds the command over this
/*
A page sets its pointer with css, and the web engine turns the keyword into a pointer of the system's: on Windows, cursor: pointer is user32's own hand at the system's size. Some pointers have no keyword. While a drag holds something, Windows shows an arrow with a dotted box hanging off it, the same with a plus for a copy, with a curved arrow for a link, and a slashed circle where a drop is refused, and these are OLE's, kept as cursor resources in ole32.dll since drag and drop came to Windows. The keywords nearest them, copy and alias, are pictures the engine carries of its own, and none of those is the dotted box. So this reads OLE's pointer out of the dll as pixels, which the page shows through cursor: url() at the size the display wants, and fuji carries none of Microsoft's drawings.

The dll is mapped as data, the way a resource editor opens one, so none of its code runs, and only from the system folder, so a file of the same name beside fuji is never the one read. Its resource numbers, 1 through 4, are not documented; they were read out of ole32.dll on Windows 10 and named by looking at each picture.

The pixels come from drawing the pointer twice, once over black and once over white. A pixel the pointer leaves clear shows each background, one it covers shows the same color on both, and one it half covers lands between, so the difference between the two drawings is how much of the background shows through, which is the alpha, and the drawing over black is the color times that alpha. That one rule reads every kind of cursor Windows keeps, the two-color ones from before alpha, as OLE's are, and the 32-bit ones with alpha, without parsing either format. A pixel that inverts the screen beneath it, which only two-color cursors can have, comes out opaque white, since an image cannot invert what it covers.
*/

pub struct Pointer {//what a platform hands back, before brass packs it into one buffer
	pub width:  u32,//backing pixels
	pub height: u32,
	pub x: u32,//the hotspot, the pixel that clicks, from the top left
	pub y: u32,
	pub pixels: Vec<u8>,//straight-alpha rgba, width times four bytes a row, top row first
}

/// One of the pointers Windows shows while a drag holds something, by name: move, copy, link or none, drawn size backing pixels square
pub fn read(name: &str, size: u32) -> Result<Pointer, String> {
	if size == 0 || size > 256 { return Err(format!("pointer: expected a size from 1 to 256 backing pixels: {size}")) }
	platform::read(name, size)
}

#[cfg(target_os = "windows")]
mod platform {
	use super::Pointer;
	use std::ffi::c_void;
	use windows::core::{w, PCWSTR};
	use windows::Win32::Graphics::Gdi::{CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};
	use windows::Win32::Foundation::FreeLibrary;
	use windows::Win32::System::LibraryLoader::{LoadLibraryExW, LOAD_LIBRARY_AS_DATAFILE, LOAD_LIBRARY_AS_IMAGE_RESOURCE, LOAD_LIBRARY_SEARCH_SYSTEM32};
	use windows::Win32::UI::WindowsAndMessaging::{DestroyCursor, DrawIconEx, GetIconInfo, LoadImageW, DI_NORMAL, HCURSOR, HICON, ICONINFO, IMAGE_CURSOR, LR_DEFAULTCOLOR};

	pub fn read(name: &str, size: u32) -> Result<Pointer, String> {
		let id: usize = match name {//ole32's cursor resources, as each picture shows
			"none" => 1,//the slashed circle, where a drop is refused
			"move" => 2,//the arrow with a dotted box
			"copy" => 3,//and with a plus in a box
			"link" => 4,//and with a shortcut's curved arrow in a box
			_ => return Err(format!("pointer: expected move, copy, link or none: {name}")),
		};
		unsafe {//unsafe promises the compiler we checked what it cannot: every handle made below is freed below, on every road out
			let module = LoadLibraryExW(w!("ole32.dll"), None, LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE | LOAD_LIBRARY_SEARCH_SYSTEM32).map_err(|e| format!("pointer: ole32.dll did not open: {e}"))?;
			let loaded = LoadImageW(Some(module.into()), PCWSTR(id as *const u16), IMAGE_CURSOR, size as i32, size as i32, LR_DEFAULTCOLOR);//a resource number passed where a name goes, which is what MAKEINTRESOURCE does in C; a cursor drawn at a size its resource lacks is stretched by Windows, as it stretches OLE's own for a program at that scale
			let _ = FreeLibrary(module);//the cursor is a copy, which outlives the dll it came from
			let cursor = HCURSOR(loaded.map_err(|e| format!("pointer: ole32.dll has no cursor {id}: {e}"))?.0);
			let read = draw(cursor, size);
			let _ = DestroyCursor(cursor);
			read
		}
	}

	unsafe fn draw(cursor: HCURSOR, size: u32) -> Result<Pointer, String> {//the cursor's hotspot and pixels
		let mut info = ICONINFO::default();
		GetIconInfo(HICON(cursor.0), &mut info).map_err(|e| format!("pointer: no information for the cursor: {e}"))?;
		if !info.hbmMask.is_invalid() { let _ = DeleteObject(info.hbmMask.into()); }//two bitmaps the call made for us, which are ours to delete, and which the drawings below do not need
		if !info.hbmColor.is_invalid() { let _ = DeleteObject(info.hbmColor.into()); }
		let black = paint(cursor, size, 0x00)?;
		let white = paint(cursor, size, 0xff)?;
		let mut pixels = vec![0u8; (size * size * 4) as usize];
		for (i, out) in pixels.chunks_exact_mut(4).enumerate() {
			let (b, w) = (&black[i * 4..i * 4 + 3], &white[i * 4..i * 4 + 3]);//blue, green and red, in the order a bitmap keeps them
			let alpha = ((0..3).map(|c| 255 - (w[c] as i32 - b[c] as i32)).sum::<i32>() / 3).clamp(0, 255);//how little of the background shows through, from the two drawings
			out[3] = alpha as u8;
			if alpha > 0 { for c in 0..3 { out[2 - c] = ((b[c] as i32 * 255 + alpha / 2) / alpha).min(255) as u8 } }//the color without the alpha drawing over black multiplied in, into red, green and blue
		}
		Ok(Pointer { width: size, height: size, x: info.xHotspot, y: info.yHotspot, pixels })
	}

	unsafe fn paint(cursor: HCURSOR, size: u32, ground: u8) -> Result<Vec<u8>, String> {//the cursor drawn over a background of one gray, as the bitmap's bytes
		let dc = CreateCompatibleDC(None);//a drawing surface in memory, belonging to no window and no screen
		if dc.is_invalid() { return Err("pointer: no drawing surface".into()) }
		let header = BITMAPINFOHEADER {
			biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
			biWidth: size as i32,
			biHeight: -(size as i32),//negative for the top row first, as the page wants it
			biPlanes: 1,
			biBitCount: 32,
			biCompression: BI_RGB.0,
			..Default::default()
		};
		let bitmap_info = BITMAPINFO { bmiHeader: header, ..Default::default() };
		let mut bits: *mut c_void = std::ptr::null_mut();
		let bitmap = match CreateDIBSection(Some(dc), &bitmap_info, DIB_RGB_COLORS, &mut bits, None, 0) {//a bitmap whose bytes this code can read directly
			Ok(bitmap) if !bits.is_null() => bitmap,
			_ => { let _ = DeleteDC(dc); return Err("pointer: no bitmap to draw into".into()) }
		};
		let before = SelectObject(dc, bitmap.into());
		let bytes = std::slice::from_raw_parts_mut(bits as *mut u8, (size * size * 4) as usize);
		bytes.fill(ground);
		let drawn = DrawIconEx(dc, 0, 0, HICON(cursor.0), size as i32, size as i32, 0, None, DI_NORMAL);
		let _ = GdiFlush();//so the drawing has reached the bytes before they are read
		let copy = bytes.to_vec();
		SelectObject(dc, before);
		let _ = DeleteObject(bitmap.into());
		let _ = DeleteDC(dc);
		drawn.map_err(|e| format!("pointer: the cursor did not draw: {e}"))?;
		Ok(copy)
	}
}

#[cfg(not(target_os = "windows"))]
mod platform {//the mac shows its plain arrow while a drag holds something, and linux has no one drag pointer, so the page asks only on windows
	use super::Pointer;
	pub fn read(name: &str, _size: u32) -> Result<Pointer, String> { Err(format!("pointer: no system pointer {name} to read on this platform")) }
}
