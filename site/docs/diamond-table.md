# Diamond Table

Fuji shows one picture at a time on a light table: full screen, on a plane you pan and zoom around. Its light table is the ***Diamond Table***, and the rest of this page is what the name means. Double-click a thumbnail on the contact sheet and you are on the light table; double-click the picture and you are back on the contact sheet.

## The controls

| press | to |
| --- | --- |
| `H` | show or hide **H**elp, a see-through card in the middle of the screen |
| **mouse drag** | pan |
| `←` `↑` `→` `↓` | pan a step, hands on the keyboard |
| **mouse wheel**, `Page Down`, `Page Up` | flip to the next picture, and back |
| `Space` | center the picture, sized to the diamond |
| `F` | **F**it the whole picture on the screen |
| `W` | fit the picture's **W**idth to the screen |
| `+` and `-`, or `Ctrl` and the wheel | zoom in and out about the center |
| **right drag** up or down | zoom in and out about the point you grabbed |
| `1` to `6` | exactly that many screen pixels per picture pixel |
| `Esc`, and `Backspace` on Windows and Linux | close |

## Pan anywhere

Mouse drag, and the picture moves under your hand. Keep dragging, and it keeps moving: past the edge, off the screen, until the corner of the picture sits in the middle of your display if that is where you want it. Most viewers stop you at the edge, as if the frame were a window you were looking through. Fuji's plane has no edge. The corner of a picture is sometimes the part you want to see up close, and the ***Diamond Table*** lets you put it wherever you like.

If you lose your place, press `Space`. The picture snaps to the center, sized for looking at.

## Sized to what?

Not to the screen. `F` does that, and it is right there under your left index finger whenever you want the whole picture at once; `W` beside it fits the picture's width instead, for a long screenshot or a scanned page you pan down like scrolling. `Space` sizes the picture to the diamond.

<div class="diagram">
<svg viewBox="-14 -14.5 28 28.5" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="A diamond standing on one vertex, with a wide rectangle for the screen inscribed in it and a taller, narrower rectangle for a portrait inscribed in the same diamond, overflowing the screen above and below">
	<polygon class="diamond" points="0,-12.5 12.5,0 0,12.5 -12.5,0" />
	<rect class="portrait" x="-5.36" y="-7.14" width="10.72" height="14.28" />
	<rect class="screen" x="-8" y="-4.5" width="16" height="9" />
	<text x="0" y="0.6" text-anchor="middle">your screen</text>
	<text x="0" y="-9.2" text-anchor="middle" class="label">a portrait</text>
	<text x="10.6" y="-3.6" text-anchor="middle" class="label">the diamond</text>
</svg>
</div>

Picture your monitor as a rectangle, and draw a diamond around it, a square standing on one corner, whose four edges pass through the four corners of the screen. That diamond is enormous, a frame floating in space before you, and every picture Fuji shows is sized to fit inside it the same way, its four corners on the diamond's four edges.

The geometry does the rest. A picture shaped like your screen fills the screen exactly. A portrait is taller than the screen and narrower, so it runs off the top and bottom a little and leaves a little black at the sides, and the two are always equal: exactly as much overflows at one end as is left over at the other. A wide landscape does the same the other way around, running off the sides by the same margin it leaves above and below. So portraits come in close, cropping a little at the top and bottom where there is usually sky or floor, and wide shots are cropped less, at the sides. The subject in the middle of the frame is always there, at a good size, and one drag shows you the rest.

## Zoom

`Ctrl` and the wheel zooms, a step a notch, and so do `+` and `-`, with or without `Ctrl`. For a smooth zoom, hold the right button and drag up to come in or down to go out; 200 pixels of drag doubles the picture, and dragging back to where you started puts it back as it was.

The keys and the wheel zoom about the center of the screen, not the center of the picture. Pan until the part you care about is in the middle, then zoom, and it stays in the middle while the rest of the picture grows around it or shrinks toward it. A viewer that zooms about the picture's center sends the part you were looking at sliding off toward the edge with every step, and you chase it back. The right drag is the exception, on purpose: it zooms about the point where you pressed the button, so you can put the pointer on a detail anywhere on the screen and pull it toward you.

## Viewport Persistence

The size is one number, the diamond, and it does not change when you flip. Neither does where you have panned to. Fuji calls this ***Viewport Persistence***, and it is what the diamond is for: flip forward and back through a folder and every picture arrives at the same place on the plane, in the same diamond, so if you have zoomed in on one part of the frame you stay zoomed in on that part of the frame, picture after picture.

Set a camera on a tripod and take fifty frames of the same scene, and this is what you want: pan to the figure, zoom in, and flip. The viewport holds, and the figure moves and the light changes while the frame stands still. It holds through a burst, through a time lapse, through a folder of scans of the same page, through anything where the same part of the frame matters from one picture to the next.

It holds when the pictures do not match, too. Mix a portrait into a folder of landscapes, or a phone screenshot into a folder of photographs, and the viewport is not lost, because the diamond fits every shape the same way, and a picture of a different shape lands in the same diamond, centered on the same point. When you flip on to the next landscape, you are exactly where you were.

<style scoped>
.diagram {
	max-width: 26rem;
	margin: 1.5rem auto;
}
.diagram svg {
	width: 100%;
	height: auto;
	display: block;
	font-size: 0.85px; /* the viewBox is 28 units across, so type is sized in those units */
	font-family: var(--vp-font-family-base);
}
.diagram .diamond {
	fill: none;
	stroke: var(--vp-c-text-1);
	stroke-width: 0.12;
}
.diagram .screen {
	fill: var(--vp-c-brand-soft);
	stroke: var(--vp-c-brand-1);
	stroke-width: 0.15;
}
.diagram .portrait {
	fill: none;
	stroke: var(--vp-c-text-2);
	stroke-width: 0.12;
	stroke-dasharray: 0.4 0.3;
}
.diagram text {
	fill: var(--vp-c-text-1);
}
.diagram .label {
	fill: var(--vp-c-text-2);
}
</style>
