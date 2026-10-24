//the panel command: how many pixels the main display really has, from copper, on the blocking pool

use crate::run_blocking;
use copper::panel::Arrow;

//the one command javascript calls; the body it reaches was chosen when this was compiled
#[tauri::command]//its body on the blocking pool because linux answers by running xrandr and waiting for it; the mac asks CoreGraphics and windows asks user32, and neither minds which thread asks
pub async fn panel_resolution() -> Arrow {
	run_blocking(|| Ok(copper::panel::resolution())).await.unwrap_or(Arrow { x: 0, y: 0 })//a {x, y} pixel count, or {0, 0} for any error or inability to find the answer, a panic in a body below included
}
