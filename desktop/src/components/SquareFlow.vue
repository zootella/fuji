<script setup>//every thumbnail fits a square and the squares flow like words; the pixels come from the operating system or from the page, decided per file

import {ref, watch, onMounted, onBeforeUnmount} from 'vue'
import parse from 'path-browserify'
import {cacheNeed, cacheRelease} from '../cache.js'
import {modelShowing} from '../model.js'
import {settingsThumbnailBox} from '../settings.js'
import {thumbnailProbe, thumbnailRender, thumbnailUnpack} from '../thumbnail.js'
import {logTrouble, logThumbnail, logCard} from '../log.js'//the log, off unless fuji.toml says otherwise; every thumbnail and every card is a row in it
import {xy, imageTypes, errorImageData} from './library.js'

/*
The one flow, and the whole of how a path becomes a tile. A card hands this its paths. The extension says what kind of tile each gets: a GIF or an SVG is an img, so a GIF animates and an SVG is painted by the engine inside the sandbox an img is; everything else is a canvas fuji sized, which is memory the sheet can count. A canvas gets its pixels one of two ways. A format on this platform's native list goes down to Rust, and the operating system's thumbnail comes back small and goes on with one putImageData; the store never hears about the file. Anything else, and everything on linux, the store reads and decodes and the page halves down into the canvas, at a cost to the main thread.

First, one probe for the whole card: Rust reads each file's first bytes and its header and says what it is and how big, without decoding. A file whose bytes are not what its name claims, or not any format fuji knows, or whose header claims a raster that would not fit in memory, gets the placeholder and nothing is tried. Every other tile is laid out at its final size at once, so the flow does not reflow as it fills.

Two loops. The native loop keeps a few thumbnails in flight, each a pool thread that never touches the page. The page loop keeps one, each a full decode held in the store and a draw on the main thread. Both stop when the card goes away, and both wait while the sheet is hidden, so a sheet behind the table does no work inside the table's frames.

Two flows came before this one and are gone, and both of their lessons are in this file. TagFlow handed the engine full-size originals in plain img tags and let it decide everything, which is why every raster tile here is a canvas instead: the engine's thumbnail is smaller than a full decode but it is the engine's to keep or drop, and a canvas is a number of bytes fuji owns and can total. CanvasFlow painted each picture down into a canvas by hand, which is the page route below, and the halving in flowShrink is the part of it that had to be got right. The thumbnail pipeline document on fuji's site is the long version, with the measurements that chose each path.
*/

const flowHolder = 'SquareFlow'//on every reference this flow takes, so a leak has a name
const flowBox = settingsThumbnailBox()//read once: every tile is sized to it, and a change means making them all again
const flowGamut = matchMedia('(color-gamut: p3)').matches ? 'display-p3' : 'srgb'//the color space every canvas is made in, read once like the box. A canvas is sRGB unless asked, and drawing a Display P3 photograph into an sRGB canvas clamps its most saturated colors away for good, so the thumbnail would come out duller than a table shows the same file. Asking the screen what it can show, rather than asking the engine whether it knows the name, is what keeps this from being a feature check: webkitgtk has no display-p3 value and throws when handed one, and is never handed one, because the query is always false there. Stale on a change of monitor, exactly as devicePixelRatio is
const flowPlatform = platform()//mac, windows or linux, read once
const flowNative = {//what each platform's operating system makes thumbnails of, and nothing off a list is tried there. Short and conservative on purpose, and not a guess at what the machine could manage: windows gets the two decoders that have shipped in every version of windows since XP, and the mac gets the formats fuji has run ImageIO against and watched decode
	mac:     ['jpeg', 'png', 'webp', 'avif', 'bmp'],
	windows: ['jpeg', 'png'],
	linux:   [],
}
const flowInFlight = 4//native thumbnails at once; a guess for the log to correct

const props = defineProps({
	paths: {type: Array, required: true},//already in the model's order, and never from two folders
})

const flowTiles = ref(props.paths.map(path => tileFor(path)))//one small reactive object per path, built once; a load or a refusal changes one tile
const flowCanvases = new Map()//path to its canvas element, kept by the refs in the template
let flowClosed = false//the card is going away, so every loop stops
let flowHeld = []//the paths this flow holds a store reference on, which are the img tiles; released on unmount, since an img needs its url as long as it shows
let flowBytes = 0//what this card's canvases cost; the imgs are the engine's and small
let flowRefused = 0

function tileFor(path) {//what the extension says a path will be, before its bytes are read
	let format = formatOf(path)
	let tile = {path, format, kind: 'canvas', route: 'page', css: null, url: ''}//kind: canvas, img or placeholder. route: native or page, meaningful for a canvas. css: the size once known; null lays out nothing until then
	if (format == 'gif' || format == 'svg') tile.kind = 'img'
	else if (flowNative[flowPlatform].includes(format)) tile.route = 'native'
	return tile
}
function formatOf(path) {//jpeg from image/jpeg, svg from image/svg+xml: the names the probe answers with, so the two compare
	let mime = imageTypes[parse.extname(path).toLowerCase()]
	if (!mime) return ''
	return mime.split('/')[1].replace('+xml', '')
}
function platform() {
	let p = navigator.platform//MacIntel on every mac, apple silicon included; Win32 on every windows
	if (p.startsWith('Mac')) return 'mac'
	if (p.startsWith('Win')) return 'windows'
	return 'linux'
}

onMounted(() => { flowFill().catch(error => logTrouble('SquareFlow: filling a card', error)) })//the top gate for this card: anything that escapes the loops lands here, loudly
onBeforeUnmount(() => {
	flowClosed = true
	for (let path of flowHeld) cacheRelease(path, flowHolder)
})

async function flowFill() {//probe, lay out, then fill by path
	let began = performance.now()
	await flowShowing()
	let probes = await thumbnailProbe(props.paths)
	if (flowClosed) return
	for (let [i, tile] of flowTiles.value.entries()) flowApply(tile, probes[i])

	let imgs = flowTiles.value.filter(tile => tile.kind == 'img')
	let native = flowTiles.value.filter(tile => tile.kind == 'canvas' && tile.route == 'native')
	let page = flowTiles.value.filter(tile => tile.kind == 'canvas' && tile.route == 'page')
	for (let tile of imgs) flowImg(tile)//set at once; the engine loads them as it likes
	await Promise.all([flowLoop(native, flowInFlight, flowNative1), flowLoop(page, 1, flowPage1)])
	if (!flowClosed) logCard({index: props.paths.length, render: Math.round(performance.now() - began), bytes: flowBytes, note: `${native.length} native, ${page.length} page, ${imgs.length} img, ${flowRefused} refused`})
}

function flowApply(tile, probe) {//what the probe said about one file: a reason to refuse it, or its size, which reserves its box
	if (probe.problem) { flowRefuse(tile, probe.problem); return }
	if (probe.format != tile.format) { flowRefuse(tile, `the bytes say ${probe.format} and the name says ${tile.format}`); return }
	if (probe.width > 0) tile.css = flowFit(xy(probe.width, probe.height)).css
}
function flowRefuse(tile, why) {//the placeholder, and a row saying which file and why; nothing is tried twice
	tile.kind = 'placeholder'
	flowRefused++
	logThumbnail({hit: 'refused', path: tile.path, note: why})
}

function flowShowing() {//resolved once the sheet is on screen, at once if it already is
	if (modelShowing.value == 'Sheet') return Promise.resolve()
	return new Promise(resolve => {
		let stop = watch(modelShowing, showing => { if (showing == 'Sheet') { stop(); resolve() } })
	})
}

async function flowLoop(tiles, width, one) {//width workers over one list, each taking the next tile
	let next = 0
	async function worker() {
		while (next < tiles.length && !flowClosed) {
			let tile = tiles[next++]
			await flowShowing()//a sheet hidden mid-fill pauses here and resumes when shown
			if (flowClosed) return
			await one(tile)
		}
	}
	await Promise.all(Array.from({length: Math.min(width, tiles.length)}, worker))
}

async function flowNative1(tile) {//one thumbnail from the operating system, onto its canvas
	let began = performance.now()
	try {
		let buffer = await thumbnailRender(tile.path, tile.format, Math.round(flowBox * window.devicePixelRatio), flowGamut)//the longer side in backing pixels; never enlarged, so a small picture comes back at its own size
		let {width, height, gamut, pixels} = thumbnailUnpack(buffer)
		let canvas = flowCanvases.get(tile.path)
		if (flowClosed || !canvas) return
		flowSize(tile, canvas, xy(width, height))
		let context = canvas.getContext('2d', {colorSpace: flowGamut})
		context.putImageData(new ImageData(pixels, width, height, {colorSpace: gamut}), 0, 0)//tagged with what the pixels are, so windows' srgb pixels are right on a wide-gamut canvas; anything past the canvas is clipped
		flowEdge(context, canvas, width, height)//the sliver flowSnap may have added, if this thumbnail came back a device pixel short of its box
		logThumbnail({hit: 'native', render: Math.round(performance.now() - began), bytes: canvas.width * canvas.height * 4, natural: `${width}x${height}`, path: tile.path})
	} catch (error) {
		flowRefuse(tile, String(error))
	}
}

async function flowPage1(tile) {//one thumbnail made by the page from the store's decoded element, halved down; the essay above flowShrink says why halving rather than one draw
	let began = performance.now()
	let promise = cacheNeed(tile.path, flowHolder)//the reference is taken before any await, so the release below is owed from this line on
	try {
		let entry = await promise
		let canvas = flowCanvases.get(tile.path)
		if (flowClosed || !canvas) return
		if (entry.error) { flowRefuse(tile, String(entry.error)); return }
		let natural = xy(entry.img.naturalWidth, entry.img.naturalHeight)
		if (!(natural.x > 0 && natural.y > 0)) { flowRefuse(tile, 'the picture has no size'); return }

		let {scale, css} = flowFit(natural)//the size it will show at, and the ratio that got it there
		let detail = Math.min(window.devicePixelRatio, 1 / scale)//never more backing pixels than the file has; from the scale rather than the sizes, because a sliver rounds up to one css pixel
		let backing = xy(Math.max(1, Math.round(css.x * detail)), Math.max(1, Math.round(css.y * detail)))
		flowSize(tile, canvas, backing)
		flowShrink(canvas.getContext('2d', {colorSpace: flowGamut}), entry.img, natural, xy(canvas.width, canvas.height))//the canvas rather than the ask, so this route fills whatever flowSnap sized it to and never leaves an edge
		logThumbnail({hit: 'page', render: Math.round(performance.now() - began), bytes: canvas.width * canvas.height * 4, natural: `${canvas.width}x${canvas.height}`, path: tile.path})
	} catch (error) {
		flowRefuse(tile, String(error))
	} finally {
		cacheRelease(tile.path, flowHolder)//the canvas holds these pixels now; the store may drop the bytes, the url, and the full-size decode
	}
}

function flowImg(tile) {//a gif or an svg: the store's url, no decode; the css fits it to the square
	let began = performance.now()
	flowHeld.push(tile.path)
	cacheNeed(tile.path, flowHolder, {decode: false})
		.then(entry => {
			if (entry.error) { flowRefuse(tile, String(entry.error)); return }
			tile.url = entry.url
			logThumbnail({hit: 'img', render: Math.round(performance.now() - began), path: tile.path})//the read only; what the engine then spends showing it, nothing here can see
		})
		.catch(error => logTrouble('SquareFlow: loading an img thumbnail', error))//the store answers a bad file with entry.error, so this catches only a store that broke
}

function flowSize(tile, canvas, backing) {//size a canvas to its pixels; assigning width or height also clears it and resets its context, so it comes before any drawing
	tile.css = flowFit(backing).css//a returned thumbnail's longer side is the box times the ratio when it was shrunk and its own when it was not, and this rule fits both
	canvas.width = flowSnap(tile.css.x, backing.x); canvas.height = flowSnap(tile.css.y, backing.y)
	canvas.style.width = tile.css.x + 'px'; canvas.style.height = tile.css.y + 'px'//set here as well as by the template, so the element is right in the frame it is painted
	flowBytes += canvas.width * canvas.height * 4
}
function flowSnap(side, have) {//how many pixels a canvas gets for one axis: the css box in device pixels, or the pixels in hand where those cannot reach it
	/*
	A canvas is laid out on whole css pixels and painted at the device ratio, so its box is a whole number of css pixels times the ratio however the fit above rounded. A bitmap that is not exactly that many device pixels is not blitted one to one, it is resampled, and the phase of that resample walks a full pixel across the picture: on a retina panel a thumbnail one device row short of its box comes back sharp at both ends and flat grey through the middle. Half of the thumbnails the operating system makes have an odd short side, so half of them land in that state, and none of it was visible on a machine where a css pixel and a device pixel were the same thing.

	So the canvas is sized to the box rather than to the picture, and the picture is put in the corner of it. Two cases, and the constant tells them apart. A thumbnail shrunk to fit misses its box by at most one device pixel of rounding, and that sliver is worth taking, because filling it buys a one to one blit for every row; flowEdge repeats the last row and column into it so the seam is the picture's own color. A picture smaller than the box misses it by far more than that and is meant to, since the fit leaves such a picture at its own size and the engine enlarges it the way an img tag would, so that one keeps the pixels it has.
	*/
	let ratio = window.devicePixelRatio
	let want = Math.round(side * ratio)
	return want > have + ratio ? have : want
}
function flowEdge(context, canvas, width, height) {//fill whatever flowSnap left over by repeating the picture's last row and column into it; the row goes first, so the column carries it into the corner
	if (canvas.height > height) context.drawImage(canvas, 0, height - 1, width, 1, 0, height, width, canvas.height - height)
	if (canvas.width > width) context.drawImage(canvas, width - 1, 0, 1, canvas.height, width, 0, canvas.width - width, canvas.height)
}
function flowFit(size) {//the css size a picture of size pixels shows at, and the ratio that got it there: longer side to the box, never enlarged, whole pixels
	let scale = Math.min(flowBox / size.x, flowBox / size.y, 1)//the 1 keeps a small picture at its own size rather than blowing it up
	return {scale, css: xy(Math.max(1, Math.round(size.x * scale)), Math.max(1, Math.round(size.y * scale)))}//whole css pixels, because that is the grid the engine lays a box out on however this rounds; flowSize is where the pixels are then made to match it
}
function flowStyle(tile) { return tile.css ? {width: tile.css.x + 'px', height: tile.css.y + 'px'} : {} }//a tile with a known size holds its box before its pixels arrive

/*
Why this halves rather than drawing once. Fuji's first thumbnails aliased on the Mac — the roof tiles of a 26-megapixel photograph turned to jaggies at 240 pixels, while Safari showed the same file smooth as an img — and two things caused it. CoreGraphics' high interpolation reads a fixed footprint of source pixels around each output pixel, so at 26 to 1 most of the picture is never read, and pixels that are never read alias. And WebKit hands drawImage a subsampled frame only when it has to decode one: a frame already decoded at full size counts as good enough for any smaller request, and the store's decode() makes exactly that frame, so drawImage was given all 26 megapixels where Safari's img, which never called decode(), was given a quarter of them. Halving until the last draw is within two to one puts every source pixel into the average. Chromium's high quality is already a chain of halvings under a cubic filter, so on Windows this is work the engine would have done anyway.

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

<!-- items-start so a short tile keeps its own height instead of stretching to the tallest in its row; a tile with a known size holds its box from the start -->
<div class="flex flex-wrap items-start" :style="{'--box': flowBox + 'px'}">
	<template v-for="tile in flowTiles" :key="tile.path">
		<canvas v-if="tile.kind == 'canvas'"
			:ref="el => el ? flowCanvases.set(tile.path, el) : flowCanvases.delete(tile.path)"
			class="myTile" :style="flowStyle(tile)" width="0" height="0"
		></canvas>
		<img v-else-if="tile.kind == 'img' && tile.url" class="myTile myImg" :src="tile.url" :style="flowStyle(tile)" @error="flowRefuse(tile, 'the engine could not show it')" />
		<img v-else-if="tile.kind == 'placeholder'" class="myTile" :src="errorImageData" :style="{width: flowBox + 'px', height: flowBox + 'px'}" />
	</template>
</div>

</template>
<style scoped>

.myTile {
	display: block; /* a canvas and an img are inline by default, and their baselines would show as a stripe under every row */
}
.myImg {
	max-width: var(--box); /* only ever shrinks: a square lands on the box, a wide one hits the limit on width alone, and an image already smaller keeps its own size */
	max-height: var(--box);
}

</style>
