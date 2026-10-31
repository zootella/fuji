//the pointer command: one of the operating system's own pointers as pixels, from copper on the blocking pool, packed into one buffer for the page to show as a css cursor

use tauri::ipc::Response;
use crate::run_blocking;

//on the blocking pool because it maps ole32.dll from the disk; one buffer, sixteen bytes of header, the width, height and hotspot as little-endian unsigned 32-bit integers, then straight-alpha rgba, top row first, which pointer.js unpacks
#[tauri::command]
pub async fn pointer_read(name: String, backing_per_css: f64) -> Result<Response, String> {
	run_blocking(move || {
		if !(backing_per_css.is_finite() && backing_per_css > 0.0) { return Err(format!("pointer_read: expected backing pixels per css pixel above zero: {backing_per_css}")) }
		let size = (32.0 * backing_per_css).round() as u32;//32 backing pixels square at 100 percent, the size Windows draws its pointers at there, and as many more as the display's scale adds
		let p = copper::pointer::read(&name, size)?;
		let mut bytes = Vec::with_capacity(16 + p.pixels.len());
		for n in [p.width, p.height, p.x, p.y] { bytes.extend_from_slice(&n.to_le_bytes()) }
		bytes.extend_from_slice(&p.pixels);
		Ok(Response::new(bytes))
	}).await
}
