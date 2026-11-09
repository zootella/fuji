//the icon command: the system's icon for a kind of file, from copper on the blocking pool, as a png for the page

use tauri::ipc::Response;
use crate::run_blocking;

const ICON_LARGEST: f64 = 1024.0;//backing pixels across, past any size a row draws an icon at on any screen, and small enough that a mistake in the page cannot ask for a giant bitmap

/// The system's icon for a kind, which is folder, a medium, image, video or audio, or file with an extension lowercased and without its dot, css pixels square at backing_per_css backing pixels each, as png bytes; its body on the blocking pool through run_blocking, since the system may take its time to answer
#[tauri::command]
pub async fn icon_render(kind: String, extension: String, css: u32, backing_per_css: f64) -> Result<Response, String> {
	run_blocking(move || {
		if !(backing_per_css.is_finite() && backing_per_css > 0.0) { return Err(format!("icon_render: expected backing pixels per css pixel above zero: {backing_per_css}")) }
		if css == 0 || css as f64 * backing_per_css > ICON_LARGEST { return Err(format!("icon_render: expected an icon from 1 to {ICON_LARGEST} backing pixels across, not {css} css pixels at {backing_per_css}")) }
		copper::icon::render(&kind, &extension, css, backing_per_css).map(Response::new)//the bytes raw, as the thumbnail's cross
	}).await
}
