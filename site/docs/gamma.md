# Gamma

A pixel's stored value and the light a screen makes from it are related by a power curve, and the exponent of that curve is called gamma.

That one sentence has a hundred and thirty years behind it. The word was coined in a photographic darkroom in 1890, borrowed by television engineers in the 1950s, carried into personal computers by two companies that chose different numbers, argued over on the web of the 1990s by people who could see that their pictures looked wrong on the other kind of machine, and finally settled by a standard that named the average monitor and called it done. Fuji has a gamma control, one key and a drag, and this page is why it does and what the number means. The history comes first, because the number is meaningless without it; the equation is in the middle, because it is the destination; and how Fuji draws it is last.

## Film

In 1890 two amateur photographers in Widnes, a chemical town on the Mersey, published [a paper](https://archive.org/details/photo-chemical-investigations-hurter-driffield-1890) in the Journal of the Society of Chemical Industry that made photography a science. Ferdinand Hurter and Vero Driffield exposed a plate to a series of known light doses, developed it, measured how dark each patch had come out, and plotted density against the logarithm of exposure. The curve that appeared was an S: a toe where the plate barely responds, a long straight middle, and a shoulder where it saturates. They called the slope of that straight middle gamma, and the whole thing has been [the characteristic curve](https://en.wikipedia.org/wiki/Sensitometry), or the H&D curve, ever since. A high gamma is a contrasty film, where a small change in light makes a big change in density; a low gamma is a flat one. A developer could raise it by developing longer, and a lab measured it to know its process was in control.

The word outlived the medium because of a piece of mathematics. A power law, light out as some power of signal in, is a straight line on log-log axes, and its slope is the exponent, so on Hurter and Driffield's paper "gamma" and "exponent" were the same thing. Every later use of the word is that slope, of whatever curve is in front of the engineer at the time.

Film also gave us the gray card. The gray that looks halfway between black and white to a human eye reflects about 18 percent of the light that falls on it, not 50, and a photographer's [18% gray card](https://en.wikipedia.org/wiki/Middle_gray) is that gray, made to set an exposure by. The same fact appears in the CIE's [lightness scale](https://en.wikipedia.org/wiki/Lightness), where L* of 50, the perceptual middle, is 18.4 percent of white's light. Remember that number. It explains everything that follows.

## Television

A cathode-ray tube makes light by firing electrons at a phosphor, and the amount of light is not proportional to the voltage that drives the gun. It rises as roughly the 2.5 power of it: the beam current follows a three-halves law from the physics of space charge, and the gun's geometry steepens it further, so that a real tube lands near 2.5 whatever its maker intended. [Charles Poynton's Gamma FAQ](https://poynton.ca/GammaFAQ.html), the reference the whole field cites, puts the number there and explains where it comes from; [the tube's own article](https://en.wikipedia.org/wiki/Cathode-ray_tube) has the physics.

The broadcasters of the 1950s could have put a correcting circuit in every receiver. Instead they corrected once, at the camera, and [the NTSC standard of 1953](https://en.wikipedia.org/wiki/NTSC) had the camera encode light to a power of about 1 ÷ 2.2 so that the millions of sets to come needed no circuit at all. The camera's curve and the tube's nearly cancel. The engineers left the small difference between 2.2 and 2.5 in deliberately, for a reason the room explains below. Every television picture ever broadcast is gamma-encoded, and so, because television's engineers built the first computer displays, is every pixel a computer has ever stored.

Poynton points at a coincidence, and it explains why gamma outlived the tube. Human vision is logarithmic: we sense brightness as a ratio, the [Weber-Fechner law](https://en.wikipedia.org/wiki/Weber%E2%80%93Fechner_law), and the CIE's lightness scale, fitted to what people report, is roughly the cube root of the light. A curve that raises signal to the 2.2 power to make light is, run backward, a curve close to a cube root. So a signal encoded for a tube is also a signal spaced nearly evenly for an eye. With only eight bits a code, that matters enormously. Linear codes would put half of them in the top stop of brightness, where the eye can tell almost none of them apart, and leave the shadows with codes 1, 2 and 3 as jumps of a hundred percent and fifty percent, which band visibly. Gamma is a compression scheme that spends bits where the eye can use them, and flat panels, which have no gun and no reason of physics for any curve, emulate the tube's curve deliberately, to keep it.

The difference between 2.2 and 2.5 is the room. Bartleson and Breneman showed in 1967 that a picture watched in dim surroundings looks flatter than the same picture in a bright room, so a television, watched in the evening with the lights down, needs a little more contrast than the scene had to look right. So the engineers left the system as a whole, camera through tube, at an overall gamma of about 1.1 to 1.2 rather than exactly 1, and every video standard since has kept [that finding](https://www.imaging.org/common/uploaded%20files/pdfs/Papers/1997/RP-0-67/2355.pdf). Those standards now name the two sides separately: the opto-electronic transfer function, the camera's curve, in [Rec. 709](https://www.itu.int/rec/R-REC-BT.709), and the electro-optical one, the display's, in [BT.1886](https://www.itu.int/rec/R-REC-BT.1886) of 2011, as a pure 2.4 power. High dynamic range's PQ and HLG curves leave power laws behind altogether, and are another story.

## Personal computers

### The chain inside the machine

A personal computer of the tube era turned a stored value into light in four steps: the framebuffer held the number, a lookup table in the graphics card translated it, a digital-to-analog converter made a voltage, and the tube made light. The table is the part that matters here. It held 256 entries per channel, one for every value a byte can take, and the card applied it at scanout to the whole screen, every application at once, for free, and invisibly. No program could see it, and any program that wrote to it changed the picture for every other program on the machine.

That table is where an operating system's gamma lived, and its exponent is the *S* ÷ *T* term in the equation below. A curved table costs precision, because it is eight bits in and eight bits out: it spreads the shadow codes apart, leaving gaps that show as faint bands in a smooth gradient, and pushes highlight codes together until neighbors merge. Professional cards answered with wider tables, ten bits on Matrox's Parhelia in 2002 and on the workstation cards, and a whole small industry of calibration existed to write the right curve into them.

### The Mac chose a system gamma

The Macintosh of 1984 corrected its display from the start, to a system gamma of 1.8, and Apple chose the number for print. Apple's business was desktop publishing, and the LaserWriter of 1985 put ink on paper with a dot gain that a screen at 1.8 happened to match, so what a designer saw on the monitor was close to what came out of the printer. That made the Mac the one personal computer whose gamma was a decision rather than an accident, and the decision held for a quarter century.

The mechanism was ColorSync, Apple's [color management system](https://en.wikipedia.org/wiki/ColorSync), which at login wrote a curve into the graphics card's table from two things: the display's profile, which for Apple's own monitors held a tone response measured at the factory, and a target the user chose in the Monitors control panel and later the Display Calibrator Assistant. The default target was 1.8. The calibrator measured the display with the squint test, the one piece of all this a user could see. A solid gray Apple logo sat on a field of fine black and white stripes, and the user moved a slider until the logo vanished into the field. The stripes are exactly half light on any monitor whatever its gamma, since they are half black and half white, so the gray that matches them is the gray that makes 50 percent light, and that gives one equation in one unknown: a matching code of 186 means the display's gamma is about 2.2, and 194 means about 2.5.

The 1.8 held from the first Mac through the Bondi Blue iMac to Mac OS X 10.6, Snow Leopard, which in 2009 moved the default to 2.2 with a note that the old value had been geared toward print and the new one suited content made for screens. [One photographer's account of the change](https://www.gballard.net/photoshop/osx_22_gamma.html), written at the time, has the details and the arguments.

### Windows inherited a monitor gamma

Windows made no such choice. There was no gamma setting in the operating system and no curve in the table, so the system gamma of a PC was whatever its tube did, about 2.5, and it drifted with the monitor's brightness and contrast knobs, since the brightness knob sets the black level and so the shape of the curve. Two PCs side by side showed the same file differently, and neither owner knew.

The table was there all the same, reachable through [SetDeviceGammaRamp](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setdevicegammaramp), and three kinds of program wrote to it. Games did, whose brightness sliders wrote the whole screen's table, which is why the desktop sometimes stayed bright after a game crashed. Adobe Gamma did: it shipped with Photoshop from version 5, walked the user through a squint test of its own, wrote a profile, and put a loader in the Startup folder so it could write the curve at every boot, since Windows itself would not do it. Its dialog offered two targets by name, Windows Default at 2.2 and Macintosh Default at 1.8, and those two labels are as close as the era came to saying out loud that the platforms disagreed. And the NVIDIA and ATI control panels did, with sliders that started at 1.0 and meant a relative correction, the other convention below.

Windows XP could install a monitor's profile, but installing it loaded nothing into the card; that still needed a loader, Adobe's or the monitor maker's. Windows 7, in 2009, the same year as Snow Leopard, brought a built-in display calibration tool, and for the first time the operating system itself wrote the table at login. Both platforms arrived at 2.2 in the same year from opposite directions.

### sRGB, 1996

By the middle of the 1990s the web was sending pictures to millions of monitors nobody had measured, and in November 1996 Michael Stokes and Ricardo Motta of Hewlett-Packard and Matthew Anderson and Srinivasan Chandrasekar of Microsoft [proposed a standard default color space for the internet](https://www.w3.org/Graphics/Color/sRGB.html). Its idea was modest and decisive: describe the average PC monitor under Windows, in a typical office, and name it, so that a file, a screen and a printer could all agree on what a value meant without measuring anything. They rounded the gamma to 2.2, took the primaries from a tube's phosphors, and set the white to daylight. It became [IEC 61966-2-1](https://en.wikipedia.org/wiki/SRGB) in 1999, and it is still what nearly every picture on the web is in.

Its curve is not a pure power law, for a practical reason. Near black, a power curve has an infinite slope, which is impossible to invert cleanly and turns sensor noise into visible garbage, so sRGB uses a short straight segment near zero and a 2.4 power above it, which together approximate 2.2 overall. That detail is why calculations on this page that say 2.2 are approximate, and the section on moving between machines says by how much.

### Where a system found out what its monitor did

Three ways, and a fourth that was not trying. A display's ICC profile could carry its tone response curve, measured at the factory, which Apple did for its own monitors and almost nobody else did for anyone's. The squint test, in ColorSync's calibrator or Adobe Gamma, let a user measure it by eye. And from the mid-1990s a monitor sent an EDID block over the cable, and [EDID 1.3](https://glenwing.github.io/docs/VESA-EEDID-A1.pdf) reserved one byte for gamma, stored as the gamma times a hundred minus a hundred, so 120 means 2.2. Monitor makers wrote 2.2 in it whatever the tube did, since 2.2 was what everyone expected, which made the byte a statement of intent rather than a measurement. Windows XP needed none of the three, because it corrected nothing.

### Two conventions for one word

A user of this era met gamma as a number in two places, and the two numbers pointed in opposite directions, and the equation below exists to dissolve that confusion.

An **absolute** number describes a setup. The Mac's 1.8, a calibrator's target, and Photoshop's Color Settings, where Apple RGB and ColorMatch RGB are 1.8 and sRGB and Adobe RGB are 2.2, all say how a display turns a value into light: a bigger number is a darker picture, because the curve bows lower.

A **relative** number adjusts a picture, with 1.00 meaning no change and a bigger number meaning brighter. Photoshop's Levels dialog has a midtone slider that reads 1.00 in the middle and runs to 9.99, and it is the most used tonal control in professional work; every picture viewer, game and graphics driver slider that followed copied its sense. Photoshop's own Exposure adjustment, from CS2, may put the raw exponent in its gamma box instead, so that the sense is reversed. Either way the only reliable test of any gamma control is to type a 2 and watch which way the picture moves.

The equation states the rule behind both: *S* describes a display, *g* describes a correction, and a correction is named for the display it would undo.

### A picture made on one looked wrong on the other

From the first web browsers until 2009, a picture made on a Mac and looked at on a PC came out dark and heavy, and a picture made on a PC and looked at on a Mac came out light and washed out, and nobody's software said why. The pixels were identical. What differed was what each machine did with them: a Mac-made gray of 128 was 28.9 percent of white's light at home and 17.9 percent on an uncalibrated PC, an exponent of 2.5 ÷ 1.8 = 1.39 too heavy, a difference anyone can see. Web designers of the late 1990s knew it as "the gamma problem," argued about it on every forum, and split the difference by making graphics at a gamma of 2.0 that were slightly wrong everywhere. [One trade column of the time](https://creativepro.com/the-web-wizard-keeping-colors-consistent-on-macs-and-pcs/) walks through the workarounds.

It fed a tribalism that was already there. The Mac was the machine of designers, printers and photographers, and its 1.8 was a print number; the PC was the machine of offices and games, and its 2.5 was a television number that nobody had chosen. Each side saw the other's pictures as wrong and had the evidence on its own screen. Apple's [Get a Mac](https://en.wikipedia.org/wiki/Get_a_Mac) commercials of 2006 were the moment that feeling became advertising, Three years later both platforms arrived at 2.2, one by decision and one by a calibration tool, and the argument ended.

The file formats could not help, because they did not carry the number. A JPEG has no field for gamma and neither does JFIF; a file could carry a whole ICC profile in an APP2 segment, which Photoshop and cameras wrote, or an Exif tag that mostly says sRGB, and browsers ignored both for a decade. PNG alone really has a gamma number, [the gAMA chunk](https://www.w3.org/TR/png/#11gAMA), with chromaticities and a profile beside it, and its designers wrote [a tutorial on gamma](https://www.w3.org/TR/PNG-GammaAppendix) into the specification to explain why. Early tools wrote wrong values into it, browsers handled it inconsistently for years, and as of this writing Chrome and Safari mostly ignore the chunk alone while Firefox honors it, so a PNG with a gAMA chunk and no profile can still look different in two browsers on the same machine.

Today every engine assumes an untagged file is sRGB, which suits most of a thirty-year pile of pictures, since PCs made most of it. What comes out wrong is mostly Mac work from before 2009, which looks dark and heavy on every screen there is now. And the pixels cannot tell a dark picture from one encoded for a display that no longer exists; only the person looking can. That is why a viewer needs a manual control, and why Fuji has one. Lifting the shadows is also a test: in a JPEG it usually reveals only the encoder's 8 by 8 blocks, which shows there was nothing more down there to find.

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

That table joins the three threads of this page. It is the equation, applied. It is the two conventions a user met, an absolute number in the monitor's control panel and a relative one in the photo suite's settings, shown to be one ratio. And it is what calibrating actually did: Adobe Gamma taking a PC from 2.5 to 2.2 was the first row, done in the graphics card for the whole screen rather than in one application for one picture.

Three values, then, to type into a viewer: 1.14, 1.22 and 1.39. Fuji's factory setting for the `G` key is 1.22, the middle row, chosen for exactly this reason after a first 1.2 and then a 1.25 that had been picked by feel. sRGB's curve is not a pure 2.2 power, so the middle row was checked against the real curve: across the midtones the exponent that best matches a 1.8 display on an sRGB screen is still 1.22, and a Mac-made gray of 128 comes out at 28.3 percent of white through it against the 28.9 percent its maker saw.

## Today, with color management

The chain inside the machine has a fifth step now, and it changes what a viewer's number means. Files and displays both carry ICC profiles, and the web engine converts every picture from the file's space to the display's before it is drawn, so the page's pixels are in a known space, sRGB or Display P3, rather than in whatever this monitor happens to do. The lookup table is still there, written at login from the display's profile, but nothing above it has to know.

So a viewer's gamma becomes absolute where it used to be relative. Fuji's exponent means the same curve on every machine, because the engine applies each display's own curve after it and independently of it; two machines that agree on the number show the same lift. So one number in `fuji.toml` serves every machine, Mac and Windows alike.

## Gamma in Fuji

Fuji holds one number, in the viewer's convention: 1 changes nothing and a bigger number lifts the shadows. It is 1 at every launch and never written anywhere, since the next folder may not need it. Four controls change it, and they work together.

- **The `G` key.** At exactly 1 it brightens to the value of `gamma.key` in `fuji.toml`, factory 1.22, the ratio of a modern screen's 2.2 to the classic Mac's 1.8; from any other value it returns to exactly 1. The way in, and always the way back out. It sits in the shell's one keydown listener beside `Esc`, because gamma is a way of looking at every view at once rather than something one view does.
- **`Shift` with `+` and `-`.** One `gamma.step`, factory 0.2, up or down from wherever the gamma is. Fuji rounds each step to a millionth, because 0.2 is not exact in binary and a walk up and back could otherwise miss 1 by a hair and leave the filter on at an exponent of 0.9999999, which the key would then not read as normal.
- **`Shift` and the wheel, on a light table.** One `gamma.wheel` a notch, factory 0.1, half the keys' step because a wheel is easier to flick, away from you to brighten.
- **`Shift` and a right drag, on a light table.** A slider laid up the frame: anchored where the button went down, only the height counts, and it adds `gamma.drag`, factory 5, over the frame's whole height, so a fifth of the frame takes 1 to 2, the same as five steps of the keys. Where the zoom drag multiplies, this one adds, because it should feel like the 1-to-2 slider a user already pictures. Below 1 darkens, which is a way to check the blacks, and everything stops at `gamma.floor`, factory 0.2, since a drag that reached 0 would be a power of infinity and a black screen.

The first build's drag added 2 over the frame and made the user lift a wrist to reach a useful gamma, so the factory value went to 4, 8 and 6, and settled by feel on 5. The first build's key was a fixed 2.0, an exponent of 0.5, which takes 2 percent to 14, 10 to 32, 50 to 71 and 90 to 95: strong in the shadows and heavy in the midtones. So the key settled on a gentle value, 1.25 by feel and then 1.22 by history, and the strong values moved to the drag, where the user picks them picture by picture.

The mechanism is one SVG filter, one custom property, and one CSS rule. The filter is a single primitive, [feComponentTransfer](https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Element/feComponentTransfer) from the [Filter Effects specification](https://www.w3.org/TR/filter-effects-1/#feComponentTransferElement), whose gamma type computes amplitude × in<sup>exponent</sup> + offset on each channel scaled from 0 to 1, leaving alpha alone, so black stays black and white stays white while the shadows lift. The exponent is 1 ÷ *g*, so Fuji's setting reads in the same convention as a photo suite's.

```html
<svg aria-hidden="true" width="0" height="0" class="absolute w-0 h-0">
	<filter v-for="n in [0, 1]" :key="n" :id="`gammaFilter${n}`" color-interpolation-filters="sRGB" x="0" y="0" width="1" height="1">
		<feComponentTransfer>
			<feFuncR type="gamma" exponent="1" />
			<feFuncG type="gamma" exponent="1" />
			<feFuncB type="gamma" exponent="1" />
		</feComponentTransfer>
	</filter>
</svg>
```

The rule that applies it reaches every tile on the contact sheet and the picture on the light table through one custom property, and its fallback, when the property is unset, is no filter at all:

```css
.myTile, .myImage {
	filter: var(--gamma-filter, none);
}
```

And the watch that sets the property is where the two filters take turns, for the reason below:

```js
let gammaFilter = 0
watch(gamma, value => {
	let root = document.documentElement
	if (value == 1) { root.style.removeProperty('--gamma-filter'); return }
	gammaFilter = 1 - gammaFilter
	for (let f of document.getElementById(`gammaFilter${gammaFilter}`).firstElementChild.children) f.setAttribute('exponent', 1 / value)
	root.style.setProperty('--gamma-filter', `url(#gammaFilter${gammaFilter})`)
})
```

There are two filters rather than one, and they take turns, because of a bug with a clean symptom and a clean mechanism. The first smoke test of the drag, on a Mac, moved the number on the hud and left the picture exactly where it was, until leaving fullscreen forced a fresh draw and the right gamma appeared all at once. WebKit does not redraw an element when a filter it is already showing through changes underneath it. The fix is to write the new exponent into the filter nothing is using, and only then point the pictures at it through the custom property, so that every change is a new `filter` value, and a new filter value is something every engine has to rebuild for.

Four smaller decisions follow from the same principle, which is that gamma is a lens and not an edit. The canvases keep what the operating system handed them and the store keeps its decode, so Fuji reads, draws and decodes nothing again, and a tap costs the same for six pictures or six hundred. Off is no filter at all rather than an exponent of 1, so the pictures at rest are exactly what [the thumbnails page](./thumbnails.html) measured. The filters run in sRGB rather than the linearRGB an SVG filter defaults to, because a power curve comes out nearly the same in either space, since powers compose, and staying in sRGB spares a round trip to linear light that at eight bits would merge the very shadow codes gamma exists to pull apart. And the filter region is the element's own box, where the default reaches a tenth past each edge for nothing.

CSS's own filter functions were not an option: `brightness()` multiplies and `contrast()` scales about the middle, and neither is a power curve. Three alternatives lost. A `backdrop-filter` on an overlay catches the hud and the dots and supports `url()` unevenly. A filter on the sheet's scroll container makes the whole scroll one filtered surface. And `mix-blend-mode: screen` over a duplicate of the picture computes 1 − (1 − x)², close to a gamma of 1.6, but the light table shows the store's one element and cannot duplicate it. A year earlier, an experiment in Fuji's own history had used the same primitive with a live exponent and `filter: none` at 1, and its comment on the scale, 0.9 slight, 0.5 strong, 0 whiteout, is right; a slider feels more even stepped by multiplying the exponent than by subtracting from it.

Measured by eye, the switch is instant on the Mac mini and on the Windows box, with no flicker, like a light switch. WebKitGTK draws SVG filters on the CPU, so a full-window image on Linux or a Raspberry Pi is the case still to watch. Whether Display P3 survives the filter is accepted rather than tested.

## Averaging in the wrong space

Everything above is about one pixel. Gamma has a second consequence that appears whenever pixels are combined, and every scaled thumbnail carries it.

Averaging encoded values is not averaging light. Black and white average to 128 as codes, but half of white's light encodes to about 188 in sRGB, and code 128 is only about 22 percent of white's light. So any resample done on encoded values darkens fine, high-contrast detail: stripes, text, stars, leaves against sky, anything where black and white pixels are being averaged into gray. Browsers and most operating system scalers do it that way, because doing it in linear light costs a conversion each way and, at eight bits, the precision the shadows need; Skia, Chromium's renderer, does its arithmetic in the encoded space of the surface it draws to, as [its maintainers say](https://groups.google.com/g/skia-discuss/c/wRA91qTkybY).

Fuji has its own evidence. The resampled test tiles on the thumbnails page came out a flat `rgb(128, 128, 128)`, the encoded average of their stripes, where light-correct averaging gives about 188, and every downscaled thumbnail carries a subtler version of the same thing. Scaling in linear light is the textbook fix, and a fidelity option for later.

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
