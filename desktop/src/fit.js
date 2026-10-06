//the fits: how big a thumbnail is, from the picture's size and the beam; fit.rs is the same arithmetic in rust, kept in step with this file operation for operation

/*
A fit is the rule that sizes a thumbnail: given a picture's size and the beam, it answers the size the thumbnail shows at. The user picks one fit for the whole sheet and switches with a click on the sheet's toolbar, which remakes every thumbnail. A fit only sizes; where the sized thumbnails go is the flow's business, a separate choice. fits.md is where the fits were planned.

The beam is one length in CSS pixels for the whole sheet, the Small, Medium, Large or Xl the user chose, and every fit uses it the same way, through one imaginary picture. That picture is 4:3 landscape, the beam wide and three quarters of the beam tall, and it never changes; call it the reference. Each fit is nothing but a measure — the longer side, the diagonal, the width plus the height, the square root of the area, the height, or the width — and it scales a picture until the picture's measure matches the reference's. So every fit but row, scale and log shows a 4:3 picture at exactly the reference's size, the beam across, and the fits differ only in what they do with other shapes; nothing in any fit is tuned by hand. Row stands the reference upright, 3:4, so a row's height is the beam just as a column's width is, and it is a portrait 4:3 picture that stays put under it. The scale and log fits use the reference differently, as the one picture that sets the sheet's single scale: a picture the size of the screen comes out measuring as the reference does by its width plus height.

The choice of 4:3 is arbitrary, and made once. It is 800 by 600, the screen of a PC in the 1990s, and the shape of every monitor and television before widescreen; and it is the shape an iPhone shoots its photographs in today, 4032 by 3024. So the commonest pictures in a collection, old screenshots and new phone photographs, stay exactly where they are whichever fit is chosen. And it keeps the arithmetic exact, because three quarters, one and a quarter and one and three quarters are all exact in binary, and a 4:3 picture's diagonal is exactly a quarter more than its width, a 3-4-5 triangle.

Every fit keeps three promises. A guard, where it has one, holds the thumbnail's longer side inside a limit: at most five beams for row, column and area, so a panorama does not run away, and at least half a beam for scale, so a small picture does not vanish. It is never enlarged: the scale is capped at one, so a picture smaller than its fit shows at its own size, which overrules even the scale fit's floor. And it lands on whole CSS pixels, at least one each way, so the canvas the flow makes can be sized to it exactly.

Why every fit is written twice. A native thumbnail, the one the operating system makes, is one call and one open of its file: the page sends a path and does not know the picture's size, and Rust learns it partway through that call, from the header ImageIO or WIC reads just before it makes the thumbnail. The size has to be chosen right then, and on the Mac as a single number, because ImageIO takes a size only as a limit on the longer side and so knows only the square fit; any other fit has to become the longer side that gives its shape. So Rust needs the fits, and fit.rs has them. The page needs them too, for every thumbnail it makes itself and to size every tile. Keeping them only here would mean asking the operating system for a thumbnail larger than any fit could want and shrinking it again in the page, a second resample on the main thread, which is the work the native route exists to avoid.

Why the two must agree to the integer. Every route hands the page the picture's own size in image pixels — the native render sends it back beside the thumbnail's pixels, the page route reads it from the decoded image, and an img tile from the loaded one — and the page runs fitSize on that to size the tile, doing every calculation from the raw fact rather than from a number partly worked out elsewhere. So on the native route the page sizes the tile from its own answer while Rust rendered from its own, and a canvas whose pixels are not exactly its box is resampled by the compositor on every row — the defect fidelity.md measured on half of all native thumbnails. Close is not enough: the same whole numbers in have to give the same whole numbers out, from both files, on every machine.

How they agree. Both languages compute in IEEE 754 double precision, where adding, subtracting, multiplying, dividing and taking a square root are correctly rounded by the standard itself, so the same operations in the same order give the same bits in both, even where the result is not exact. Min and max are exact, and the two languages' rounding agrees for every positive number, which every number rounded here is. Anything else — powers, exponentials, logarithms, hypot, the trigonometric functions — is left to each platform's library and may differ in the last bit, so a fit that seems to need one is rewritten in those operations or is not written; that is why the log fit's curve is a square root. Floating point does not reorder freely, so each line of _fitScale matches its arm in fit.rs operation for operation, parentheses and all, and a change to either file is a change to both, in the same commit.
*/

const fitLongerAtMost = 5//in beams: row, column and area hold a thumbnail's longer side to at most this many
const fitLongerAtLeast = 0.5//in beams: and scale holds it to at least this many
const rustU32Max = 4294967295//the largest u32, which is what fit.rs takes its sizes as, so both accept exactly the same inputs

const fitReferenceHeight = 0.75//the reference picture is 4:3 landscape, its height three quarters of its width, which is the beam

export const fitMeasures = {//each fit's measure of a picture, and the setting's values, which fit.rs matches on; each fit, introduced
	RowFit:     (w, h) => h,//every thumbnail as tall as the beam, and as wide as its picture's shape makes it, held to five beams; the one fit that stands the reference upright, so a row is as tall as a column is wide, and a landscape 4:3 comes out a third larger, 320 by 240 at Medium. Flickr's look: calm and level across, generous to panoramas, and the fit a justified flow wants, since a row of one height can be stretched to meet the bucket's edge
	ColumnFit:  (w, h) => w,//every thumbnail as wide as the reference, the beam, and as tall as its shape makes it, held to five beams. Pinterest's look, the row fit turned on its side: generous to portraits, phone screenshots and long scrolling captures, and the fit a columns flow wants
	SquareFit:  (w, h) => Math.max(w, h),//the longer side, so every picture fits the same square, the beam on a side. What Finder and Explorer do: orderly and familiar, with square pictures getting the most room and panoramas the least
	CircleFit:  (w, h) => Math.sqrt(w * w + h * h),//the diagonal, so every picture fits the same circle, the one through the reference's corners, a beam and a quarter across. It shrinks a picture the nearer it is to square and grows a wider one: softer and more even than the square. One square root rather than math.hypot, which each engine is free to compute its own way
	DiamondFit: (w, h) => w + h,//the width plus the height, so every picture fits the same diamond, a beam and three quarters around. The light table's own measure, so a thumbnail is a small copy of what the table shows, and shapes even out further than under the circle
	AreaFit:    (w, h) => Math.sqrt(w * h),//the square root of the area, so every thumbnail covers the reference's area whatever its shape, held to five beams on a side. The fairest by ink: each picture gets the same share of the sheet
	ScaleFit:   (w, h) => w + h,//one scale for every picture, set so a picture the size of the screen measures as the reference does by width plus height, so thumbnails keep the sizes their pictures have against each other: a big scan is big and an icon is small. Like looking down on the whole folder from a height, held to half a beam on the longer side so nothing vanishes
	LogFit:     (w, h) => w + h,//scale with the differences compressed by a square root: a picture the size of the screen shows as it would under scale, one four times as large across is only twice as large, and an icon stays something you can see. It still says which pictures are big, without letting one dwarf the rest
}
export const fitNames = Object.keys(fitMeasures)//in the order above

/*
fitSize: the size a picture shows at under a fit. It takes one object, with these names:

	fit            text           one of fitNames
	width          image pixels   the picture's width as it shows, after its orientation
	height         image pixels   the picture's height, the same way
	beam           CSS pixels     the beam's length, the 120, 240, 360 or 480 the user chose: the width a 4:3 picture comes out at
	screenWidth    CSS pixels     the screen's width, as screen.width gives it; read only by ScaleFit and LogFit
	screenHeight   CSS pixels     the screen's height, as screen.height gives it; the same
	returns        CSS pixels     {width, height, scale}: the size, whole and at least 1 each way, and the scale that made it, in CSS pixels per image pixel

Image pixels are the picture's own pixels, not a unit of the screen at all. They meet CSS pixels in one place, the cap that is never enlarged, which takes one image pixel as one CSS pixel, as flowFit always has. Nothing here is in backing pixels: a caller that needs them, like the native render's limit, multiplies by devicePixelRatio afterward. The screen's size is read today only as its width plus height, and a fit other than ScaleFit or LogFit need not pass it. Throws on a fit it does not know, on a size that is not a whole number from 1 to rustU32Max, and on ScaleFit or LogFit without the screen's size.
*/
export function fitSize({fit, width, height, beam, screenWidth, screenHeight}) {
	if (!fitNames.includes(fit)) throw new Error(`fit: no fit named ${fit}`)
	for (let [name, value] of Object.entries({width, height, beam})) {
		if (!_fitWhole(value)) throw new Error(`fit: expected ${name} to be a whole number from 1 to ${rustU32Max}: ${value}`)
	}
	if ((fit == 'ScaleFit' || fit == 'LogFit') && !(_fitWhole(screenWidth) && _fitWhole(screenHeight))) throw new Error(`fit: expected the screen's width and height for ${fit}: ${screenWidth}, ${screenHeight}`)

	let scale = _fitScale(fit, width, height, beam, screenWidth + screenHeight)//the sum, exact here and in fit.rs, which adds them as f64 so two u32s cannot overflow
	scale = Math.min(scale, 1)//never enlarged, which also overrules the scale fit's floor
	return {
		width:  Math.max(1, Math.round(width * scale)),//math.round sends a half up and rust's round sends it away from zero, which agree for every positive number, and every number here is positive
		height: Math.max(1, Math.round(height * scale)),
		scale,//before the rounding, so a caller can tell how much detail the picture has for its size, which the rounded size cannot say for a sliver
	}
}

function _fitScale(fit, w, h, b, s) {//css pixels per picture pixel, before never enlarged; s is the screen's width plus height. Each line matches fit.rs operation for operation and in the same order, because floating point is exact only as far as that
	let measure = fitMeasures[fit]
	let rw = b, rh = fitReferenceHeight * b//the reference picture, 4:3 landscape, the beam by three quarters of it
	if (fit == 'RowFit') { rw = fitReferenceHeight * b; rh = b }//stood upright for row, so a row's height is the beam as a column's width is
	let reference = measure(rw, rh)//its measure, exact for every fit but area
	let m = measure(w, h)//and this picture's
	if (fit == 'ScaleFit') return Math.max(reference / s, (fitLongerAtLeast * b) / Math.max(w, h))//one scale for the sheet, a screen-sized picture measuring as the reference, raised for a picture whose longer side would fall under the floor
	if (fit == 'LogFit')   return (reference * Math.sqrt(m / s)) / m//the reference's measure times the square root of the picture's against the screen's, as a scale
	if (fit == 'RowFit' || fit == 'ColumnFit' || fit == 'AreaFit') return Math.min(reference / m, (fitLongerAtMost * b) / Math.max(w, h))//the measure to the reference's, and the longer side no more than the guard
	return reference / m//square, circle and diamond: the measure to the reference's, and nothing more
}

function _fitWhole(n) { return Number.isInteger(n) && n >= 1 && n <= rustU32Max }//a size both languages take the same way
