//the fits: how big a thumbnail is, from the picture's size and the beam; fit.js is the same arithmetic in javascript, kept in step with this file operation for operation

/*
fit_size is fitSize from fit.js, in Rust. The essay at the top of fit.js is the whole account: what a fit is, why every fit is written in both languages, and the rules that keep the two giving the same integers for the same integers. Read it before changing anything here, and change both files together.

What is Rust's own. The sizes arrive as u32, and every u32 is exact as an f64, as every whole number fit.js accepts is exact as a JavaScript number; fit.js refuses anything above the largest u32, so the two take the same inputs. The screen's width and height are added as f64 rather than as u32, so the sum cannot overflow and comes out exactly what fit.js adds. Rust's round sends a half away from zero where Math.round sends it up, which agree because every number here is positive, and Rust never fuses a multiply and an add unless asked, so the arithmetic below is exactly what it says. A mistake comes back as an Err where fit.js throws.

Who calls it. The native render, through target in thumbnail.rs, between learning the picture's size and making the thumbnail. It takes the picture's width and height from ImageIO's properties or WIC's GetSize, turned for an orientation of 5 to 8; it asks for the fit; and it hands the library the answer in backing pixels, the longer side as ImageIO's one number on the Mac, and the exact width and height to WIC on Windows. It sends back, beside the thumbnail's pixels, the picture's own size in image pixels, after its orientation — never the fit's answer — so the page runs fitSize on that raw size itself, as it does for every other route, and the same size is there for anything else the page wants to show. The conversion to backing pixels and its rounding are part of the agreement too, since the page sizes the canvas from the same numbers.
*/

const LONGER_AT_MOST: f64 = 5.0;//in beams: row, column and area hold a thumbnail's longer side to at most this many
const LONGER_AT_LEAST: f64 = 0.5;//in beams: and scale holds it to at least this many
const REFERENCE_HEIGHT: f64 = 0.75;//the reference picture is 4:3 landscape, its height three quarters of its width, which is the beam

fn fit_measure(fit: &str, w: f64, h: f64) -> Option<f64> {//each fit's measure of a picture, the twin of fitMeasures in fit.js, where each fit is introduced; none for a name it does not know
	match fit {
		"RowFit"     => Some(h),
		"ColumnFit"  => Some(w),
		"SquareFit"  => Some(w.max(h)),
		"CircleFit"  => Some((w * w + h * h).sqrt()),//one square root rather than hypot, which each platform's library is free to compute its own way
		"DiamondFit" => Some(w + h),
		"AreaFit"    => Some((w * h).sqrt()),
		"ScaleFit" | "LogFit" => Some(w + h),
		_ => None,
	}
}

/*
fit_size: the size a picture shows at under a fit, the twin of fitSize in fit.js, whose essay says what each of these means.

	fit             text           one of the names fit.js lists in fitNames
	width           image pixels   the picture's width as it shows, after its orientation
	height          image pixels   the picture's height, the same way
	beam            CSS pixels     the beam's length, the 120, 240, 360 or 480 the user chose: the width a 4:3 picture comes out at
	screen_width    CSS pixels     the screen's width, as the page's screen.width gives it; read only by ScaleFit and LogFit
	screen_height   CSS pixels     the screen's height, as the page's screen.height gives it; the same
	returns         CSS pixels     Ok((width, height, scale)): the size, whole and at least 1 each way, and the scale that made it, in CSS pixels per image pixel

A fit other than ScaleFit or LogFit may pass 0 for the screen. Answers Err for a fit it does not know, a zero width, height or beam, and ScaleFit or LogFit without the screen's size, where fit.js throws on the same.
*/
pub fn fit_size(fit: &str, width: u32, height: u32, beam: u32, screen_width: u32, screen_height: u32) -> Result<(u32, u32, f64), String> {
	if width == 0 || height == 0 || beam == 0 { return Err(format!("fit: expected a width, height and beam of at least 1: {width}, {height}, {beam}")) }
	if (fit == "ScaleFit" || fit == "LogFit") && (screen_width == 0 || screen_height == 0) { return Err(format!("fit: expected the screen's width and height for {fit}: {screen_width}, {screen_height}")) }
	let (w, h, b) = (width as f64, height as f64, beam as f64);//every u32 is exact as an f64, as every whole number fit.js accepts is exact as a javascript number
	let s = screen_width as f64 + screen_height as f64;//the screen's width plus height, added as f64 so two u32s cannot overflow, and exact, as the same sum is in fit.js

	let (mut rw, mut rh) = (b, REFERENCE_HEIGHT * b);//the reference picture, 4:3 landscape, the beam by three quarters of it
	if fit == "RowFit" { rw = REFERENCE_HEIGHT * b; rh = b }//stood upright for row, so a row's height is the beam as a column's width is
	let (Some(reference), Some(m)) = (fit_measure(fit, rw, rh), fit_measure(fit, w, h)) else { return Err(format!("fit: no fit named {fit}")) };//the reference picture's measure, and this picture's

	let scale = match fit {//css pixels per picture pixel, before never enlarged; each arm matches its line in _fitScale in fit.js
		"ScaleFit" => (reference / s).max((LONGER_AT_LEAST * b) / w.max(h)),//one scale for the sheet, a screen-sized picture measuring as the reference, raised for a picture whose longer side would fall under the floor
		"LogFit"   => (reference * (m / s).sqrt()) / m,//the reference's measure times the square root of the picture's against the screen's, as a scale
		"RowFit" | "ColumnFit" | "AreaFit" => (reference / m).min((LONGER_AT_MOST * b) / w.max(h)),//the measure to the reference's, and the longer side no more than the guard
		_ => reference / m,//square, circle and diamond: the measure to the reference's, and nothing more
	};
	let scale = scale.min(1.0);//never enlarged, which also overrules the scale fit's floor
	Ok(((w * scale).round().max(1.0) as u32, (h * scale).round().max(1.0) as u32, scale))//the scale before the rounding, as fit.js returns it
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::arithmetic_side_effects, clippy::panic, clippy::type_complexity)]//a test may index, unwrap and add, and pin a table in one tuple: a mistake in one fails the test, which is what a test is for
mod tests {
	use super::*;

	#[test]
	fn the_rust_twin_gives_what_fit_js_gives() {//every fit over six pictures, pinned to the numbers fitSize in fit.js answered for the same arguments on 2026-10-24, run through node on the Mac mini: a 4:3 picture at the beam, a portrait, a panorama, a picture smaller than the beam, a 1080p frame at the largest beam, and a sliver. The sizes match exactly and the scale to the last place, since both languages do the same operations on the same doubles
		let rows: &[(&str, u32, u32, u32, u32, u32, u32, u32, f64)] = &[
			("RowFit", 640, 480, 240, 1920, 1080, 320, 240, 0.5),
			("ColumnFit", 640, 480, 240, 1920, 1080, 240, 180, 0.375),
			("SquareFit", 640, 480, 240, 1920, 1080, 240, 180, 0.375),
			("CircleFit", 640, 480, 240, 1920, 1080, 240, 180, 0.375),
			("DiamondFit", 640, 480, 240, 1920, 1080, 240, 180, 0.375),
			("AreaFit", 640, 480, 240, 1920, 1080, 240, 180, 0.375),
			("ScaleFit", 640, 480, 240, 1920, 1080, 120, 90, 0.1875),
			("LogFit", 640, 480, 240, 1920, 1080, 147, 110, 0.22912878474779197),
			("RowFit", 480, 640, 240, 1920, 1080, 180, 240, 0.375),
			("ColumnFit", 480, 640, 240, 1920, 1080, 240, 320, 0.5),
			("SquareFit", 480, 640, 240, 1920, 1080, 180, 240, 0.375),
			("CircleFit", 480, 640, 240, 1920, 1080, 180, 240, 0.375),
			("DiamondFit", 480, 640, 240, 1920, 1080, 180, 240, 0.375),
			("AreaFit", 480, 640, 240, 1920, 1080, 180, 240, 0.375),
			("ScaleFit", 480, 640, 240, 1920, 1080, 90, 120, 0.1875),
			("LogFit", 480, 640, 240, 1920, 1080, 110, 147, 0.22912878474779197),
			("RowFit", 3000, 500, 240, 1920, 1080, 1200, 200, 0.4),
			("ColumnFit", 3000, 500, 240, 1920, 1080, 240, 40, 0.08),
			("SquareFit", 3000, 500, 240, 1920, 1080, 240, 40, 0.08),
			("CircleFit", 3000, 500, 240, 1920, 1080, 296, 49, 0.09863939238321437),
			("DiamondFit", 3000, 500, 240, 1920, 1080, 360, 60, 0.12),
			("AreaFit", 3000, 500, 240, 1920, 1080, 509, 85, 0.16970562748477142),
			("ScaleFit", 3000, 500, 240, 1920, 1080, 420, 70, 0.14),
			("LogFit", 3000, 500, 240, 1920, 1080, 389, 65, 0.12961481396815722),
			("RowFit", 100, 100, 240, 1920, 1080, 100, 100, 1.0),
			("ColumnFit", 100, 100, 240, 1920, 1080, 100, 100, 1.0),
			("SquareFit", 100, 100, 240, 1920, 1080, 100, 100, 1.0),
			("CircleFit", 100, 100, 240, 1920, 1080, 100, 100, 1.0),
			("DiamondFit", 100, 100, 240, 1920, 1080, 100, 100, 1.0),
			("AreaFit", 100, 100, 240, 1920, 1080, 100, 100, 1.0),
			("ScaleFit", 100, 100, 240, 1920, 1080, 100, 100, 1.0),
			("LogFit", 100, 100, 240, 1920, 1080, 54, 54, 0.5422176684690383),
			("RowFit", 1920, 1080, 480, 2560, 1440, 853, 480, 0.4444444444444444),
			("ColumnFit", 1920, 1080, 480, 2560, 1440, 480, 270, 0.25),
			("SquareFit", 1920, 1080, 480, 2560, 1440, 480, 270, 0.25),
			("CircleFit", 1920, 1080, 480, 2560, 1440, 523, 294, 0.27236735535142165),
			("DiamondFit", 1920, 1080, 480, 2560, 1440, 538, 302, 0.28),
			("AreaFit", 1920, 1080, 480, 2560, 1440, 554, 312, 0.28867513459481287),
			("ScaleFit", 1920, 1080, 480, 2560, 1440, 403, 227, 0.21),
			("LogFit", 1920, 1080, 480, 2560, 1440, 466, 262, 0.2424871130596428),
			("RowFit", 20, 3000, 120, 1920, 1080, 1, 120, 0.04),
			("ColumnFit", 20, 3000, 120, 1920, 1080, 4, 600, 0.2),
			("SquareFit", 20, 3000, 120, 1920, 1080, 1, 120, 0.04),
			("CircleFit", 20, 3000, 120, 1920, 1080, 1, 150, 0.04999888892592456),
			("DiamondFit", 20, 3000, 120, 1920, 1080, 1, 209, 0.0695364238410596),
			("AreaFit", 20, 3000, 120, 1920, 1080, 4, 600, 0.2),
			("ScaleFit", 20, 3000, 120, 1920, 1080, 1, 210, 0.07),
			("LogFit", 20, 3000, 120, 1920, 1080, 1, 209, 0.06976782688943502),
		];
		let mut wrong = vec![];
		for &(fit, width, height, beam, screen_width, screen_height, want_width, want_height, want_scale) in rows {
			let (w, h, scale) = fit_size(fit, width, height, beam, screen_width, screen_height).unwrap();
			if (w, h) != (want_width, want_height) || (scale - want_scale).abs() > 1e-12 { wrong.push(format!("{fit} {width}x{height} at {beam}: {w}x{h} scale {scale} against {want_width}x{want_height} scale {want_scale}")) }
		}
		assert!(wrong.is_empty(), "{}", wrong.join("\n"));
	}

	#[test]
	fn the_mistakes_a_caller_could_make_are_errors() {//as fit.js throws on the same
		assert!(fit_size("SquareFit", 0, 480, 240, 0, 0).is_err());
		assert!(fit_size("SquareFit", 640, 480, 0, 0, 0).is_err());
		assert!(fit_size("NoSuchFit", 640, 480, 240, 0, 0).is_err());
		assert!(fit_size("ScaleFit", 640, 480, 240, 0, 0).is_err());//the two fits that read the screen refuse a zero screen
		assert!(fit_size("LogFit", 640, 480, 240, 1920, 0).is_err());
		assert!(fit_size("SquareFit", 640, 480, 240, 0, 0).is_ok());//and the others never read it
	}
}
