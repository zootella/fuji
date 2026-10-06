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
