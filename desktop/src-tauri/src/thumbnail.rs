//the thumbnail commands: a thumbnail at a fit's size or the picture at its own, from copper on the blocking pool, packed into one buffer for the page

use tauri::ipc::Response;
use crate::run_blocking;
use copper::thumbnail::{Ask, Thumbnail};

//the one thumbnail, its body on the blocking pool through run_blocking, which keeps a decode off the thread that runs the window and turns a panic anywhere below into an error
#[tauri::command]
pub async fn thumbnail_render(path: String, fit: String, beam: u32, screen_width: u32, screen_height: u32, backing_per_css: f64, gamut: String) -> Result<Response, String> {
	run_blocking(move || {
		if path.trim().is_empty() { return Err("thumbnail_render: expected a path".into()) }//the mistakes a caller could make, returned as errors the page can read; fit.rs checks the fit's own
		if !(backing_per_css.is_finite() && backing_per_css > 0.0) { return Err(format!("thumbnail_render: expected backing pixels per css pixel above zero: {backing_per_css}")) }
		let ask = Ask { fit, beam, screen_width, screen_height, backing_per_css };
		let wide = gamut == "display-p3";//anything else is srgb, which is what a canvas is unless asked
		let t = copper::thumbnail::render(&path, &|width, height| copper::thumbnail::target(&ask, width, height), wide)?;//which holds the wall, because it has the picture's size in hand before it decodes
		Ok(pack(t))
	}).await
}

//the picture at its own size, through the same library, the same wall and the same buffer as thumbnail_render, for a page that has to show a picture its web engine cannot decode; on the blocking pool for the same reasons
#[tauri::command]
pub async fn thumbnail_natural(path: String, gamut: String) -> Result<Response, String> {
	run_blocking(move || {
		if path.trim().is_empty() { return Err("thumbnail_natural: expected a path".into()) }
		let wide = gamut == "display-p3";
		let t = copper::thumbnail::render(&path, &|width, height| Ok((width, height)), wide)?;//every pixel the picture shows, so no fit and nothing to shrink
		Ok(pack(t))
	}).await
}

fn pack(t: Thumbnail) -> Response {//the header, then the pixels, in one buffer so it crosses raw
	let mut bytes = Vec::with_capacity(20 + t.pixels.len());
	bytes.extend_from_slice(&t.width.to_le_bytes());
	bytes.extend_from_slice(&t.height.to_le_bytes());
	bytes.extend_from_slice(&(t.wide as u32).to_le_bytes());
	bytes.extend_from_slice(&t.natural_width.to_le_bytes());
	bytes.extend_from_slice(&t.natural_height.to_le_bytes());
	bytes.extend_from_slice(&t.pixels);
	Response::new(bytes)
}
