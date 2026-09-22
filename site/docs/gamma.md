# Gamma

A pixel's stored value and the light a screen makes from it are related by a power curve, and the exponent of that curve is called gamma.

::: warning Draft
Everything above *The equation* and below its examples is notecards: details, snippets and research to do, in the order the page will tell them. None of it is written yet, and anything marked *verify* was said from memory in conversation and needs a source before it is stated.
:::

## Film

- Hurter and Driffield, 1890: plotted a negative's density against the log of the exposure that made it. The straight middle of that curve is the *characteristic curve*, and they named its slope gamma. High gamma is a contrasty film, low gamma a flat one. *Verify* the paper and its title, and find a period plot of the curve.
- Why the word survived: a power law is a straight line on log-log axes, and its slope is the exponent, so "gamma" and "exponent" became the same thing.
- The 18% gray card: the gray that *looks* halfway between black and white is about 18% of white's light, CIE lightness 50. Ties film's gray card to the perceptual argument below.

## Television

- A cathode-ray tube's light is roughly its drive voltage to the 2.5 power, a property of the electron gun. *Research* the physics — the space-charge law is 3/2, and why a real tube lands near 2.5.
- Correct once, at the camera: broadcasters encoded light to the 1/2.2 power so millions of sets needed no correcting circuit. The camera's curve and the tube's cancel. *Verify* NTSC 1953 and the value it assumed.
- The happy coincidence, Poynton's point: the tube's curve is nearly the inverse of human vision. We sense brightness as a ratio (Weber–Fechner), and perceived lightness goes roughly as the cube root of light (CIE L*). So a gamma-encoded signal is close to perceptually uniform.
- Why it outlived the tube: with 8 bits, linear codes would band the shadows — codes 1, 2 and 3 are jumps of 100% and 50% — and waste dozens of codes in the highlights nobody can tell apart. Gamma is a compression scheme that spends bits where the eye can use them, and displays now emulate the old curve on purpose.
- System gamma slightly above 1, about 1.1 to 1.2, for a dim room, because a dark surround lowers perceived contrast. *Research* Bartleson and Breneman.
- The modern names: OETF for the encoding side, EOTF for the display side. Rec. 709 for encoding, BT.1886 (2011) at a pure 2.4 for display, and PQ and HLG for HDR, which leave power laws behind.

## Personal computers

### The chain inside the machine

- framebuffer → lookup table → DAC → tube → light. The table is 256 entries per channel in the graphics card, applied at scanout to the whole screen, every application at once, for free, and invisible to every application.
- The table's exponent is *S* ÷ *T*, the one piece of this the core section below covers.
- A curved 8-bit table costs precision: it spreads the shadows, leaving gaps that show as faint bands, and merges neighbors in the highlights. *Research* which graphics cards had DACs wider than 8 bits, and which Macs.

### The Mac chose a system gamma

- 1.8, from the first Macs, to match Apple's displays and the LaserWriter, so the screen matched the print. *Verify* the origin story and its date.
- ColorSync wrote the curve into the table at login from the display's profile and a target chosen in the Monitors control panel and later the Display Calibrator, 1.8 by default.
- The squint test: an Apple logo of solid gray on a field of fine black and white stripes, and a slider moved until the logo vanished. Stripes are exactly 50% light on any monitor, so the match gives one equation in one unknown: code 186 means a gamma of about 2.2, code 194 about 2.5.
- The number held from the first Macs through the Bondi Blue iMac to Mac OS X 10.6 Snow Leopard, 2009, which moved the default to 2.2.

### Windows inherited a monitor gamma

- No setting in the operating system and no curve in the table: *S* was whatever the tube did, about 2.5, and it drifted with the monitor's brightness and contrast knobs, because the brightness knob sets the black level.
- The table was still there, reachable through `SetDeviceGammaRamp`, and three things wrote it. Games, whose brightness sliders wrote the whole screen's table, which is why the desktop sometimes stayed bright after a crash. Adobe Gamma, which shipped with Photoshop and put a loader in the Startup folder to write a calibrated curve at every boot. And the NVIDIA and ATI control panels, whose sliders started at 1.0.
- Adobe Gamma offered "Windows default 2.2" and "Macintosh default 1.8" as targets. *Verify.*
- Windows 7, 2009, the same year as Snow Leopard, brought a built-in display calibration tool. *Research* what XP's ICM monitor profiles did and did not do.

### sRGB, 1996

- HP and Microsoft named the average PC monitor as a standard, rounded to 2.2, so files, screens and later profiles could agree on something. Standardized as IEC 61966-2-1 in 1999. *Verify* both.
- Not a pure power law: a straight segment near black, then a 2.4 power, approximating 2.2 overall — the formula is on the page below or here, and the reason is noise and invertibility near zero.

### Where a system found out what its monitor did

- A display's ICC profile, with its tone response curve measured at the factory for Apple's own displays.
- The squint test, for everyone else.
- EDID, the block a monitor sends over the cable from the mid-1990s: one byte for gamma, stored as gamma × 100 − 100, so 120 means 2.2. Often just 2.2 whatever the tube did. *Verify* against VESA EDID 1.3.
- Windows XP needed none of these, because it corrected nothing.

### Two conventions for one word

- **Absolute** numbers describe a setup: the Mac's 1.8, calibrators' target gamma, Photoshop's Color Settings, where Apple RGB and ColorMatch RGB are 1.8 and sRGB and Adobe RGB are 2.2.
- **Relative** numbers adjust a picture, with 1.00 as no change and bigger as brighter: Photoshop's Levels midtone, the most-used tonal control in professional work, and every picture viewer and graphics driver slider that followed it.
- Photoshop's Exposure adjustment, from CS2, may put the raw exponent in its gamma box instead, so that lower is brighter. *Verify.* Either way the only reliable test is to type a 2 and watch which way the picture moves.
- The naming rule behind both is written up in the core section below: *S* describes a display, *g* describes a correction.

### A picture made on one looked wrong on the other

- A JPEG has no field for gamma, and neither does JFIF. A file could carry an ICC profile in an APP2 segment, which Photoshop and cameras wrote, or an EXIF color space tag that mostly says sRGB. EXIF also defines a gamma tag almost nothing writes or reads. *Verify* which browsers ignore it.
- PNG is the format that really has a gamma number: the `gAMA` chunk, with `cHRM`, `sRGB` and `iCCP` beside it. Early tools wrote wrong values, and browsers handled it inconsistently for years. *Research* what each engine does today.
- The numbers: a Mac-made gray of 128 is 28.9% of white's light at home and 17.9% on a PC, an exponent of 2.5 ÷ 1.8 = 1.39 too heavy. Correcting it by hand meant a viewer setting of about 1.39.
- Today an untagged file is assumed sRGB, which suits most of a thirty-year pile, since most of it was made on PCs. What comes out wrong is mostly Mac work from before 2009, which looks dark and heavy.
- The pixels cannot tell a dark picture from one encoded for the wrong display, so only the person looking can decide. That is the argument for a manual control.
- Lifting the shadows is also a test: in a JPEG it usually reveals only the encoder's 8 × 8 blocks, which shows there was nothing more down there to find.

## The equation

On a personal computer from before 2009, driving a cathode-ray tube, the whole trip from file to screen is one equation:

<div class="equation">
<math display="block"><mi>h</mi><mo>=</mo><msup><mi>v</mi><mfrac><mi>S</mi><mrow><mi>g</mi><mo>·</mo><mi>T</mi></mrow></mfrac></msup><mspace width="3em" /><mi>L</mi><mo>=</mo><msup><mi>h</mi><mi>T</mi></msup><mo>=</mo><msup><mi>v</mi><mfrac><mi>S</mi><mi>g</mi></mfrac></msup></math>
</div>

| | what it is | where it lives |
| :-: | --- | --- |
| *v* | the file's value, divided by 255 so it runs from 0 to 1 | the file |
| *g* | the viewer's gamma setting: 1 changes nothing, 2 floods the shadows | the application |
| *S* | the system gamma: 1.8 on a Mac, 2.2 on a calibrated PC, 2.5 on one nobody calibrated | the operating system |
| *T* | the tube's gamma, about 2.5, which is physics rather than a setting | the monitor |
| *h* | the number the software hands the hardware, from 0 to 1 | the graphics card |
| *L* | the light the screen emits, where 1 is its white | the glass |

*S* and *g* are both called gamma and work in opposite directions, because a gamma number is named after the display it describes or the display it cancels. **_S_ describes a display**: a system gamma of 1.8 means the machine turns a value into light as a 1.8 power, so a bigger *S* is a darker picture. **_g_ describes a correction**, named for a display it would undo: gamma correction 2.0 undoes a display of gamma 2.0, which takes a power of 1 ÷ 2.0, so a bigger *g* is a brighter one. That is why *S* sits on top of the fraction and *g* underneath it, and why they cancel when they match — a viewer set to 1.8 on a Mac makes the light exactly proportional to the file's value.

The operating system's own share is *S* ÷ *T*. It lives in a lookup table in the graphics card, which applies whatever exponent, multiplied by the tube's, comes out at *S*: 1.8 ÷ 2.5 = 0.72 on the Mac, and 1 on the uncalibrated PC, whose table passes every value through untouched.

### Examples

A flat gray of 128, so *v* = 128 ÷ 255 = 0.502, on a Mac with the viewer at 1:

<div class="equation">
<math display="block"><mi>h</mi><mo>=</mo><msup><mn>0.502</mn><mfrac><mn>1.8</mn><mrow><mn>1</mn><mo>·</mo><mn>2.5</mn></mrow></mfrac></msup><mo>=</mo><msup><mn>0.502</mn><mn>0.72</mn></msup><mo>=</mo><mn>0.609</mn><mspace width="3em" /><mi>L</mi><mo>=</mo><msup><mn>0.502</mn><mn>1.8</mn></msup><mo>=</mo><mn>0.289</mn></math>
</div>

The file says 128, the Mac sends 0.609 × 255 = 155, and the screen shows 28.9% of its white. The same gray everywhere:

| machine | *S* | *h* at *g* = 1 | *h* at *g* = 2 | *L* at *g* = 1 | *L* at *g* = 2 |
| --- | --: | --: | --: | --: | --: |
| Mac | 1.8 | 155 | 199 | 28.9% | 53.8% |
| calibrated PC | 2.2 | 139 | 188 | 22.0% | 46.9% |
| uncalibrated PC | 2.5 | 128 | 181 | 17.9% | 42.3% |

One file and one setting send three different numbers to three machines, and the same gray comes out nearly twice as bright on the Mac as on the uncalibrated PC — which is why a picture made on one looked wrong on the other.

Real hardware of this era rounded to a whole number from 0 to 255 after each step, so a result worked on paper can land one away from what was actually sent.

### From one machine to another

A viewer set to *g* makes a machine of system gamma *S* behave like one of gamma *S* ÷ *g*. So the setting that makes one machine look like another is the ratio of their system gammas:

<div class="equation">
<math display="block"><mi>g</mi><mo>=</mo><mfrac><msub><mi>S</mi><mtext>from</mtext></msub><msub><mi>S</mi><mtext>to</mtext></msub></mfrac></math>
</div>

| making a | look like a | *g* |
| --- | --- | --: |
| raw PC, 2.5 | calibrated PC, 2.2 | 2.5 ÷ 2.2 = 1.14 |
| calibrated PC, 2.2 | Mac, 1.8 | 2.2 ÷ 1.8 = 1.22 |
| raw PC, 2.5 | Mac, 1.8 | 2.5 ÷ 1.8 = 1.39 |

The other way round is the reciprocal of each, 0.88, 0.82 and 0.72, and each of those darkens. A screen today is sRGB, close to 2.2, so the middle row is the one that matters most now: a viewer set to about 1.22 shows a picture made on a Mac before 2009 roughly as its maker saw it.

::: tip Notecards
- This table joins three threads of the page: the equation, the two conventions a user met — an absolute number in the monitor's control panel and a relative one in the photo suite's settings — and numbers worth typing into fuji.
- It says what calibrating actually did: Adobe Gamma taking a PC from 2.5 to 2.2 was the first row, done in the graphics card for the whole screen rather than in one application for one picture.
- The three values are the ones to suggest in fuji: 1.14, 1.22 and 1.39. Fuji's factory `gamma.key` of 1.25, chosen by feel as a gentle lift and nudged up from a first 1.2, sits just past the middle row.
- *Verify* how far sRGB's piecewise curve moves 1.22 from exact, since sRGB is not a pure 2.2 power.
:::

## Today, with color management

- Files and displays both carry ICC profiles, and the engine converts every picture from the file's space to the display's. The page's pixels are in a known space, sRGB or Display P3, rather than in whatever this monitor does.
- So a viewer's number becomes absolute where it used to be relative: fuji's exponent means the same curve on every machine, and each display's own curve is handled after it. Two machines agreeing on the `g` key is consistent with that. *Verify* the order of the steps in WebKit and Chromium from their sources.
- *Research* what the macOS and Windows display pipelines do today, where the table went, and what calibration writes.

## Gamma in fuji

- One number, `gamma` in `gamma.js`, in the viewer's convention, 1 at every launch and never written anywhere, since the next folder may not need it.
- The `g` key, in the shell's one keydown listener beside `c`, because gamma is a way of looking at every view at once: at exactly 1 it brightens to `gamma.key` in `fuji.toml`, factory 1.25, and from any other value it returns to 1. The way in, and always the way back out.
- Shift and a right drag on the table, a slider laid up the frame: anchored where the button went down, only the height counts, linear, adding `gamma.drag` over the frame's height and floored at `gamma.floor`, both in `fuji.toml`. The first build added 2 over the frame and made the user lift a wrist to reach a useful gamma, so the factory value went to 4, 8 and 6, and settled by feel on 5, a fifth of the frame from 1 to 2 — the same distance as five steps of the keys; the floor is 0.2. Where the zoom drag multiplies, this one adds, because it is meant to feel like the slider from 1 to 2 that a user already pictures. Below 1 darkens, a way to check the blacks.
- Shift and the wheel on the table, beside the wheel's flip and ctrl's zoom: one `gamma.wheel` a notch, factory 0.1, half the keys' step because a wheel is easier to flick, away from you to brighten. macOS reports a shift wheel as horizontal, which the direction test already reads. A trackpad sends dozens of small events a gesture and races, as ctrl-wheel zoom does; smoothing both is one job for later.
- Shift with plus and minus, beside the zoom keys: one `gamma.step`, factory 0.2 after a first 0.25, up or down from wherever the gamma is, with the drag's floor. 0.2 is not exact in binary, so each step is rounded to a millionth, or a walk up and back could miss 1 by a hair and leave the filter on at an exponent of 0.9999999. It took shift and the main row's `=` key away from zoom, which now zooms in on the unshifted `=`.
- The mechanism is one SVG filter, one class on the root, and one CSS rule. Snippets from `Shell.vue`, to be refreshed from the code when the page is written, since the exponent is now `1 / gamma` and the class follows the number:

```html
<filter id="fujiGamma" color-interpolation-filters="sRGB" x="0" y="0" width="1" height="1">
	<feComponentTransfer>
		<feFuncR type="gamma" :exponent="gammaExponent" />
		<feFuncG type="gamma" :exponent="gammaExponent" />
		<feFuncB type="gamma" :exponent="gammaExponent" />
	</feComponentTransfer>
</filter>
```

```css
.gamma .myTile, .gamma .myImage {
	filter: url(#fujiGamma);
}
```

```js
function toggleGamma() { document.documentElement.classList.toggle('gamma') }
```

- `feComponentTransfer`'s gamma type computes amplitude × in<sup>exponent</sup> + offset per channel, and alpha is left alone. The exponent is 1 ÷ *g*, so fuji's setting reads in the same convention as a photo suite's.
- What the first build's fixed 2.0 did, an exponent of 0.5: 2% → 14%, 10% → 32%, 50% → 71%, 90% → 95%. Strong in the shadows and heavy in the midtones, which is why the key settled on a gentle 1.25 and the strong values moved to the drag, where they are chosen picture by picture.
- A lens rather than an edit: the canvases keep what the operating system handed them and the store keeps its decode, so nothing is read, drawn or decoded again, and a tap costs the same for six pictures or six hundred.
- Off is no filter at all rather than an exponent of 1, so the pictures at rest are exactly what fuji's fidelity measurements describe.
- sRGB rather than the linearRGB an SVG filter defaults to: a power curve comes out nearly the same in either space, because powers compose — (x<sup>2.2</sup>)<sup>e</sup> = (x<sup>e</sup>)<sup>2.2</sup> — and staying in sRGB spares an 8-bit round trip to linear light that would merge the shadow codes gamma exists to separate.
- The filter region is the element's own box, where the default reaches a tenth past each edge for nothing.
- Why not CSS's own filter functions: `brightness()` multiplies and `contrast()` scales about the middle, and neither is a power curve.
- Alternatives set aside: `backdrop-filter` on an overlay, which catches the HUD and the dots and supports `url()` unevenly; the filter on the sheet's scroll container, which makes the whole scroll one filtered surface; and `mix-blend-mode: screen` over a duplicate, 1 − (1 − x)², close to a gamma of 1.6, which the table cannot use because its image is the store's one element.
- Measured by eye: instant on the Mac and on Windows, no flicker, like a light switch. *Record* which Mac. WebKitGTK draws SVG filters on the CPU, so a full-window table image on Linux or the Pi is the case still to watch.
- Found in the first smoke test of the drag: WebKit does not redraw an element when the SVG filter it is already showing through changes underneath it. The hud's number moved with the drag and the picture stayed put, until leaving fullscreen forced a fresh draw and the right gamma appeared. The fix is two filters taking turns: write the exponent into the one nothing is using, then point a CSS custom property at it, so every change is a new `filter` value the engine has to rebuild for. A good story for the page — a bug with a clean symptom and a mechanism.
- Display P3 may not survive the filter. Accepted rather than tested.
- Prior art in fuji: `MyLens.vue`, a year earlier, used the same primitive with a live exponent and `filter: none` at 1, in the linearRGB default. Its comment on the scale — 0.9 slight, 0.5 strong, 0 whiteout — is right, and a slider feels more even stepped by multiplying the exponent than by subtracting from it.

## Averaging in the wrong space

- Averaging encoded values is not averaging light. Black and white average to 128 as codes, but 50% light encodes to about 188, and code 128 is only about 22% of white's light.
- So any resample done on encoded values darkens fine, high-contrast detail: stripes, text, stars, leaves against sky. Browsers and most operating system scalers do it that way. *Verify* which.
- Fuji's own evidence: the resampled test tiles on the thumbnail pipeline page came out a flat `rgb(128,128,128)`, the encoded average, where light-correct averaging gives about 188. Every downscaled thumbnail carries a subtler version of the same thing.
- Scaling in linear light is the textbook fix, and a fidelity option for later.

## Sources

To gather, and to cite beside the facts above: Charles Poynton's gamma FAQ and his books; Hurter and Driffield, 1890; IEC 61966-2-1 for sRGB; ITU-R BT.709 and BT.1886; the PNG specification's `gAMA` chunk; VESA EDID 1.3; Apple's own notes on 1.8 and on the change in Snow Leopard; Adobe's documentation for Adobe Gamma and Levels; CIE L*; Microsoft's `SetDeviceGammaRamp`.

<style scoped>
.equation {
	overflow-x: auto; /* a long equation scrolls on a narrow screen rather than widening the page */
	overflow-y: hidden; /* said outright, because auto on one axis turns the other from visible to auto, and a math font's fractions reach a pixel or two past the box they report, which drew a scrollbar beside every equation */
	padding: 0.4em 0; /* the room that reach needs, so hiding it clips nothing */
	margin: 0.85rem 0;
}
.equation math {
	font-family: 'STIX Two Math', 'Cambria Math', 'Latin Modern Math', math; /* a face with a MATH table, which is what lays out fractions and superscripts properly: STIX Two Math ships with macOS and Cambria Math with Windows */
	font-size: 1.3em;
}
</style>
