# Zoom

Four ways to zoom on the light table: the wheel and the keys by notches, a right drag smoothly, and the number keys straight to a round number. One rule makes them feel right: the keys and the wheel zoom about the center of the screen, and a right drag zooms about the point you grabbed.

| do this | to |
| --- | --- |
| `Ctrl` and the **mouse wheel** | zoom in and out, a notch at a time |
| `+` and `-` | zoom in and out, a notch a press; `=` is `+` without the shift, and `Ctrl` with any of them works too |
| **right drag** up or down | zoom smoothly: up to come in, down to go out, 200 pixels of drag to double or halve |
| `1` to `6` | exactly that many screen pixels per picture pixel; `1` is actual size |
| `F` | the whole picture on the screen |
| `W` | the picture's width across the screen |
| `Space` | the picture sized to the diamond, centered |

The wheel alone flips to the next picture, and `Shift` with the wheel is gamma, so hold `Ctrl` to zoom. A notch is a little under one and a half times, six notches are exactly ten times, and the curve that gives is below. The 200 pixels of a right drag are `zoom.drag` in `fuji.toml`.

## Where the zoom holds still

Every zoom holds one point of the picture still and grows or shrinks everything around it, and which point that is decides how it feels.

**The keys and the wheel hold the center of the screen still.** Pan until the part you care about is in the middle, then zoom, and it stays in the middle while the rest of the picture grows around it or shrinks toward it. Most viewers zoom about the center of the picture instead, which sends the part you were looking at sliding toward the edge with every step and has you chasing it back. Fuji's choice has a second effect that is easy to miss: a picture that is off center drifts further out as you zoom in, and comes back toward the center as you zoom out. So zooming out always brings a lost picture home, and `Space` brings it home at once.

**A right drag holds the point you grabbed still.** Put the pointer on a detail anywhere on the screen, hold the right button, and drag up: the detail stays under the pointer and the picture grows around it. Drag down to shrink it. The zoom is set by how far above the start the pointer is, not by how far it has traveled, so dragging back to where you started puts the picture back exactly as it was, and a drag sideways does nothing.

## What a notch is

`+`, `-` and each notch of the wheel multiply the picture's size by the sixth root of ten, about 1.468. The notches compound, so three in make the picture a little over three times larger, √10, and six in make it exactly ten times larger; six back out return it to exactly where it was, because a root of ten undoes itself. The right drag is the same curve read off the mouse, two times per 200 pixels of height, so the two controls agree: a notch is about half a doubling, and two notches are a little more than a 200-pixel drag.

A notch never lands on a round number on its own. From `1`, the notches go 1.47, 2.15, 3.16, 4.64, 6.81, 10. That is what the number keys are for.

## The number keys

`1` to `6` are the fourth way, and the only one that goes straight to a round number. Press `2` and every pixel of the picture is drawn on a two-by-two block of the screen's, `3` on three-by-three, and so on to `6`, about the center of the screen like the other keys, so the part in the middle stays in the middle. `1` is actual size, one picture pixel to one screen pixel, which is how a picture looks in the program that made it. The main row and the number pad both work.

They matter for small pictures. An icon, a sprite, a GIF drawn pixel by pixel in 1996: `F` would blow it up to fill the screen at some awkward ratio, and `4` shows it four times its size with every pixel a clean square, which is the one zoom where the picture's own pixels line up with the screen's. That lining up is what the `R` key on the [Ben Day page](./ben-day.html) is for.

## ACDSee's ladder, and the curve inside it

The curve under Fuji's wheel is the one inside ACDSee's ladder. [ACDSee](https://www.acdsee.com/) was the picture viewer of the first multimedia decade, [first sold over bulletin boards in November 1994 for fifteen dollars](https://en.wikipedia.org/wiki/ACDSee) and written by David S. Hooper, and ACD Systems still sells it today. ACDSee 32 version 2.3, from 1998, zooms by a ladder of fixed sizes, a rung a notch. Its rungs above actual size, and beside them the same six notches in Fuji from `1`:

| notches in | ACDSee | Fuji | ACDSee is |
| :-: | --: | --: | --: |
| 0 | 1 | 1 | the same |
| 1 | 1.5 | 1.468 | 2.2% above |
| 2 | 2 | 2.154 | 7.2% below |
| 3 | 3 | 3.162 | 5.1% below |
| 4 | 5 | 4.642 | 7.7% above |
| 5 | 7 | 6.813 | 2.7% above |
| 6 | 10 | 10 | the same |

After one notch a picture 200 pixels wide is 300 pixels wide in ACDSee and 294 in Fuji; after six it is 2000 in both. Below actual size ACDSee's rungs are 70, 50, 30, 20, 15 and 10 percent, the reciprocals of the rungs above rounded the same way, and its ladder ends one rung further down at 5 percent, a floor rather than the curve's 7. ACDSee's column is a list its authors chose, and it has a curve underneath it that they never had to write down, which is Fuji's column. The *k*-th rung is ten to the *k* over six, rounded to a round number:

<div class="equation">
<math display="block"><msub><mi>z</mi><mi>k</mi></msub><mo>=</mo><mi>round</mi><mo>(</mo><msup><mn>10</mn><mfrac><mi>k</mi><mn>6</mn></mfrac></msup><mo>)</mo><mspace width="3em" /><msub><mi>z</mi><mrow><mi>k</mi><mo>+</mo><mn>6</mn></mrow></msub><mo>=</mo><mn>10</mn><mo>·</mo><msub><mi>z</mi><mi>k</mi></msub></math>
</div>

Six steps to a decade, each a little under one and a half, the sixth root of ten; three steps is √10, about 3.16. The bare curve passes through 1 and 10 and misses every rung between, and the rounding is a choice a person made: a geometric spacing pulled onto numbers a person can say, the same move as the engineer's 1, 2, 5 series with 1.5, 3 and 7 filled in.

The rounding is a small move. ACDSee's rungs weave above and below Fuji's curve, two above, two below, two on it, and the farthest strays less than eight percent, about a fifth of one step. Seen from one rung to the next, the gaps come out 1.5, 1.33, 1.5, 1.67, 1.4 and 1.43 instead of 1.468 every time: uneven by a sixth of a step at most, and every one of them reads to the eye as about one and a half.

<div class="diagram">
<svg viewBox="0 0 640 250" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="The six rungs of one decade plotted as their shift from the smooth curve, weaving above and below a zero line and staying inside a band of plus and minus ten percent">
	<line class="band" x1="40" y1="10" x2="580" y2="10" />
	<line class="band" x1="40" y1="210" x2="580" y2="210" />
	<line class="curve" x1="40" y1="110" x2="580" y2="110" />
	<polyline class="weave" points="40,110 130,88 220,182 310,161 400,33 490,83 580,110" />
	<g class="rung">
		<circle cx="40" cy="110" r="5" /><circle cx="130" cy="88" r="5" /><circle cx="220" cy="182" r="5" /><circle cx="310" cy="161" r="5" /><circle cx="400" cy="33" r="5" /><circle cx="490" cy="83" r="5" /><circle cx="580" cy="110" r="5" />
	</g>
	<g class="label">
		<text x="40" y="132" text-anchor="middle">1</text>
		<text x="130" y="76" text-anchor="middle">1.5</text>
		<text x="220" y="202" text-anchor="middle">2</text>
		<text x="310" y="181" text-anchor="middle">3</text>
		<text x="400" y="21" text-anchor="middle">5</text>
		<text x="490" y="71" text-anchor="middle">7</text>
		<text x="580" y="132" text-anchor="middle">10</text>
	</g>
	<g class="axis">
		<text x="600" y="14">+10%</text>
		<text x="600" y="114">Fuji</text>
		<text x="600" y="214">−10%</text>
		<text x="40" y="240" text-anchor="middle">0</text>
		<text x="130" y="240" text-anchor="middle">1</text>
		<text x="220" y="240" text-anchor="middle">2</text>
		<text x="310" y="240" text-anchor="middle">3</text>
		<text x="400" y="240" text-anchor="middle">4</text>
		<text x="490" y="240" text-anchor="middle">5</text>
		<text x="580" y="240" text-anchor="middle">6 notches</text>
	</g>
</svg>
</div>

The straight line is Fuji's curve, which on this chart is the zero every rung is measured from; each dot is one of ACDSee's rungs, placed by how far its round number sits from Fuji's; the dashed lines are ten percent either way, and no rung reaches them.

Fuji's wheel follows the curve and leaves the rounding out. A notch is the sixth root of ten from wherever the picture is, so Fuji lands on ACDSee's 1 and 10 exactly and passes within eight percent of every rung between, and it does the same from any size at all, after a drag or `F` or a number key, with no nearest rung to find first. The round numbers that ACDSee's rounding reached for are the number keys, which reach them exactly. ACDSee's ladder had a curve inside it all along, and Fuji's wheel is that curve. That is the homage a good list deserves.

## The zoom stays when you flip

Flip to the next picture and the zoom is still the zoom. The size is one number, the diamond, and neither it nor where you have panned to changes when the picture does, which is ***Viewport Persistence***, and the [Diamond Table page](./diamond-table.html) says why the light table is built around it. Zoom in on one corner of a frame and flip through a folder, and every picture arrives zoomed in on the same corner.

<style scoped>
.diagram {
	max-width: 40rem;
	margin: 1.5rem auto;
}
.diagram svg {
	width: 100%;
	height: auto;
	display: block;
	font-family: var(--vp-font-family-base);
	font-size: 13px;
}
.diagram .band { stroke: var(--vp-c-text-3); stroke-width: 1; stroke-dasharray: 4 4; }
.diagram .curve { stroke: var(--vp-c-text-1); stroke-width: 1.5; }
.diagram .weave { fill: none; stroke: var(--vp-c-brand-1); stroke-width: 1.5; }
.diagram .rung circle { fill: var(--vp-c-brand-1); }
.diagram .label text { fill: var(--vp-c-text-1); }
.diagram .axis text { fill: var(--vp-c-text-2); }
.equation {
	overflow-x: auto; /* a long equation scrolls on a narrow screen rather than widening the page */
	overflow-y: hidden; /* said outright, because auto on one axis turns the other from visible to auto, and a math font's fractions reach a pixel or two past the box they report, which drew a scrollbar beside every equation on the Gamma page */
	padding: 0.4em 0; /* the room that reach needs, so hiding it clips nothing */
	margin: 0.85rem 0;
}
.equation math {
	font-family: 'STIX Two Math', 'Cambria Math', 'Latin Modern Math', math; /* a face with a MATH table, which lays out fractions properly: STIX Two Math ships with macOS and Cambria Math with Windows */
	font-size: 1.3em;
}
</style>
