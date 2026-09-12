# The thumbnail pipeline

Fuji shows a folder as a contact sheet: hundreds of images at once, and it can assume nothing about any of them. One may be a sixteen-pixel icon of a few hundred bytes. The next may be 6240 by 4160, six megabytes on disk and a hundred and four megabytes once it is decoded. This is how one of those files becomes one small picture on that contact sheet: what decides the route it takes, what each route costs, and what Fuji does to keep the picture that arrives sharp, correctly colored, and undistorted.

Fuji is a desktop application built in two halves. A Rust program owns the window, the files, and anything the operating system will do on Fuji's behalf; a web page draws everything the user sees, rendered by [WebKit](https://webkit.org) on macOS, [Chromium](https://www.chromium.org) on Windows and [WebKitGTK](https://webkitgtk.org) on Linux. [Tauri](https://tauri.app) joins the two and carries calls and bytes between them. Nearly every decision below comes down to which half should do a piece of the work.

Fuji's words for its own parts:

- **Contact sheet** — one folder seen whole, as one top-to-bottom scroll. Fuji's code calls it the sheet.
- **Light table** — the other way Fuji shows images: one at a time, on a plane the user pans and zooms around. Fuji's code calls it a table.
- **Flow** — the code that turns a list of file paths into thumbnails: the routing, the sizing, and the arranging.
- **Tile** — one thumbnail as it sits on the contact sheet: either a `<canvas>` Fuji painted or an `<img>` the engine draws.
- **Box** — the square one tile is fitted inside, whatever shape the picture is. One of four sizes, and the user picks which.
- **Store** — the one place Fuji keeps a file's bytes and its decoded pixels. It hands them out and takes them back when told, and decides nothing on its own.

A light table's route to the screen is deliberately plain: an `<img>`, sized by two lines of CSS, flipped by hiding one element and showing another. Nothing there needs to be clever. The contact sheet cannot do the same thing. The rest of this is why, and what it does instead.

## Why a contact sheet cannot do what a light table does

That second file is the one to keep in mind. Six megabytes of JPEG becomes 6240 × 4160 × 4 bytes once a decoder undoes the compression, which is a hundred and four megabytes of raster for one picture. Two hundred of those on screen at once would be twenty gigabytes.

The obvious answer, letting the engine worry about it, is better than it sounds. Both engines decode a large picture small when they know it will be painted small, called *subsampling*. Chromium decodes a JPEG at a half, a quarter or an eighth of its size for a small paint. WebKit subsamples by a rule of its own: the level for a draw is the power of two nearest the reduction, and the deepest level allowed is the first at which the frame falls under five million pixels, so a 6240 by 4160 raster is decoded at a quarter of its width, 1560 by 1040, and a 4000 by 3000 one at a half. An `<img>` thumbnail is therefore far smaller than a full decode in both engines, and a wall of images that scrolls is the most ordinary thing on the web — the engines have spent twenty years making it work.

So the case for doing it ourselves is not about the number of bytes. **It is about who owns them.**

An `<img>`'s decoded pixels are the engine's to keep or to drop, on a byte budget it does not publish. Under that budget the engine holds a hidden picture's decoded frame, and showing it again takes about fifteen milliseconds. Over the budget the engine throws the frame away, and showing that same picture again means reading and decoding the file from scratch, a quarter of a second for a large one. The budget covers everything else the page is holding as well. Fuji cannot ask what it is, cannot ask how much of it is spent, and the engine tells it nothing when it drops a frame — so the only move Fuji has is to hold less and hope that is enough.

A canvas works the other way round. When Fuji sets a canvas to 480 by 320 device pixels, the engine allocates 480 × 320 × 4 bytes — 600 kilobytes — and gives Fuji a surface to paint on. Those bytes become the only copy of that thumbnail in the process: Fuji draws the picture in, then releases the file it came from, so nothing in the page points at the original any more.

That is what the engine cannot work around. It drops an `<img>`'s decoded frame only because it can decode the file again when the picture comes back on screen; here it would have nothing to decode. So the 600 kilobytes stand for exactly as long as the canvas element stands. They go when the element goes — when the user scrolls that stretch of the contact sheet away and Fuji tears the tile down, or when Fuji quits — and nothing else frees them, however hard the machine is squeezed for memory.

What Fuji gets for that is a number it can work out before it opens a file:

| thumbnail size | one thumbnail | two hundred of them |
| --- | --- | --- |
| Small, 120 px | 150 KB | 30 MB |
| Medium, 240 px | 600 KB | 120 MB |
| Large, 360 px | 1.35 MB | 270 MB |
| XL, 480 px | 2.4 MB | 480 MB |

At a [device pixel ratio](https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio) of 2, for an image three units wide by two tall. On Windows at 150% the same numbers are a little over half of these.

Two hundred Medium thumbnails come to a hundred and twenty megabytes where two hundred full decodes would be twenty gigabytes. The saving is not really the point, since the engine was never going to hold twenty gigabytes either; the point is that Fuji knows the figure. So **every raster thumbnail in Fuji is a canvas**, whichever route made its pixels, and Fuji adds their bytes up as it goes. That turns "how much is the contact sheet holding" from a question about the engine's internals into a number Fuji can print.

## The fork

Every file goes down the same three decisions:

<div style="overflow-x:auto;margin:2rem 0">
<svg viewBox="0 0 640 520" role="img" aria-label="Each file passes a probe, then its extension chooses an img tile or a canvas tile, then the platform's list chooses which decoder fills the canvas" style="width:100%;min-width:480px;max-width:640px;height:auto;font-family:var(--vp-font-family-base, system-ui, sans-serif)">
	<defs>
		<marker id="forkArrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
			<path d="M 0 1 L 9 5 L 0 9 z" fill="currentColor" fill-opacity="0.45" />
		</marker>
	</defs>
	<g fill="none" stroke="currentColor" stroke-opacity="0.45" marker-end="url(#forkArrow)">
		<path d="M 170 62 L 170 96" />
		<path d="M 170 160 L 170 204" />
		<path d="M 170 250 L 170 294" />
		<path d="M 170 350 L 170 394" />
		<path d="M 310 128 L 372 128" />
		<path d="M 310 227 L 372 227" />
		<path d="M 310 322 L 372 322" />
	</g>
	<g fill="none" stroke="currentColor" stroke-opacity="0.28" rx="6">
		<rect x="30" y="16" width="280" height="46" rx="6" />
		<rect x="30" y="100" width="280" height="60" rx="6" />
		<rect x="30" y="208" width="280" height="42" rx="6" />
		<rect x="30" y="298" width="280" height="52" rx="6" />
		<rect x="30" y="398" width="280" height="60" rx="6" />
		<rect x="372" y="105" width="240" height="46" rx="6" />
		<rect x="372" y="199" width="240" height="56" rx="6" />
		<rect x="372" y="294" width="240" height="56" rx="6" />
	</g>
	<g fill="currentColor" font-size="14" text-anchor="middle">
		<text x="170" y="44">one file path from the folder</text>
		<text x="170" y="124">the probe reads the first bytes</text>
		<text x="170" y="144">and the header, decoding nothing</text>
		<text x="170" y="235">is it a GIF or an SVG?</text>
		<text x="170" y="322">is the format on this</text>
		<text x="170" y="341">platform&#39;s native list?</text>
		<text x="170" y="424">the page decodes the file</text>
		<text x="170" y="444">and halves it down</text>
		<text x="492" y="133">a placeholder, and nothing retried</text>
		<text x="492" y="222">an img tile: the engine draws</text>
		<text x="492" y="242">the file itself, at any size</text>
		<text x="492" y="317">the operating system decodes</text>
		<text x="492" y="337">it already scaled</text>
	</g>
	<g fill="currentColor" fill-opacity="0.65" font-size="12">
		<text x="180" y="184">it passes</text>
		<text x="180" y="278">no, so a canvas tile</text>
		<text x="180" y="376">no</text>
		<text x="318" y="122">refused</text>
		<text x="318" y="221">yes</text>
		<text x="318" y="316">yes</text>
	</g>
</svg>
</div>

The last question names the two routes the rest of this document turns on: the ***native route***, where the operating system decodes the file and Fuji paints the pixels it returns, and the ***page route***, where the web engine decodes it and Fuji scales it down by hand.

Linux is not a fourth branch. It is the platform whose native list is empty, so every raster file on it answers that last question the same way and takes the page route.

### Fuji shows a GIF or an SVG in an img

Fuji sends a GIF to an `<img>` because only an `<img>` animates: a picture that bounces in a browser should bounce on the contact sheet too. Drawing one into a canvas would get its first frame — a perfectly good thumbnail, and what most file managers show — but we would rather show the thing itself. A contact sheet of two hundred animations sounds expensive. It is not: an animation runs only while it is actually in view, so a wall of GIFs costs only the handful on screen.

Fuji sends an SVG to the same element because an SVG has no pixels to own. The engine parses it and paints it sharp at whatever size the CSS gives, and an SVG without intrinsic dimensions gives a canvas nothing to scale a drawing to, so it cannot go the other way at all.

An `<img>` shows an SVG as an image rather than as a document: its scripts never run, it loads nothing from outside itself, it receives no events, and the element clips it, so it cannot draw over its neighbours however its coordinates are written. That is what makes it safe to show one this way, and why Fuji never shows one inline in its own document, where those scripts would run as Fuji.

### The native lists

| platform | thumbnailer | formats we send it |
| --- | --- | --- |
| macOS | [ImageIO](https://developer.apple.com/documentation/imageio) | JPEG, PNG, WebP, AVIF, BMP |
| Windows | [Windows Imaging Component](https://learn.microsoft.com/en-us/windows/win32/wic/-wic-about-windows-imaging-codec) | JPEG, PNG |
| Linux | — | none |

Each list is short and conservative, and Fuji never goes past it. Windows gets JPEG and PNG because WIC's decoders for both have shipped in every version of Windows since XP and are its most exercised code. macOS gets the formats we have run ImageIO against and watched decode. A format off a column's list goes to the page whatever that machine might have been able to do, because **we do not probe for codecs** — the routing table is a decision we made and can read, not a capability negotiation whose result changes from machine to machine.

The page route is the only one of the two that works on all three platforms, which is why it is the one that has to keep working.

The formats Fuji opens at all are a shorter list than any of the decoders underneath it:

```
.jpg  .jpeg  .jpe  .jfif  .png  .gif  .bmp  .webp  .avif  .svg
```

For context on how much headroom that leaves: ImageIO on a current Mac will decode some sixty types, including thirty-odd camera raw formats; WIC on a stock Windows install handles JPEG, PNG, GIF, BMP, TIFF, ICO, JPEG-XR and DDS, with WebP, AVIF and HEIC arriving as store extensions. Chromium's own decoders cover JPEG, PNG, GIF, WebP, BMP, ICO, AVIF and JPEG XL. The two platforms come out opposite. On macOS the native side is the more capable one and the web side adds only SVG; on Windows it is the other way round for the web-born formats, until the codec packs are installed.

## The probe

Before Fuji makes any thumbnail, one Rust call — *the probe* — reads the file's first bytes:

```js
thumbnailProbe(paths)   // → [{format, width, height, problem}, …]
```

For each file path it answers what the bytes say the file is, how big the header claims the picture is, and, if Fuji will not show it, why. It decodes nothing to answer any of that.

The format comes from a signature table, the same signatures Chromium chooses a decoder by:

```
first bytes                                 format
FF D8 FF                                    jpeg
89 "PNG" 0D 0A 1A 0A                        png
"GIF87a" or "GIF89a"                        gif
"BM"                                        bmp
"RIFF", then "WEBP" at byte 8               webp
"ftyp" at byte 4, then "avif" or "avis"     avif
"ftyp" at byte 4, then "heic", "mif1", …    heic
a "<" first, past any byte order mark       svg
```

Every format Fuji handles announces itself in its first twelve bytes, and Fuji reads thirty-two, which is also enough to size three of them.

### Why the size matters

A tile whose aspect ratio Fuji already knows can take its final size before its pixels arrive, so the thumbnail lands in a box that is already there. Without the sizes the contact sheet reflows on every arrival, each picture nudging the ones after it down the page as it appears.

Where the size comes from depends on the format, and only partly on the platform:

| format | read from | macOS | Windows | Linux |
| --- | --- | :-: | :-: | :-: |
| PNG, GIF, BMP | the first 32 bytes, parsed in Rust | yes | yes | yes |
| JPEG, WebP, AVIF, HEIC | the platform library, header only | yes | yes | no |
| SVG | nothing to read | no | no | no |

The first row is a few lines of our own Rust, so Fuji gets those three sizes on every platform. Only JPEG, WebP and AVIF reflow on Linux, and an SVG reflows on every platform.

The probe also turns files back at this point: bytes that are not a format we know, bytes that disagree with the extension, or a header claiming a raster that would not fit in half the machine's physical memory. Fuji shows the placeholder for those and never hands them to a decoder at all. The render command repeats every one of these checks itself, so a caller that skipped the probe cannot hand it a mystery. Why those particular walls, and what else belongs around a decoder, is a subject of its own and not this one.

## The operating system's route

`thumbnail.rs` calls *ImageIO* on macOS and the *Windows Imaging Component* on Windows — the same libraries Finder and Explorer use for the thumbnails in their own windows. Both take a file path rather than bytes. The library opens and reads the file itself, and the page never holds it.

On macOS the body is short, because ImageIO does the whole job in one call given three options: the longest side to scale to, a flag to render a thumbnail from the full image rather than handing back whatever small preview the file may have embedded, and a flag to apply the file's EXIF orientation. Then Fuji draws the result into a bitmap context of the color space it wants. That draw is where [Core Graphics](https://developer.apple.com/documentation/coregraphics) does the color management. One loop afterwards undoes the *premultiplied alpha* a drawing context produces: colors pre-scaled by their own transparency, which have to be divided back out before the pixels mean what `ImageData` expects.

On Windows the body is long, because WIC is a pipeline of separate objects, each initialised over the last: a decoder over the file path, its first frame, a scaled decode through the codec's own [source transform](https://learn.microsoft.com/en-us/windows/win32/api/wincodec/nn-wincodec-iwicbitmapsourcetransform) where it offers one, the [Fant scaler](https://learn.microsoft.com/en-us/windows/win32/api/wincodec/ne-wincodec-wicbitmapinterpolationmode) for the rest of the way, a flip-rotator for the EXIF orientation that *WIC* leaves to the caller, a color transform from the file's profile to sRGB, and a format converter to straight-alpha RGBA. Nothing runs until the final `CopyPixels`, which pulls the whole pipeline.

Either way one buffer comes back. Its bytes, by offset:

```
0-3    width,  little-endian u32
4-7    height, little-endian u32
8-11   1 if the pixels are Display P3, 0 if they are sRGB
12-    straight-alpha RGBA, top row first, 4 bytes a pixel
```

One buffer rather than a structured result, because Tauri serializes a `Vec<u8>` crossing its bridge into a JSON array — one decimal number per byte, written on one side and parsed on the other. We measured that at about a hundred and fifty milliseconds per megabyte, flat and linear in file size, which no disk is. As an `ArrayBuffer` the same bytes cross in single-digit milliseconds.

The clock is not the only difference. Both libraries perform a *scaled decode*: a **baseline** JPEG comes out at an eighth of its size by skipping most of the inverse transform, and baseline is the ordinary case, so **the full-size raster never exists**. And the work runs on Tauri's thread pool, so it never touches the thread that runs the window.

## The page's route

For everything else — WebP, AVIF and BMP on Windows, everything on Linux — the page does it. The store reads the file through Rust, wraps the bytes in a blob, makes an object URL, points an element at it and decodes it. The flow then draws that element down into the canvas and releases it, so the store can drop the bytes, the URL and the full-size decode the moment the canvas has the pixels.

On macOS nothing takes this route at all: every raster extension Fuji opens is on that platform's native list. Emptying that list is the only way to reach it on a Mac, and that is how we test it.

One image at a time, because each one is a full decode held in memory and a draw on the main thread.

### Why the page halves instead of drawing once

A single [`drawImage`](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/drawImage) from a full-size decode into a small canvas comes out rough on the Mac: fine repeating texture in a 6240 by 4160 raster turns to jaggies at 240 pixels, where Safari shows the same file smooth as an `<img>`. Two engine behaviours cause it, and they compound.

*Core Graphics*' high interpolation reads a fixed footprint of source pixels around each output pixel. At twenty-six to one, that leaves most of the picture unread, and pixels that are never read alias. Chromium's `high` is a chain of successive halvings sampled with a cubic filter, which does read everything, and is why the same one-pass draw looks fine on Windows.

And WebKit hands `drawImage` a subsampled frame only when it has to decode one. A frame already decoded at full size counts as good enough for any smaller request, and the store's [`decode()`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLImageElement/decode) produces exactly that frame, so the engine hands `drawImage` all twenty-six million pixels, where it hands Safari's `<img>`, which never calls `decode()`, a sixteenth of them.

So the page route halves through scratch canvases until the last draw is within two to one, the ratio at which a fixed footprint is an honest average of the source. Every step sets [`imageSmoothingQuality`](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/imageSmoothingQuality) to `high`; the default, `low`, reads only the four nearest source pixels and would alias even at that ratio.

[`createImageBitmap`](https://developer.mozilla.org/en-US/docs/Web/API/Window/createImageBitmap) looks like the purpose-built tool here — ask it for `resizeWidth` and `resizeQuality` and it decodes straight to the size wanted — and on Chromium it is. WebKit has the code and does not ship the options, so on macOS it returns a full-size bitmap: a whole second copy of a large raster, allocated for nothing, then scaled by the same `drawImage` that could have done the job alone. So Fuji does not use it.

## Speed

Measured outside the app on an Apple M2, macOS 15.7, one image at a time, third run of three, thumbnail longest side 480. The native column is `thumbnail.rs` timed from a scratch binary. The web columns are the page route's exact code run in a headless WKWebView: `decode` is the element's decode, `main` is what the halving cost the main thread, and `complete` is when the pixels were finished in whichever process drew them. All in milliseconds.

Six large JPEGs, all progressive, 0.4 to 6.2 MB on disk:

| raster | native | web decode | web main | web complete |
| --- | --: | --: | --: | --: |
| 6240 × 4160 | 48 | 971 | 224 | 260 |
| 4096 × 4096 | 17 | 80 | 52 | 76 |
| 4105 × 3079 | 22 | 217 | 86 | 104 |
| 3629 × 2419 | 78 | 143 | 80 | 86 |
| 2455 × 3068 | 73 | 123 | 74 | 80 |
| 3299 × 2199 | 57 | 106 | 59 | 64 |

Two cautions about that table. The native column does not track raster size, because these six differ in bytes as much as in pixels; nothing here explains the ordering and we have not chased it. And all six are progressive JPEGs. A scaled decode saves least on those, because every scan has to be entropy-decoded before any scaling helps, so this table understates the native route's advantage on ordinary baseline JPEGs.

The same six re-saved as baseline JPEGs at web sizes, which is most of what a folder actually holds:

| source | native | web decode | web main | web complete |
| --- | --: | --: | --: | --: |
| 1600 px, 120–680 KB | 6–9 | 10–15 | 1–2 | 11–18 |
| 1000 px, 57–270 KB | 3–4 | 5–7 | 0–1 | 4–6 |

For web-sized pictures both routes are fast and the native one is about three times quicker on the clock. A folder of two hundred fills in a few seconds either way and nobody drops a frame.

For rasters this large the picture changes. Native is three to twenty-five times faster. The number that decided the design is the next one: the halving costs the main thread fifty to two hundred and twenty milliseconds per image. That is a dropped frame or a dozen, per picture, while a folder of large images fills. The cost comes from the intermediate canvases forcing the engine to finish each step before the next one reads it; a lone draw into a canvas costs the main thread nothing, because the engine defers it.

**Responsiveness ranks above total time.** A person who waits a second for a folder while Fuji stays completely responsive is far better off than one who waits the same second while Fuji stutters. The native route costs the main thread nothing at all, because the decode runs on a pool thread and the page receives only small pixels. That, more than the multiplier, is why the operating system goes first wherever we let it.

### Where this document stops

The costs above are per picture, and they are what this document is about. How many thumbnails should be in flight at once, in what order, what a contact sheet ought to be holding while the user scrolls, and whether finished thumbnails belong on disk are different questions with different answers, and Fuji's are still moving. The one thing that follows directly from the numbers above: the native route can run several at a time, because each is a pool thread that never touches the page, and the page route cannot, because each is a full decode and a draw on the one thread that also has to keep the window responsive.

## Fidelity

Three things can go wrong between the file and the tile: the wrong size, the wrong sharpness, the wrong colors. Everything in this section was measured on hardware rather than reasoned about, because two of the three are invisible on an ordinary sRGB monitor where a CSS pixel and a device pixel are the same thing.

### Fit

The longer side goes to the box, and Fuji never enlarges a picture:

```js
let scale = Math.min(box / size.x, box / size.y, 1)
```

A square picture lands exactly on the box, a wide one hits the limit on width alone, and a picture already smaller than the box keeps its own size. **Fuji crops nothing** — every tile shows a whole picture, and the rows come out ragged because of it. The box is one of four named sizes: Small, Medium, Large and XL, at 120, 240, 360 and 480 CSS pixels. All four are settings, so Medium means whatever the user says it means.

### One device pixel

The engine lays a canvas out on whole CSS pixels and paints it at the device pixel ratio, so the box it occupies on screen is a whole number of CSS pixels times that ratio. If the bitmap is not exactly that many device pixels, the compositor cannot *blit* it one to one, copying each pixel straight onto the screen. It resamples instead.

The long axis always survives and the short one often does not. For a landscape thumbnail that comes back 480 × 269, in a box of 240 at a ratio of 2:

```
scale = 240/480 = 0.5
css.x = round(480 × 0.5) = 240 → 240 × 2 = 480 = bitmap  matches
css.y = round(269 × 0.5) = 135 → 135 × 2 = 270 ≠ 269     one short
```

The long axis is the box times the ratio by construction, so it always matches. The short axis is whatever the thumbnailer computed for the aspect ratio, and that is odd about half the time — so on a Retina panel **half of all native-route thumbnails** would sit in a box one device pixel taller or wider than they have pixels for, and the compositor would resample every row.

That resample is not a soft edge. The phase between source and destination rows drifts from zero at one end of the picture to a full pixel at the other, passing through exactly half a pixel in the middle, where every output row is the mean of two source rows.

Measured on a Retina, Display P3 Mac at a device pixel ratio of 2, macOS 26.2, with two test images authored at exactly the size the thumbnailer returns, so neither is resampled on the way in, each carrying one-pixel black and white rows. The figure is the share of pixels that came back neither near-black nor near-white:

| tile | canvas sized to the picture | canvas sized to the box |
| --- | --: | --: |
| 480 × 268, even short side | 0.11% | 0.00% |
| 480 × 269, odd short side | 89.91% | 0.00% |

Sized to the picture, the odd tile decays from clean alternating rows at the top to a flat `rgb(128,128,128)` at its midpoint, the pattern erased entirely, and recovers toward the bottom. On a continuous-tone image that is softness; on any fine repeating detail it is a total collapse of contrast.

The even tile is the control, and its two figures are not a result: its geometry is the same under both rules, and the 0.11% against the 0.00% comes from two different kinds of screen capture rather than from anything on the page.

**Fractional CSS does not fix this**, though it looks as though it should. Giving the element a height of `backing / devicePixelRatio` un-rounded is legal, and the engine honours it exactly: `style 134.5px`, `computed 134.5px`, a layout rectangle of 134.5. The odd tile still reads 90.74% intermediate. Padding the bitmap instead, against a whole-pixel CSS height, drops it to zero. A fractional CSS box does not buy a device-pixel-exact blit in WebKit.

**So Fuji sizes each canvas to the box the compositor will use**, rather than to the picture, and puts the picture in the corner of it:

```js
function flowSnap(side, have) {
	let ratio = window.devicePixelRatio
	let want = Math.round(side * ratio)
	return want > have + ratio ? have : want
}
```

That tolerance, `have + ratio`, separates two cases. A thumbnail shrunk to fit misses its box by at most one device pixel of rounding, and taking that sliver buys a one-to-one blit for every row — so Fuji takes it, and repeats the picture's last row and column into it, making the seam the picture's own color rather than a transparent line. A picture *smaller* than the box misses it by far more than that and is meant to, since the fit leaves such a picture at its own size and the engine enlarges it the way an `<img>` would; that one keeps the pixels it has.

Three things follow. There is **no reflow**, because the box Fuji reserves from the header size and the box it computes from the thumbnail land on the same whole CSS pixel. The rule is a **no-op at ratio 1**, so an ordinary monitor behaves exactly as before. And it is a **no-op on the page route**, where the canvas is derived from the CSS size and the two cannot disagree — the native route is the only place where a bitmap arrives from outside and the CSS size has to be recovered from it by division.

One platform difference is not settled. macOS scale factors are only ever 1 or 2, so a whole-CSS box times the ratio is always a whole number of device pixels and Fuji can always size a canvas to it exactly. Windows offers 125%, 150% and 175%, so the ratio is fractional and a whole-CSS box lands on a fraction of a device pixel: at 1.5, a 135-pixel box is 202.5 device pixels, and a canvas can be 202 or 203 but not 202.5. Windows has one pixel unit fewer than macOS, and pays for the simpler model with worse arithmetic. Nobody has yet measured whether Chromium snaps such a box to a whole device pixel, or which way it goes.

### Color

A canvas is sRGB unless the page asks for something else, and `drawImage` converts its source into the canvas's color space on the way in. Any color the source holds that sRGB cannot express, `drawImage` moves to the nearest color sRGB can, and a file tagged Display P3 holds plenty of them. Nothing afterwards records what the original value was; the canvas holds the moved one. On a wide-gamut screen the thumbnail then looks duller than a light table shows the same file.

Fuji's answer has three parts, each in a different layer.

**The page asks the screen, not the engine.** The [`color-gamut`](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/color-gamut) media query reports what the display can show, and Fuji creates every canvas in that space. Asking the engine by name instead would be a feature check: Chromium accepts `display-p3` everywhere, and WebKit compiles that value only into its Apple builds, so on WebKitGTK the value does not exist and passing it throws. Asking the screen sidesteps the question — a wide-gamut panel gets a wide canvas, an ordinary screen gets the sRGB canvas it always had, and the engine that has no such value is never asked for one.

**Rust converts and then says what it converted to.** `thumbnail.rs` draws each thumbnail into a bitmap context of the space the page asked for, and reports in the twelve-byte header which space it actually delivered. macOS delivers what it was asked for; WIC on Windows answers sRGB whatever is asked, because it has no Display P3 context without a profile file, and the header says so.

**The page tags the pixels with what the header said**, so a wide-gamut canvas converts Windows' always-sRGB pixels rather than reading them as wide values.

Measured with three PNGs authored so that *Core Graphics* performed every conversion and nobody typed a value in by hand. Each halved image has a primary that lies outside sRGB on the left, and the *sRGB* primary expressed in P3 coordinates on the right — the same physical color, one in gamut and one out. On a correct pipeline the halves differ visibly; if anything clamps, they become identical.

| image | what Rust returned | canvas read as `display-p3` | canvas read as `srgb` |
| --- | --- | --- | --- |
| P3 red halves | `(255,0,0)` / `(234,51,35)` | `(255,0,0)` / `(234,51,35)` | `(255,0,0)` / `(255,0,0)` |
| P3 green halves | `(0,255,0)` / `(117,251,76)` | `(0,255,0)` / `(117,251,76)` | `(0,255,0)` / `(3,255,0)` |
| sRGB red control | `(234,51,35)` both halves | `(234,51,35)` | `(255,0,0)` |

Every value matches the file exactly. The sRGB readings are the confirmation rather than a failure: pure P3 red is outside sRGB, so it clips when asked for in sRGB, while the in-gamut half maps back to the `(255,0,0)` it was born as. The green pair round-trips three units off in one channel, which is the expected cost of two conversions at eight bits.

The control runs the other way, and it matters as much: Fuji must *convert* an sRGB file into P3 coordinates rather than reinterpret it as P3, or every ordinary image on the contact sheet would come out oversaturated. It converts correctly both ways.

Captured from the screen rather than read back from the canvas, the contact sheet shows two distinct clusters per hue in equal counts, and the sRGB control lands on the same value as the P3 file's in-gamut half — two different encodings of one physical color arriving at the same value. If anything in the chain reinterpreted where it should convert, or converted where it should reinterpret, that is the pair that would come apart. Running the same three images through the page route instead produced values identical to the native route, to the last unit, which means WebKit's `<img>` decode preserves the wide gamut and `drawImage` into a P3 canvas preserves it too.

## What is still open

We left every one of these alone on purpose.

- **Animated WebP** is a still on the native route. Sending it to an `<img>` the way Fuji sends a GIF means reading the animation flag out of its header in the probe.
- **Linux's size gate** covers PNG, GIF and BMP, which our own Rust can size. WebP and AVIF are unsized there, and they are two of the formats that decode whole — a JPEG never needs the gate, because both operating systems decode one scaled and the full raster never exists.
- **`flowSnap` at a fractional device pixel ratio**, per the Windows note above. Until somebody measures it, the most we claim for Windows is "no worse than before".
- We read **WIC's color behaviour** out of its documentation and have never run it against a file with a profile in it.
- **A change of monitor.** Fuji never remakes a canvas when a window moves to a different screen, so the device pixel ratio and the gamut both go stale until the contact sheet rebuilds.
