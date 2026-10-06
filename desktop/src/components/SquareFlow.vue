<script setup>//every thumbnail sized by a fit, and the tiles flow like words; the pixels come from the operating system or from the page, decided per file

import {ref, onMounted, onBeforeUnmount} from 'vue'
import parse from 'path-browserify'
import {cacheNeed, cacheRelease} from '../cache.js'
import {governorRun} from '../governor.js'
import {settings, settingsThumbnailBeam} from '../settings.js'
import {fitSize} from '../fit.js'
import {thumbnailRender, thumbnailUnpack} from '../thumbnail.js'
import {logTrouble, logThumbnail, logCard} from '../log.js'//the log, off unless fuji.toml says otherwise; every thumbnail and every card is a row in it
import {xy, errorImageData, platform} from './library.js'
import {fileTypesEnabled} from '../fileTypes.js'

/*
The one flow, and the whole of how a path becomes a tile. A card hands this its paths. The extension's entry in fileTypes.js says what kind of tile each gets on this platform: one whose contactSheet is img, a GIF or an SVG, is an img, so a GIF animates and an SVG is painted by the engine inside the sandbox an img is; everything else is a canvas fuji sized, which is memory the sheet can count, and a file nothing on this platform can draw gets the placeholder. A canvas gets its pixels one of two ways. A file whose entry lists this platform under imageNative goes down to Rust, and the operating system's thumbnail comes back small and goes on with one putImageData; the store never hears about the file. Anything else, and everything on linux, the store reads and decodes and the page halves down into the canvas, at a cost to the main thread.

Every tile is sized the same way whatever its route: from the picture's own size, by the fit thumbnail.fit names, which the sheet's toolbar sets, through flowFit and fit.js. A page tile has that size once its decode resolves, an img tile once the engine has loaded it, and a native tile from Rust, which sends it back beside the pixels after running the same fit to choose how large to render. The name says square because the square fit came first; the flow itself is wrap, which places tiles left to right like words.

Nothing is read ahead of its thumbnail. A tile takes up room when its pixels arrive, so the rows reflow as a card fills, and a file that cannot be shown becomes the placeholder when its render or its decode fails. The check on the size a file's header claims is the render's own, in thumbnail.rs, where the decoder runs unsandboxed.

Every tile on a card is asked for at once, and each waits in the line of the resource it taxes, which governor.js lets through four at a time in the order asked, so a card fills from the top. A native tile waits under rust computation, since its decode runs on a Rust pool thread and never touches the page. A page tile waits under web computation, since it is a full decode held in the store and a draw on the main thread. An img tile has no line of its own; it waits only for the store's read, which every view's reads share under disk. A card that goes away leaves its tiles in line, and each returns the moment it is let in, since a call already made cannot be called back. Nothing pauses a card, so a sheet hidden behind the table goes on filling.

Two flows came before this one and are gone, and both of their lessons are in this file. TagFlow handed the engine full-size originals in plain img tags and let it decide everything, which is why every raster tile here is a canvas instead: the engine's thumbnail is smaller than a full decode but it is the engine's to keep or drop, and a canvas is a number of bytes fuji owns and can total. CanvasFlow painted each picture down into a canvas by hand, which is the page route below, and the halving in flowShrink is the part of it that had to be got right. The thumbnail pipeline document on fuji's site is the long version, with the measurements that chose each path.
*/

const flowHolder = 'SquareFlow'//on every reference this flow takes, so a leak has a name
const flowBeam = settingsThumbnailBeam()//the length every thumbnail is measured against, in css pixels; read once: every tile is sized to it, and a change means making them all again
const flowFitName = settings.thumbnail.fit//how every tile is sized against the beam, one of fitNames in fit.js; read once like the beam
const flowScreen = xy(screen.width, screen.height)//the screen's size in css pixels, which ScaleFit and LogFit measure against; read once, and stale on a change of monitor exactly as devicePixelRatio is
const flowGamut = matchMedia('(color-gamut: p3)').matches ? 'display-p3' : 'srgb'//the color space every canvas is made in, read once like the beam. A canvas is sRGB unless asked, and drawing a Display P3 photograph into an sRGB canvas clamps its most saturated colors away for good, so the thumbnail would come out duller than a table shows the same file. Asking the screen what it can show, rather than asking the engine whether it knows the name, is what keeps this from being a feature check: webkitgtk has no display-p3 value and throws when handed one, and is never handed one, because the query is always false there. Stale on a change of monitor, exactly as devicePixelRatio is
const flowPlatform = platform()//mac, windows or linux, read once, which is how this flow reads each type's imageNative and imageWeb lists in fileTypes.js

const props = defineProps({
	paths: {type: Array, required: true},//already in the model's order, and never from two folders
})

const flowTiles = ref(props.paths.map(path => tileFor(path)))//one small reactive object per path, built once; a load or a refusal changes one tile
const flowCanvases = new Map()//path to its canvas element, kept by the refs in the template
let flowClosed = false//the card is going away, so every tile still in line returns as it is let in
let flowHeld = []//the paths this flow holds a store reference on, which are the img tiles; released on unmount, since an img needs its url as long as it shows
let flowBytes = 0//what this card's canvases cost; the imgs are the engine's and small
let flowRefused = 0

function tileFor(path) {//the kind of tile a path gets on this platform, from its extension alone
	let entry = fileTypesEnabled[parse.extname(path).toLowerCase()]
	let native = entry?.imageNative.includes(flowPlatform)
	let web = entry?.imageWeb.includes(flowPlatform)
	let kind = 'placeholder'//nothing on this platform can draw it
	if (web && entry.contactSheet == 'img') kind = 'img'//an img wins, so a GIF animates and an SVG stays vector even where the operating system could make a still of it
	else if (native) kind = 'native'//a canvas the operating system fills, preferred wherever it can
	else if (web) kind = 'page'//a canvas the page fills
	return {path, kind, url: '', css: null}//kind turns to placeholder when a render or a decode fails; url is an img tile's, once the store has it, and css its size once the engine has loaded it
}

onMounted(() => { flowFill().catch(error => logTrouble('SquareFlow: filling a card', error)) })//the top gate for this card: anything that escapes a tile's work lands here, loudly
onBeforeUnmount(() => {
	flowClosed = true
	for (let path of flowHeld) cacheRelease(path, flowHolder)
})

async function flowFill() {//ask for every tile at once, each in the line of the resource it taxes, and log the card when all of them are done
	let began = performance.now()
	let ofKind = kind => flowTiles.value.filter(tile => tile.kind == kind)
	let imgs = ofKind('img'), native = ofKind('native'), page = ofKind('page')
	for (let tile of ofKind('placeholder')) flowRefuse(tile, 'nothing on this platform can draw it')//already a placeholder; this writes its row in the log
	await Promise.all([
		...imgs.map(tile =>                                       flowImg(tile)),//not governed here: the store's read is, under disk, and governing it again would wait in that line behind itself
		...native.map(tile => governorRun('rust computation', () => flowNative1(tile))),
		...page.map(tile =>   governorRun('web computation',  () => flowPage1(tile))),//keeps its place through the store's read under disk, so at most four page tiles hold a whole file and its decode at once; governor.js says why this one caller holds two lines
	])
	if (!flowClosed) logCard({index: props.paths.length, render: Math.round(performance.now() - began), bytes: flowBytes, note: `${native.length} native, ${page.length} page, ${imgs.length} img, ${flowRefused} refused`})
}

function flowRefuse(tile, why) {//the placeholder, and a row saying which file and why; nothing is tried twice
	tile.kind = 'placeholder'
	flowRefused++
	logThumbnail({hit: 'refused', path: tile.path, note: why})
}

async function flowNative1(tile) {//one thumbnail from the operating system, onto its canvas
	if (flowClosed) return//a card that went away while this tile waited in line asks Rust for nothing
	let began = performance.now()
	try {
		let buffer = await thumbnailRender({path: tile.path, fit: flowFitName, beam: flowBeam, screenWidth: flowScreen.x, screenHeight: flowScreen.y, backingPerCss: window.devicePixelRatio, gamut: flowGamut})//rust runs the same fit on the picture's size to choose how large to render; never enlarged, so a small picture comes back at its own size
		let {width, height, gamut, naturalWidth, naturalHeight, pixels} = thumbnailUnpack(buffer)
		let canvas = flowCanvases.get(tile.path)
		if (flowClosed || !canvas) return//and one that went away while Rust worked throws the thumbnail away
		flowSize(canvas, flowFit(xy(naturalWidth, naturalHeight)).css, xy(width, height))//the tile's size from the picture's own, which rust sends back for this, as every route works it out
		let context = canvas.getContext('2d', {colorSpace: flowGamut})
		context.putImageData(new ImageData(pixels, width, height, {colorSpace: gamut}), 0, 0)//tagged with what the pixels are, so windows' srgb pixels are right on a wide-gamut canvas; anything past the canvas is clipped
		flowEdge(context, canvas, width, height)//the sliver flowSnap may have added, if this thumbnail came back a backing pixel short of its box
		logThumbnail({hit: 'native', render: Math.round(performance.now() - began), bytes: canvas.width * canvas.height * 4, natural: `${width}x${height}`, path: tile.path})
	} catch (error) {
		flowRefuse(tile, String(error))
	}
}

async function flowPage1(tile) {//one thumbnail made by the page from the store's decoded element, halved down; the essay above flowShrink says why halving rather than one draw
	if (flowClosed) return//before the store is asked, so a card that went away takes no reference it would have to give back
	let began = performance.now()
	let promise = cacheNeed(tile.path, flowHolder)//the reference is taken before any await, so the release below is owed from this line on
	try {
		let entry = await promise
		let canvas = flowCanvases.get(tile.path)
		if (flowClosed || !canvas) return
		if (entry.error) { flowRefuse(tile, String(entry.error)); return }
		let natural = xy(entry.img.naturalWidth, entry.img.naturalHeight)
		if (!(natural.x > 0 && natural.y > 0)) { flowRefuse(tile, 'the picture has no size'); return }

		let {scale, css} = flowFit(natural)//the size it will show at, and the css pixels per image pixel that got it there
		let detail = Math.min(window.devicePixelRatio, 1 / scale)//canvas pixels per css pixel: devicePixelRatio, but never more than the file has; from the scale rather than the sizes, because a sliver rounds up to one css pixel
		let backing = xy(Math.max(1, Math.round(css.x * detail)), Math.max(1, Math.round(css.y * detail)))
		flowSize(canvas, css, backing)
		flowShrink(canvas.getContext('2d', {colorSpace: flowGamut}), entry.img, natural, xy(canvas.width, canvas.height))//the canvas rather than the ask, so this route fills whatever flowSnap sized it to and never leaves an edge
		logThumbnail({hit: 'page', render: Math.round(performance.now() - began), bytes: canvas.width * canvas.height * 4, natural: `${canvas.width}x${canvas.height}`, path: tile.path})
	} catch (error) {
		flowRefuse(tile, String(error))
	} finally {
		cacheRelease(tile.path, flowHolder)//the canvas holds these pixels now; the store may drop the bytes, the url, and the full-size decode
	}
}

function flowImg(tile) {//a gif or an svg: the store's url, no decode; flowImgLoad sizes it once the engine has it. Resolves once the url is set or refused
	let began = performance.now()
	flowHeld.push(tile.path)
	return cacheNeed(tile.path, flowHolder, {decode: false})
		.then(entry => {
			if (entry.error) { flowRefuse(tile, String(entry.error)); return }
			tile.url = entry.url
			logThumbnail({hit: 'img', render: Math.round(performance.now() - began), path: tile.path})//the read only; what the engine then spends showing it, nothing here can see
		})
		.catch(error => logTrouble('SquareFlow: loading an img thumbnail', error))//the store answers a bad file with entry.error, so this catches only a store that broke
}

function flowImgLoad(tile, event) {//an img tile, sized by the fit once the engine knows its picture's size; until then, and for an svg with no size of its own, the css in myImg holds it to the beam's square
	let natural = xy(event.target.naturalWidth, event.target.naturalHeight)
	if (natural.x > 0 && natural.y > 0) tile.css = flowFit(natural).css
}

function flowSize(canvas, css, backing) {//size a canvas to the css size its fit chose and the backing pixels it has, which is also what gives its tile room in the flow; assigning width or height also clears it and resets its context, so it comes before any drawing
	canvas.width = flowSnap(css.x, backing.x); canvas.height = flowSnap(css.y, backing.y)
	canvas.style.width = css.x + 'px'; canvas.style.height = css.y + 'px'//the only place a canvas gets its css size, so the template binds none and never overwrites it
	flowBytes += canvas.width * canvas.height * 4
}
function flowSnap(side, have) {//how many pixels a canvas gets for one axis: the css box in backing pixels, or the pixels in hand where those cannot reach it
	/*
	A canvas is laid out on whole css pixels and painted in backing pixels, so its box is a whole number of css pixels times devicePixelRatio however the fit above rounded. A bitmap that is not exactly that many backing pixels is not blitted one to one, it is resampled, and the phase of that resample walks a full pixel across the picture: on a retina panel a thumbnail one backing row short of its box comes back sharp at both ends and flat grey through the middle. Half of the thumbnails the operating system makes have an odd short side, so half of them land in that state, and none of it was visible on a machine where a css pixel and a backing pixel were the same thing.

	So the canvas is sized to the box rather than to the picture, and the picture is put in the corner of it. Two cases, and the constant tells them apart. A thumbnail shrunk to fit misses its box by at most one backing pixel of rounding, and that sliver is worth taking, because filling it buys a one to one blit for every row; flowEdge repeats the last row and column into it so the seam is the picture's own color. A picture smaller than the box misses it by far more than that and is meant to, since the fit leaves such a picture at its own size and the engine enlarges it the way an img tag would, so that one keeps the pixels it has.

	Nearest rather than down, and flooring was tried and measured and is worse. On a mac the choice is invisible: devicePixelRatio is 1 or 2, so the box is always a whole number of backing pixels and the two agree exactly. Windows scales at 125, 150 and 175 percent, where a box on whole css pixels lands on a quarter, a half or three quarters of a backing pixel and no canvas can sit on it either way. There the two differ on half of all sizes, and the reasoning that said flooring should win was this: across three scales, every canvas that came out larger than its box and whose row had drifted off the backing grid was resampled, while every canvas smaller than its box was clean. Never overshoot, and the compositor has nothing to resample.

	It does not hold. Every one of those clean undershoots was short by exactly a quarter of a backing pixel, which is what rounding happens to produce; flooring makes them short by a half or three quarters, and those resample as readily as an overshoot does — measured at 175 percent, where flooring fixed two tiles and broke three, including two in a row sitting squarely on the grid. A canvas cannot be made to fit a box that is not a whole number of backing pixels, and how far it misses by matters more than which side it misses on. So this rounds, the miss is at most half a backing pixel in either direction, and the tiles that still resample are left to the layout question that owns them.

	The thumbnail pipeline document on fuji's site has the tiles all of this was read from.
	*/
	let backingPerCss = window.devicePixelRatio
	let want = Math.round(side * backingPerCss)//nearest, because the size of the miss matters more than its direction; flooring was measured and was worse
	return want > have + backingPerCss ? have : want
}
function flowEdge(context, canvas, width, height) {//fill whatever flowSnap left over by repeating the picture's last row and column into it; the row goes first, so the column carries it into the corner
	if (canvas.height > height) context.drawImage(canvas, 0, height - 1, width, 1, 0, height, width, canvas.height - height)
	if (canvas.width > width) context.drawImage(canvas, width - 1, 0, 1, canvas.height, width, 0, canvas.width - width, canvas.height)
}
function flowFit(natural) {//the css size a picture of natural image pixels shows at under this flow's fit, and the css pixels per image pixel that got it there; every route sizes its tile here, from the picture's own size, and fit.js has the fits themselves
	let fitted = fitSize({fit: flowFitName, width: natural.x, height: natural.y, beam: flowBeam, screenWidth: flowScreen.x, screenHeight: flowScreen.y})
	return {scale: fitted.scale, css: xy(fitted.width, fitted.height)}//whole css pixels, because that is the grid the engine lays a box out on; flowSize is where the pixels are then made to match it
}

/*
Why this halves rather than drawing once. Fuji's first thumbnails aliased on the Mac — the roof tiles of a 26-megapixel photograph turned to jaggies at 240 css pixels, while Safari showed the same file smooth as an img — and two things caused it. CoreGraphics' high interpolation reads a fixed footprint of source pixels around each output pixel, so at 26 to 1 most of the picture is never read, and pixels that are never read alias. And WebKit hands drawImage a subsampled frame only when it has to decode one: a frame already decoded at full size counts as good enough for any smaller request, and the store's decode() makes exactly that frame, so drawImage was given all 26 megapixels where Safari's img, which never called decode(), was given a quarter of them. Halving until the last draw is within two to one puts every source pixel into the average. Chromium's high quality is already a chain of halvings under a cubic filter, so on Windows this is work the engine would have done anyway.

createImageBitmap looks like the purpose-built tool here, and on Chromium it is: ask it for resizeWidth and resizeQuality and it decodes straight to the size wanted. WebKit has the code and does not ship the options, so on the Mac it handed back a full-size bitmap — a whole second copy of a large photograph, allocated for nothing, then scaled by the same drawImage that could have done the job alone. It was tried and taken back out.
*/
function flowShrink(context, source, size, target) {//draw source, of size pixels, into context at target pixels, halving through scratch canvases until the last draw is within two to one; a fixed-footprint filter is only an honest average at that ratio
	let scratch = [document.createElement('canvas'), document.createElement('canvas')]//two, taken in turn, because a canvas cannot be shrunk into itself
	let step = 0
	while (size.x > target.x * 2) {//one axis is enough, because the fit kept the aspect; strictly more than double, so the last draw below is between one to one and two to one
		let half = xy(Math.ceil(size.x / 2), Math.ceil(size.y / 2))
		let canvas = scratch[step % 2]//never the one that is the current source
		canvas.width = half.x; canvas.height = half.y//sizing clears it and resets its context, so the quality is set again below
		let c = canvas.getContext('2d', {colorSpace: flowGamut})//the same gamut all the way down, so nothing is converted twice
		c.imageSmoothingQuality = 'high'
		c.drawImage(source, 0, 0, half.x, half.y)//the first step reads the store's element, and the engine applies the file's orientation there; every later step reads a canvas
		source = canvas; size = half; step++
	}
	context.imageSmoothingQuality = 'high'//rather than the default low, which reads only the four nearest source pixels per output pixel
	context.drawImage(source, 0, 0, target.x, target.y)
	for (let canvas of scratch) { canvas.width = 0; canvas.height = 0 }//give the backing stores back now; the first is a quarter of the picture's own size
}

</script>
<template>

<!-- every tile names its path in data-path, which is how the sheet knows which one was double-clicked without a handler on each. items-start so a short tile keeps its own height instead of stretching to the tallest in its row, and gap-1.5 with p-1.5 for 6 css pixels between tiles and between a tile and the card's edge, so two cards meet 12 apart -->
<div class="flex flex-wrap items-start gap-1.5 p-1.5" :style="{'--beam': flowBeam + 'px'}">
	<template v-for="tile in flowTiles" :key="tile.path">
		<canvas v-if="tile.kind == 'native' || tile.kind == 'page'"
			:ref="el => el ? flowCanvases.set(tile.path, el) : flowCanvases.delete(tile.path)"
			class="myTile" :data-path="tile.path" width="0" height="0"
		></canvas>
		<img v-else-if="tile.kind == 'img' && tile.url" class="myTile" :class="{myImg: !tile.css}" :data-path="tile.path" :src="tile.url" :style="tile.css ? {width: tile.css.x + 'px', height: tile.css.y + 'px'} : {}" @load="flowImgLoad(tile, $event)" @error="flowRefuse(tile, 'the engine could not show it')" />
		<img v-else-if="tile.kind == 'placeholder'" class="myTile" :data-path="tile.path" :src="errorImageData" :style="{width: flowBeam + 'px', height: flowBeam + 'px'}" />
	</template>
</div>

</template>
<style scoped>

.myTile {
	display: block; /* a canvas and an img are inline by default, and their baselines would show as a stripe under every row */
	border-radius: 8px; /* a clip the compositor draws, so the pixels inside are blitted exactly as before; it only hides the few in each corner, and a tile shorter than 16px gets a smaller radius from css itself */
	outline: 2px solid var(--color-brand); /* an outline rather than a border, which would take its width from each side of the box the canvas is sized to and resample every thumbnail into what was left; an outline takes no room and follows the radius */
	outline-offset: -2px; /* drawn just inside the edge, over the picture's outermost two css pixels, so the outline sits on the tile's own box and the gap between tiles stays the gap the flow set */
}
.myImg { /* an img tile until flowImgLoad has sized it by the fit, and for good if its picture has no size of its own */
	max-width: var(--beam); /* only ever shrinks: a square lands on the beam, a wide one hits the limit on width alone, and an image already smaller keeps its own size */
	max-height: var(--beam);
}

</style>
