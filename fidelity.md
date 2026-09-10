# Fidelity

Whether a picture arrives on the screen with its pixels and its colors intact. Fuji was written on a Mac mini driving a twenty-year-old Dell, where a CSS pixel, the backing store and the hardware were all one grid and the screen was sRGB. Two whole families of code path — everything sensitive to `devicePixelRatio`, and everything behind the Display P3 check — therefore ran for the first time on 2026-09-10, on a Retina, wide-gamut MacBook Air. This file is what that day established.

Everything below was measured on hardware. Where a claim comes from reading the code rather than from a measurement, it says so, and where something remains untested it is listed at the end rather than quietly assumed. The audit found one defect, and fixing it is the change that came out of the day.

`canvas.md` says what a canvas can hold and why a thumbnail is made where it is; `thumbnail-plan.md` is the plan `SquareFlow.vue` implements; `performance.md` says what any of it costs. This file says whether it is *right*.

## The machine, in three units

Every number below came off one machine, and a cross-platform project should always say which:

    13.6" MacBook Air, Apple M3, 16 GB, macOS 26.2 (build 25C56)
    Apple clang 17.0.0, arm64-apple-darwin25.2.0
    display set to "looks like" 1710 × 1112, the mode Kevin works in

Three pixel units are at work, and confusing two of them is the whole risk.

**Points** are what CSS, the user, and the macOS display settings speak — the "looks like" resolution. **Backing pixels** are the bitmap macOS renders into, always exactly 2× points on a Retina panel, because macOS scale factors are only ever 1 or 2 and never fractional. **Native pixels** are the panel's own lights.

On this machine, in the mode this audit ran in:

    points          1710 × 1112     what CSS and the user see
    backing         3420 × 2224     what macOS renders, and what devicePixelRatio reaches
    native          2560 × 1664     the panel, including the 64-row notch strip

The ratios are ugly on purpose: 1710 was chosen for comfort, not arithmetic. The smallest square that survives every stage whole is 171 points → 342 backing → 256 native. If a mode with countable arithmetic is ever needed, "looks like" 1024 × 640 is exact in both axes — 2 points → 4 × 4 backing → 5 × 5 native — as a below-notch letterboxed mode.

The backing → native squish is invisible to every web API, and it happens to everything on screen equally. Measured live inside fuji:

    devicePixelRatio                     2
    screen.width × screen.height         1710 × 1112     points
    tauri monitor.size                   3420 × 2224     backing
    tauri monitor.scaleFactor            2
    matchMedia('(color-gamut: p3)')      true

**Tauri reports the backing store, not the panel.** That single fact is what makes `screenToViewport` correct, and it is the kind of thing only hardware can settle.

The native 2560 × 1664 figure comes from the earlier code-reading audit, which replicated `panel.rs`'s heuristic in C on this machine; it was not re-measured in this session. That heuristic — enumerate every display mode and keep the tallest by pixel height — is right *only* because it passes null options to `CGDisplayCopyAllDisplayModes`. With `kCGDisplayShowDuplicateLowResolutionModes` the list gains the scaled modes' backing stores and the winner becomes 3420 × 2224, which is exactly the wrong answer. The null is load-bearing.

`panel.rs` is rightly unused by the thumbnail pipeline. A canvas can only address the backing store, so native-resolution pixels would be resampled twice on the way down. It exists for a different promise — that "100%" on a table can one day mean one image pixel on one light — and its only caller today is diagnostics.

## The instrument

Four ways of looking, each answering a different question.

**In-page canvas readback.** `getImageData` with an explicit `colorSpace` says what a canvas actually holds, in whichever space you ask for. Reading the same canvas as both `display-p3` and `srgb` is what separates "the wide value survived" from "the wide value was clamped and happens to look similar".

**Whole-display capture.** `screencapture -x` writes the composited framebuffer at 3420 × 2224, tagged `kCGColorSpaceDisplayP3`. This is the end of the line: what the compositor put on the glass.

**Single-window capture.** `screencapture -x -o -l <CGWindowID>` grabs one window's own buffer at 1972 × 1716, also tagged Display P3, regardless of what is stacked in front of it. The window id comes from `CGWindowListCopyWindowInfo` filtered by owner name. This is what made the audit scriptable, because fuji could not be brought to the front — Tauri's capability list has no `core:window:allow-set-focus`.

**Scratch tools in C.** The `swift` CLI is broken on this machine (SDK and toolchain version mismatch), but `clang -framework ApplicationServices -framework CoreFoundation` compiles and runs fine, and every generator and inspector below was built that way. Two tools: one that authors test images through `CGBitmapContext` and `CGImageDestination`, and one that reports an image's embedded color space and its stored bytes *without converting them*, plus region statistics.

One methodological point carries the color half of the audit. **A screenshot is tagged with the display's profile, so comparing two regions of the same screenshot sidesteps every question about what that profile is.** The test images are built so that the answer is a difference between two halves of one picture, not an absolute value.

Note also that the whole-display capture and the window capture do not agree to the last unit: the same tile reads `rgb(255,0,0)` in the window buffer and `rgb(254,0,0)` on the display. The display capture goes through one more conversion, which is the likely cause, but it was not isolated. Comparisons in this file are always within one capture.

## Color

### What the pipeline promises

A web canvas is sRGB unless asked otherwise, and drawing a P3 photograph into an sRGB canvas clamps its most vivid colors for good. Fuji's answer is in three parts. `SquareFlow.vue` asks the *screen* what it can show — `matchMedia('(color-gamut: p3)')`, not the engine — and creates every canvas in that space. `thumbnail.rs` converts each thumbnail's pixels into the space it was asked for, by drawing the decoded `CGImage` into a `CGBitmapContext` of that color space, and reports in its twelve-byte header which space it actually delivered. The page then tags the `ImageData` with what the header said, so Windows' always-sRGB pixels stay correct on a wide-gamut canvas.

The capability everything rests on was confirmed first: `getContext('2d', {colorSpace: 'display-p3'}).getContextAttributes().colorSpace` returns `display-p3` in this WKWebView.

### The test images

Three PNGs, authored by filling rectangles with colors created in a *named* space so that CoreGraphics performed every conversion and no value was typed in by hand. Read back from the files, unconverted:

    p3-red-halves.png      2000×1000   Display P3   left (255,0,0)   right (234,51,35)
    p3-green-halves.png    2000×1000   Display P3   left (0,255,0)   right (117,251,76)
    srgb-red-control.png   2000×1000   sRGB         left (255,0,0)   right (255,0,0)

In the two halved images the left half is a primary that lies outside sRGB, and the right half is the *sRGB* primary expressed in P3 coordinates — the same physical color the old Dell could show. On a correct pipeline the two halves are visibly different. If anything in the chain clamps, they become identical. That binary difference is the whole test.

The control is the other direction, and it is the case the original test plan did not include: an sRGB file must be *converted* into P3 coordinates, not reinterpreted as P3, or every ordinary picture would come out oversaturated.

### Through Rust, and onto a canvas

Sizes first: at the Medium thumbnail box of 240 points and `devicePixelRatio` 2, fuji asks for a longest side of 480 backing pixels, and a 2000 × 1000 source comes back 480 × 240. The header reported `display-p3`.

The bytes Rust returned, and the same canvas read back afterwards:

    p3-red-halves      rust    left (255,0,0)     right (234,51,35)
                       canvas  left (255,0,0)     right (234,51,35)     read as display-p3
                       canvas  left (255,0,0)     right (255,0,0)       read as srgb
    p3-green-halves    rust    left (0,255,0)     right (117,251,76)
                       canvas  left (0,255,0)     right (117,251,76)    read as display-p3
                       canvas  left (0,255,0)     right (3,255,0)       read as srgb
    srgb-red-control   rust    (234,51,35)        both halves
                       canvas  (234,51,35)        read as display-p3
                       canvas  (255,0,0)          read as srgb

Every value matches the file exactly. The sRGB readings are the confirmation rather than a failure: pure P3 red is outside sRGB, so it clips to `(255,0,0)` when asked for in sRGB, while the in-gamut half maps back to the `(255,0,0)` it was born as. The green pair round-trips to `(3,255,0)` rather than `(0,255,0)`, a three-unit error in one channel from two 8-bit conversions — the expected cost of the round trip, not a fault in the chain.

The control converts correctly in both directions. **An sRGB file is not mistaken for a P3 one.**

### Through the page

The page route — the store decodes the file into an `<img>` and the page halves it down into the canvas — is unreachable on the Mac in normal use (see *Smaller things*, below), so it was exercised by temporarily emptying the Mac entry in `flowNative`. It produced **values identical to the native route on all three images**, to the last unit.

That result carries further than the page route itself, because it means two things at once: WebKit's `<img>` decode preserves the wide gamut, and `drawImage` into a P3 canvas preserves it too. The diamond table shows the store's own decoded `<img>`, so this is also the evidence for the table.

### On the glass

The contact sheet, from a whole-display capture, clustered over fuji's viewport:

     83.63%   rgb(  0,  0,  0)   the sheet's background
      5.30%   rgb(234, 50, 34)   the sRGB-red half, plus the whole sRGB control
      1.77%   rgb(254,  0,  0)   57,600 px — exactly 480 × 120, the pure-P3 half
      1.76%   rgb(  0,255,  0)
      1.76%   rgb(117,251, 76)

The diamond table, from a window capture, showing the same image full size through an `<img>`:

     25.97%   rgb(234, 51, 35)   n = 878,906
     25.97%   rgb(255,  0,  0)   n = 878,906

Two distinct clusters per hue, in equal counts, on both views. **Display P3 survives from the file to the display, through the operating system's thumbnailer, through the page's own canvas route, and through an `<img>` in a table.** And the sRGB control lands on the *same* value as the P3 file's in-gamut half, which is the strongest form this test can take: two different encodings of one physical color converge.

## Space

### The ask is right

Established by the earlier code-reading audit and unchanged by anything measured since. `SquareFlow.vue` asks Rust for `flowBox × window.devicePixelRatio` as the longest side in backing pixels, sizes each canvas bitmap to what comes back, and derives the CSS size back down by the same ratio. The page route computes `detail = min(devicePixelRatio, 1 / scale)` and so never allocates more canvas pixels than the file actually has. The worry that fuji asked for point-sized thumbnails on a Retina screen was unfounded.

### One device pixel

The defect the audit found, and the reason for the code change that came with it.

A canvas is laid out on whole CSS pixels and painted at the device ratio, so its box on screen is a whole number of CSS pixels times that ratio. `flowFit` rounds the CSS size to whole pixels; `flowSize` then sized the canvas *to the thumbnail* rather than to that box. For a landscape thumbnail returned at the full requested maximum:

    scale  = 240/480 = 0.5
    css.x  = round(480 × 0.5) = 240        → 240 × 2 = 480 = bitmap    ✓
    css.y  = round(269 × 0.5) = 135        → 135 × 2 = 270 ≠ 269       ✗

The long axis always survives, because it *is* the box times the ratio by construction — an even number by definition. The short axis is whatever the thumbnailer computed for the aspect, `round(short × 480 / long)`, and that is odd about half the time. So roughly half of all native-route thumbnails were handed a box one device pixel taller or wider than they had pixels for, and the compositor filled the gap by resampling.

That resample is not a soft edge. The phase between source and destination rows drifts from zero at one end to a full pixel at the other, passing through exactly half a pixel in the middle, where every output row is the mean of two source rows.

Two test images were authored at exactly the size the thumbnailer returns — 480 on the long side, so neither is resampled on the way in — carrying one-pixel black and white rows:

    fit-480x268-even.png    css 240 × 134   →  480 × 268 backing   exact
    fit-480x269-odd.png     css 240 × 135   →  480 × 270 backing   one row too many

Measured over the same-sized region of each tile, counting pixels that are neither near-black nor near-white:

    even tile      0.11%     whole-display capture
    odd  tile     89.91%     whole-display capture

The odd tile decayed from `(0, 254)` at the top to a flat `rgb(128,128,128)` at its midpoint — the pattern erased entirely — and recovered toward the bottom. On a photograph this is the softness the first audit predicted; on any fine repeating detail it is a total collapse of contrast.

Only the native route can produce it. The page route derives its bitmap *from* the CSS size — `backing = round(css × detail)` — so the two cannot disagree. The native route is the only place where the bitmap arrives from outside fuji and the CSS size has to be recovered from it by division, and dividing by the device ratio is where the half lands. At `devicePixelRatio` 1 that division is the identity, which is why no 1:1 machine could ever have shown it.

### The fix that did not work

The first audit proposed setting the canvas's CSS size to `backing / devicePixelRatio` un-rounded, on the grounds that fractional CSS is legal. It is legal, and the engine accepted it exactly:

    thumb 480×269   style 240px × 134.5px   computed 240px × 134.5px   rect 240×134.5 at y 0

and the tile was **still 90.74% intermediate**, unchanged. **A fractional CSS box does not buy a device-pixel-exact blit in WKWebView.** Confirmed from the other side by padding the bitmap to 270 rows against a whole-pixel CSS height of 135, which dropped straight to 0.00%. Recorded here because it is the kind of plausible fix that would otherwise be tried again.

### The fix

Size the canvas to the box the compositor will actually use, rather than to the picture, and put the picture in the corner of it. In `SquareFlow.vue`:

```js
function flowSize(tile, canvas, backing) {
	tile.css = flowFit(backing).css
	canvas.width = flowSnap(tile.css.x, backing.x); canvas.height = flowSnap(tile.css.y, backing.y)
	…
}
function flowSnap(side, have) {
	let ratio = window.devicePixelRatio
	let want = Math.round(side * ratio)
	return want > have + ratio ? have : want
}
```

The constant separates two cases. A thumbnail shrunk to fit misses its box by at most one device pixel of rounding, and taking that sliver buys a one-to-one blit for every row. A picture *smaller* than the box misses it by far more and is meant to — the fit leaves such a picture at its own size and the engine enlarges it the way an `<img>` tag would — so that one keeps the pixels it has.

`flowEdge` then repeats the picture's last row and column into whatever sliver was taken, so the seam is the picture's own color rather than a transparent line. `flowFit` is unchanged.

    even tile      0.00%     window capture
    odd  tile      0.00%     window capture

The odd tile's midpoint, which was flat grey, reads `0, 255, 0, 255, …`; every column sampled alternates cleanly down to the last real row, and the pad row after it reports `alpha 255` — filled, not transparent. The even tile's 0.11% before and 0.00% after is not a change in its rendering: its geometry is identical either way, since `flowSnap` returns the size it already had, so the difference belongs to the two kinds of capture.

Three properties are worth recording. There is **no reflow**: `flowApply` reserves each box from the header size and `flowSize` computes the final one from the thumbnail, and for an odd short side both land on the same whole CSS pixel, so `flowApply` needed no change. It is a **no-op at ratio 1**, so the machine fuji was written on behaves exactly as before. And it is a **no-op on the page route**, where the canvas already equals the ask; `flowShrink` was pointed at the canvas's real size rather than at that ask so the route cannot leave an edge under any rounding.

## The window

### screenToViewport

`screenToViewport()` converts everything to CSS pixels before subtracting, using `scale = cssScreen.y / backingScreen.y`. On the Dell that scale was 1 and any unit confusion in it was invisible. Here it is `1112 / 2224 = 0.5` exactly, because Tauri reports the backing store. Measured alongside it: Tauri's `outerSize` and `innerSize` are both `1972 × 1716` — they are the same number on the Mac, which is why the function takes its inner size from `window.innerWidth/Height` instead — giving a border of 0 and a title bar of 32 CSS points.

Its one consumer is the diamond table's full-screen transition, which uses the before-and-after difference to hold the image still on the glass. Measured as the image's bounding box in *screen* coordinates, computed as the window's origin plus its box within the window's own buffer:

    windowed      screen x 1083..2959   y 500..1438   1877 × 939   n = 1,762,502
    fullscreen    screen x 1083..2959   y 500..1438   1877 × 939   n = 1,762,502
    windowed      screen x 1083..2959   y 500..1438   1877 × 939   n = 1,762,502

Not one pixel of movement, in either direction, with an identical pixel count each time.

### Remember and restore

Fuji records the window in physical pixels at both ends, which is self-consistent on one machine. Driven through Tauri's own `setPosition` and `setSize` so the real listeners fired: the window was moved to `300,200` and sized to `1400×1000` physical, and on a clean exit `fuji.toml` held exactly `x = 300, y = 200, width = 1400, height = 1000`. Relaunching put it back at 150,100 points and 700 × 500 — the same rectangle. Restore was separately confirmed twice at the original `1036,78 / 1972×1716`.

Worth knowing rather than testing: a rectangle recorded on a `devicePixelRatio` 1 machine restores at half size here. That is cross-machine settings migration, not a defect in this code.

## Smaller things the audit settled

**The page canvas route is dead code on the Mac.** `imageTypes` in `library.js` contains no canvas-kind extension that is off the Mac's native allow list — GIF and SVG go to `<img>`, and everything else (`jpeg`, `png`, `webp`, `avif`, `bmp`) is on the list. There is no `.heic` entry at all. The route is live on Linux and reachable on Windows, and it was exercised here only by emptying the list.

**A canvas is never remade when its window changes monitors**, so a `devicePixelRatio` or gamut change goes stale until the sheet rebuilds. Read from the code; not exercised.

**An image whose pixels fall between the box and box × ratio is displayed enlarged in backing terms.** `flowFit` treats bitmap pixels as CSS pixels, which is exactly an `<img>` tag's semantics and is consistent, but "never enlarged" is true only of the bitmap, not of what reaches the glass. This is a design choice, not a defect, and `flowSnap` preserves it deliberately.

Three things that only matter to someone driving fuji from outside, all met while building this audit. Tauri's capability list has no `core:window:allow-set-focus` or `core:window:allow-close`, so a script can move and resize the window but cannot raise or quit it. `DiamondTable` loads an image only in response to a drop, so opening the model is not enough to make it show anything. And `tauri dev`'s rebuild kills the app without reaching `RunEvent::Exit`, so a rebuild writes neither the settings nor the log — as does any run short enough that the page's 1500 ms quiet timer never flushed a line.

## What is still not established

- **The `wide: false` path.** When the screen is not P3, `flowGamut` is `srgb` and `thumbnail.rs` converts into sRGB instead. Untestable here for want of an sRGB display.
- **Windows.** `flowSnap` is written for fractional ratios, but a `devicePixelRatio` of 1.25 or 1.5 is untested, and there a whole-CSS box lands on a half device pixel no matter what fuji does. The most that can be claimed for Windows today is "no worse than before".
- **WIC's color behaviour**, which reports sRGB whatever is asked and says so in the header. Read from the code, never run.
- **The native panel resolution**, 2560 × 1664, carried from the earlier code-reading audit and not re-measured in this session.
- **Monitor changes**, per the stale-canvas note above.

## Repeating this

The test images are the whole method, and each is a few dozen lines of C. Author them in a `CGBitmapContext` of a named color space, fill rectangles with `CGColor`s created in *different* named spaces so CoreGraphics does every conversion, and write them out with `CGImageDestination`, which embeds the profile. For color, make a two-half image whose halves are the same hue in and out of sRGB, and ask only whether the halves differ. For geometry, author at exactly `box × devicePixelRatio` on the long side so nothing is resampled on the way in, put one-pixel rows and columns in it, and count the pixels that come back neither black nor white.

Read the results with `screencapture -x -o -l <CGWindowID>` and a C tool that reports stored bytes without converting them. Never compare across two captures.

Keep the images and the tools outside the working tree; the sheet will open a folder anywhere.
