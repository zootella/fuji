# Fits

How the contact sheet decides how big each thumbnail is, and the several ways it could. Begun 2026-10-05 on the Mac mini, built the same day, and tried there by the user; not yet run on Windows. Every fit is written in `fit.js` and `fit.rs`, every route sizes its tile with `fitSize` from the picture's own size, the native render chooses its size with `fit_size`, and `thumbnail.fit` in `fuji.toml` holds the fit, `SquareFit` at the factory. The flow is still wrap, the only one. The user means to add fits to the list below as they come. A spike of the sheet's toolbar, at the top of `Sheet.vue`, has a radio button for each fit, and a click writes the setting and remakes every thumbnail on the sheet; buttons for the beam lengths beside them are still to come, and whether a click should write the setting or hold a try-out until the user keeps one is still open. Once built, the part a user needs becomes a User Guide page on the site, and the site's Thumbnails page and its glossary change with it.

**Beam** is the length every thumbnail is measured against: a quantity of CSS pixels, one for the whole sheet. It has one meaning under every fit, through an imaginary picture chosen once and never changed, the **reference**: 4:3 landscape, the beam wide and three quarters of the beam tall. Each fit measures a picture its own way and scales it until that measure matches the reference's, so every fit but row, scale and log shows a 4:3 picture at exactly the beam across, 240 × 180 at Medium; row stands the reference upright, so a row is as tall as a column is wide. 4:3 is 800 × 600, the screen of a 1990s PC and every monitor before widescreen, and the shape an iPhone shoots today, so the commonest pictures in a collection stay put whichever fit is chosen; and it keeps the arithmetic exact. The user's choice, 2026-10-05, replacing a constant tuned per fit — first a square picture at the beam, then a 3:2 photograph for the circle — with one rule. "Size" was the name before and is retired from this subject, because fuji sizes things everywhere; beam names this one number and nothing else, and the user's own notes on fits used it first. The code holds it as `flowBeam`, read from `settingsThumbnailBeam()`.

**Beam lengths** are the choices the user clicks between, Small, Medium, Large and Xl, each a number of CSS pixels the user can set in `fuji.toml`: 120, 240, 360 and 480 at the factory. Raise them all to make every thumbnail bigger, or only Xl to make the largest choice larger. The choice is the setting `thumbnail.beam`, and the numbers are `thumbnail.small` through `thumbnail.xl`. One set of lengths serves every fit, because the reference ties them together: a click between fits leaves a 4:3 picture where it was and changes only how other shapes sit around it, except under row, where a landscape one grows a third.

**Fit** is the rule that turns a picture's own size and the beam into the size its thumbnail shows at, chosen for the whole sheet in the setting `thumbnail.fit`, `SquareFit` at the factory, and on the sheet's toolbar. A fit is only a measure: the longer side, the diagonal, the width plus the height, the square root of the area, the height, or the width, held in `fitMeasures` in `fit.js` and `fit_measure` in `fit.rs`. Every fit keeps three promises. Never enlarged: a picture smaller than its fit stays at its own size, one image pixel to one CSS pixel. Never cropped: a fit decides only a scale, and every pixel of the picture is in its thumbnail. Whole CSS pixels, so `flowSnap` and `flowEdge` keep every canvas one backing pixel to one. Some fits also carry a guard on the longer side: at most five beams for row, column and area, so a panorama does not run away, and at least half a beam for scale, so a small picture does not vanish. The pictures to imagine going through every fit are the really small, the really big and the really sharp. Scale and log also read the screen's size in CSS pixels, which goes stale on a change of monitor as `devicePixelRatio` does. Below, *b* is the beam and *w* by *h* the picture's size as it shows, after its orientation, and each example is at Medium, *b* = 240, for a picture large enough that the cap at its own size does not bite, with a screen of 1920 × 1080 for scale and log. The fits come from the user's notes, which called them sizes.

**Flow** is what places the thumbnails a fit has sized, chosen independently of the fit: the user's distinction, 2026-10-05. `structure.md` already has the word: a flow arranges within one bucket, and one flow governs every bucket. The fit is now a function the flow calls, through `flowFit`, and `TestFlow` is only the wrap flow, which places tiles left to right like words, top-aligned; it was `SquareFlow` until 2026-10-06, a name from when it was the square fit too, and a flow setting comes beside `thumbnail.fit` once there is a second. Other flows, none planned: grid, every tile centered in a cell the beam on each side, as Finder's icon view does; justified, Flickr's, each row stretched until its right edge meets the bucket's; and columns, Pinterest's masonry, each tile dropping into the shortest column. Some pairings are natural — grid with the square fit, justified with the row fit, columns with the column fit — and any is allowed. A justified flow scales a whole row after the fit, so a flow may change the size a fit chose; whether that is any flow's right or justified's alone is open, and either way never enlarged has to hold in the flow too, or a row of small pictures is stretched past its own pixels.

**The native route** was the one place a fit was hard. The page used to ask Rust for "the longer side at most *N* backing pixels", which is the square fit and needs no knowledge of the picture's shape. Every other fit's longer side depends on the shape, which the page learns only when the thumbnail comes back, while Rust learns it from the header just before it decodes. Settled by the user, 2026-10-05, in three decisions. A thumbnail is one call and one open of its file, so the page cannot ask for the size first and render second — a thousand extra reads for a thousand thumbnails, the probe's cost one tile at a time. Every fit is therefore written twice, by name, in `fit.js` and `fit.rs`, which must give the same whole numbers for the same whole numbers; the essay in `fit.js` says why and how, and a parity check of the two over 2,360,024 cases, run in a scratch harness outside the repository, found them identical. And the native render sends back, beside the thumbnail's pixels, the picture's own size in image pixels after its orientation, never the fit's answer: every route then hands the page the same raw fact — the page route from the decoded image, an img tile from the loaded one — and the page runs `fitSize` on it, so every calculation happens at the end rather than from a number partly worked out elsewhere, and the size is there for features such as a caption under each thumbnail. Rust runs `fit_size` on the header's size only to choose how large to render. The page route needed nothing new; an img tile, a GIF or an SVG, now takes its size from the fit once the engine has loaded it, in `flowImgLoad`, where it used to be sized by `max-width` and `max-height` at the beam, which can say only the square fit, and which stay as the fallback for an SVG with no size of its own.

**How each platform is told the size, as the code stands.** The page sends the fit's name, the beam, the screen's size and `devicePixelRatio`. Rust learns the picture's size partway through the call, refuses a picture whose size the library will not give, runs `fit_size`, and turns its CSS answer into backing pixels in `target` in `thumbnail.rs`, rounding as `flowSnap` does and never past the picture's own pixels. The two platforms take that differently.

- **The Mac: ImageIO assumes the square fit.** Its only control over size is `kCGImageSourceThumbnailMaxPixelSize`, a limit on the longer side; it keeps the aspect, never enlarges, and rounds the other side itself, so it cannot be given a width and a height. `natural_and_limit` in `thumbnail.rs` reads the size and the EXIF orientation from ImageIO's properties for the picture before the thumbnail is made, within the same single open, and passes the target's longer side as that one number. ImageIO's rounding of the shorter side is its own, which is the one place the canvas and the thumbnail can disagree, below.
- **Windows: a sequence of WIC calls, with the size discovered partway through.** `CreateDecoderFromFilename` opens the file, and `GetFrame` and `GetSize` give the picture's size, with its orientation from the metadata. The target's exact width and height go to WIC: first `GetClosestSize` for the codec's scaled decode, then the Fant scaler's `Initialize`. WIC takes any size it is given, so the thumbnail is exactly the canvas the page makes.

**What moving to SquareFit changed, simulated on 2026-10-05 against the code before it.** The page route sizes every tile exactly as before. A native tile can come out one CSS pixel different from before, about one tile in twenty at a `devicePixelRatio` of 2 and one in fourteen at Windows' fractional scales, because the tile is now sized from the picture's own size rather than from the thumbnail that came back; the user accepted that as the more correct answer. On Windows, every canvas is now exactly its thumbnail, where up to one in ten was not, and the one-row clipping the old code had at 175 percent is gone. On the Mac at a ratio of 2, about one native thumbnail in twenty comes back from ImageIO one backing pixel longer on its shorter side than the canvas the page sizes from the fit, so its last row or column is cut off; the old code sized the canvas from the thumbnail and never cut one. That figure rests on assuming ImageIO rounds the shorter side to the nearest pixel, which has not been measured, and it is open.

## Row

The height, to the beam: every thumbnail is 240 tall and as wide as its shape makes it, as Flickr does. The one fit that stands the reference upright, 3:4, so a row's height is the beam just as a column's width is; under it a portrait 4:3 stays put and a landscape one comes out a third larger. The user's change, 2026-10-05, from the landscape reference's three quarters of a beam, which made rows shorter than columns are wide. Guarded: the longer side no more than five beams.

- 3:4 is 180 × 240, 1:1 is 240 × 240, and 4:3 is 320 × 240
- 3:2 is 360 × 240, and 16:9 is 427 × 240
- 4:1 is 960 × 240
- 10:1 would be 2400 wide, so the guard makes it 1200 × 120

What makes Flickr look like Flickr is the justified flow, which stretches each row of row-fitted tiles to the bucket's width; it needs every size in a row before the row can be laid out, which each tile supplies as it arrives, since every route hands the page the picture's own size, though a row would still settle as its last tile lands.

## Column

The width, to the reference's: the beam, so every thumbnail is 240 wide and as tall as its shape makes it, as Pinterest does; the row fit turned on its side, and the fit the columns flow wants. Guarded: the longer side no more than five beams.

- 4:3 is 240 × 180, 1:1 is 240 × 240, and 3:4 is 240 × 320
- 2:3 is 240 × 360
- 1:10 would be 2400 tall, so the guard makes it 120 × 1200

## Square

The longer side, to the reference's: the beam, so every picture fits the same square. What Finder and Explorer do, and what fuji did before fits; bit for bit the same arithmetic as the old `flowFit`.

- 4:3 is 240 × 180, 1:1 is 240 × 240
- 3:2 is 240 × 160, and 16:9 is 240 × 135
- 4:1 is 240 × 60

A square picture gets the most room of any fit, and a panorama the least.

## Circle

The diagonal, to the reference's: a beam and a quarter, exactly, since the reference is a 3-4-5 triangle. Every picture fits the circle through the reference's corners. The user tried a diagonal of the beam alone, which made every circle thumbnail smaller than the square's, then the beam times √2, which made most of them larger, then √13/3, anchored on a 3:2 photograph, before the reference made every fit anchor on 4:3.

- 4:3 is 240 × 180, 1:1 is 212 × 212
- 3:2 is 250 × 166, and 16:9 is 261 × 147
- 4:1 is 291 × 73

The nearer a picture is to square, the more it shrinks against the square fit, and the wider, the more it grows, so shapes come much closer in area than under it.

## Diamond

The width plus the height, to the reference's: a beam and three quarters. Every picture fits the same diamond, a square standing on one vertex, whose four edges pass through its four corners. This is the light table's own measure: `DiamondTable.vue` keeps `quiverA.diamond` as a picture's width plus height.

- 4:3 is 240 × 180, 1:1 is 210 × 210
- 3:2 is 252 × 168, and 16:9 is 269 × 151
- 4:1 is 336 × 84

Square, circle and diamond are one formula with one number changed — the largest coordinate, the straight-line length, and the sum of the coordinates, whose unit shapes are a square, a circle and a diamond. Worth a sentence on the site.

## Area

The square root of the area, to the reference's: every thumbnail covers the reference's 240 × 180, whatever its shape. Guarded: the longer side no more than five beams.

- 4:3 is 240 × 180, 1:1 is 208 × 208
- 3:2 is 255 × 170, and 16:9 is 277 × 156
- 4:1 is 416 × 104
- 30:1 is 1138 × 38, and 40:1 would be 1315 wide, so the guard makes it 1200 × 30

## Scale

One scale for every picture on the sheet, so thumbnails keep the sizes the pictures have relative to each other. The scale is set so that a picture the size of the screen in CSS pixels measures as the reference does by its width plus height, a beam and three quarters, and every other picture is shown at that same scale, larger or smaller. Guarded: the longer side no less than half a beam.

On a screen of 1920 by 1080 CSS pixels the scale is 420 / 3000, 0.14:

- a 6000 × 4000 photograph is 840 × 560
- a 1920 × 1080 screenshot is 269 × 151
- an 800 × 600 picture would be 112 × 84, and the guard raises it to 120 × 90, reading half the beam as the longer side; the session's reading, for the user to confirm
- a 64 × 64 icon would be 9 × 9, and the guard would raise it past its own size, which never enlarged forbids, so it shows at 64 × 64

Open: nothing guards the large end, and a 20,000 × 15,000 scan is 2800 × 2100 at this scale, far wider than the bucket.

## Log

The scale fit with the differences compressed: a picture the size of the screen shows exactly as under the scale fit, a picture half as big has a thumbnail that is not half as small, and one twice as big has a thumbnail that is not twice as big. The aim is that a favicon does not become a dot, and a really huge picture is noticeably large in the flow without dwarfing the others. No guard; the curve is the protection, and never enlarged still holds, so a tiny picture is not blown up either.

The thumbnail's width plus height is the reference's times the square root of the picture's width plus height against the screen's, a power of one half. The user's notes left the math open; a square root is what the parity rules in `fit.js` allow, since every other curve is left to each platform's library. At a power of one this would be the scale fit and at zero the diamond fit, so the log fit sits halfway between them. On the same 1920 by 1080 screen:

- a 6000 × 4000 photograph is 460 × 307, against 840 × 560 under scale
- a 1920 × 1080 screenshot is 269 × 151, the same as under scale
- an 800 × 600 picture is 164 × 123, against 120 × 90
- a 64 × 64 icon is 43 × 43, against its own 64 × 64
- a 20,000 × 15,000 scan is 820 × 615, against 2800 × 2100
