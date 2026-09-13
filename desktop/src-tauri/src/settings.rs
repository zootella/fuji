use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::disk;

/*
The early half of fuji's settings, and the only part of them Rust knows about.

The page owns this file. settings.js has the schema, the factory values, the repair of a bad one, the comments it writes back, and the whole of the round trip — it reads fuji.toml at startup, keeps it in memory, and hands the finished text to desktop.rs to be written when the application exits. None of that changes. What this module adds is a second, earlier, much smaller read, for the one question that has to be answered before the page exists.

That question is how big to make the window. The window is built before any JavaScript has run, so fuji used to build a placeholder 800 by 600 and have the page resize it once the settings had been read — which meant the window manager placed a window fuji did not want, and the real one arrived afterwards by growing from its top left corner. So this module exists to ask the right question: what size, before window.rs builds it. It parses fuji.toml, takes two numbers out of it, and ignores everything else in it.

**Reading the size early did not fix a window landing under the taskbar, though it was expected to.** The reasoning was that the operating system would place a window correctly if only it were told the true size, and that is not what Windows does — measured on 2026-09-13, three instances built at their remembered size and Windows still cascaded two of them past the bottom of the work area, one by 6 pixels and one by 40. Its cascade walks a fixed staircase and does not check that what it is placing fits. That is worth knowing here because it is the obvious theory and it is wrong; window.rs answers the taskbar instead, by looking at where the window actually landed and moving it. What reading the file early did buy is a window built once at the size it will keep rather than built wrong and corrected, and a size recorded in a unit that means the same thing on every screen.

**It reads and never writes.** Writing stays with the page and desktop.rs, and keeping it that way is what stops there being two things that believe they own this file. If this module ever learns to write, the next reader will have to work out which half wins.

**It is a partial reader on purpose.** It does not know the schema, does not repair anything, does not supply factory values for the rest, and does not complain about a file it cannot make sense of — it answers "no size recorded" and window.rs opens at a fraction of the screen instead, which is what a first launch does anyway. Every other setting reaches fuji through settings.js exactly as before, and a malformed file is still that module's to report.

**The size is in css pixels, which is the only one of fuji's three pixel units that means the same thing everywhere.** fidelity.md names them: css pixels are what the page and the user's display setting speak, backing bitmap pixels are what macOS renders into and squishes down from, and physical pixels are the panel's own lights. Windows has two of the three, since its backing bitmap is the panel.

Tauri has only two words for this, logical and physical. Its logical is css pixels exactly. Its physical is the backing bitmap on macOS and the real pixels on Windows — one word doing two jobs, because Tauri has no notion of the squish. That is what makes it the wrong unit to write down: a window recorded as 2304 of Tauri's physical pixels on a retina panel is a comfortable 1152 css pixels there, and twice the width of the screen beside it. So the page records css pixels, this reads css pixels, and the window builder is handed css pixels, which is what it wanted all along.
*/

#[derive(Deserialize)]
struct SettingsFile { window: Option<WindowSection> }

#[derive(Deserialize)]
struct WindowSection { width: Option<f64>, height: Option<f64> }

/// The size the settings file remembers for the window, in css pixels, or none if it does not say
pub fn settings_window_size(app: &AppHandle) -> Option<(f64, f64)> {
	let path = settings_path(app)?;
	let bytes = disk::read_bytes(&path).ok()?;//no file yet on a first launch, which is not a problem and not worth a line anywhere
	let text = String::from_utf8(bytes).ok()?;
	let file: SettingsFile = toml::from_str(&text).ok()?;//anything this cannot parse is settings.js's to report, once the page is up and reads the same file properly
	let window = file.window?;
	let (width, height) = (window.width?, window.height?);
	if width > 0.0 && height > 0.0 { Some((width, height)) } else { None }//a zero is how the file says nothing has been recorded yet
}

/// Where fuji.toml lives, which has to agree with settings.js and is checked against it by nothing but this comment
fn settings_path(app: &AppHandle) -> Option<String> {
	let home = app.path().home_dir().ok()?;
	Some(home.join("fuji.toml").to_str()?.to_string())
}
